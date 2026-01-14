use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiProvider {
    Local,
    OpenAI,
    Anthropic,
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::Local => write!(f, "Local"),
            AiProvider::OpenAI => write!(f, "OpenAI"),
            AiProvider::Anthropic => write!(f, "Anthropic"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiMode {
    Off,           // No suggestions
    LocalOnly,     // Only local Markov suggestions
    ApiOnly,       // Only API suggestions
    Hybrid,        // Local + API fallback
}

impl std::fmt::Display for AiMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiMode::Off => write!(f, "Off"),
            AiMode::LocalOnly => write!(f, "Local"),
            AiMode::ApiOnly => write!(f, "API"),
            AiMode::Hybrid => write!(f, "Hybrid"),
        }
    }
}

pub const OPENAI_MODELS: &[(&str, &str)] = &[
    ("gpt-4o", "GPT-4o (Best)"),
    ("gpt-4o-mini", "GPT-4o Mini (Fast)"),
    ("gpt-4-turbo", "GPT-4 Turbo"),
    ("gpt-3.5-turbo", "GPT-3.5 Turbo (Cheap)"),
];

pub const ANTHROPIC_MODELS: &[(&str, &str)] = &[
    ("claude-sonnet-4-20250514", "Claude Sonnet 4 (Best)"),
    ("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet"),
    ("claude-3-5-haiku-20241022", "Claude 3.5 Haiku (Fast)"),
    ("claude-3-haiku-20240307", "Claude 3 Haiku (Cheap)"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub openai_api_key: Option<String>,
    pub openai_model: String,
    pub openai_model_index: usize,
    pub anthropic_api_key: Option<String>,
    pub anthropic_model: String,
    pub anthropic_model_index: usize,
    pub ai_provider: AiProvider,
    pub ai_mode: AiMode,
    pub pulse_speed_ms: u64,
    pub suggestion_delay_ms: u64,
    pub auto_suggest: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            openai_model: "gpt-4o-mini".to_string(),
            openai_model_index: 1,
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            anthropic_model: "claude-3-5-haiku-20241022".to_string(),
            anthropic_model_index: 2,
            ai_provider: AiProvider::Local,
            ai_mode: AiMode::Hybrid,
            pulse_speed_ms: 800,
            suggestion_delay_ms: 2000,
            auto_suggest: true,
        }
    }
}

impl Config {
    pub fn has_api_key(&self) -> bool {
        match self.ai_provider {
            AiProvider::Local => true,
            AiProvider::OpenAI => self.openai_api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false),
            AiProvider::Anthropic => self.anthropic_api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false),
        }
    }

    pub fn has_openai_key(&self) -> bool {
        self.openai_api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false)
    }

    pub fn has_anthropic_key(&self) -> bool {
        self.anthropic_api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false)
    }

    pub fn has_any_api_key(&self) -> bool {
        self.has_openai_key() || self.has_anthropic_key()
    }

    pub fn current_model(&self) -> &str {
        match self.ai_provider {
            AiProvider::Local => "Markov",
            AiProvider::OpenAI => &self.openai_model,
            AiProvider::Anthropic => &self.anthropic_model,
        }
    }

    pub fn current_model_display(&self) -> &str {
        match self.ai_provider {
            AiProvider::Local => "Local Markov",
            AiProvider::OpenAI => {
                OPENAI_MODELS.get(self.openai_model_index)
                    .map(|(_, name)| *name)
                    .unwrap_or("GPT")
            }
            AiProvider::Anthropic => {
                ANTHROPIC_MODELS.get(self.anthropic_model_index)
                    .map(|(_, name)| *name)
                    .unwrap_or("Claude")
            }
        }
    }

    pub fn cycle_provider(&mut self) {
        self.ai_provider = match self.ai_provider {
            AiProvider::Local => {
                if self.has_openai_key() {
                    AiProvider::OpenAI
                } else if self.has_anthropic_key() {
                    AiProvider::Anthropic
                } else {
                    AiProvider::Local
                }
            }
            AiProvider::OpenAI => {
                if self.has_anthropic_key() {
                    AiProvider::Anthropic
                } else {
                    AiProvider::Local
                }
            }
            AiProvider::Anthropic => AiProvider::Local,
        };
    }

    pub fn cycle_model(&mut self) {
        match self.ai_provider {
            AiProvider::Local => {}
            AiProvider::OpenAI => {
                self.openai_model_index = (self.openai_model_index + 1) % OPENAI_MODELS.len();
                self.openai_model = OPENAI_MODELS[self.openai_model_index].0.to_string();
            }
            AiProvider::Anthropic => {
                self.anthropic_model_index = (self.anthropic_model_index + 1) % ANTHROPIC_MODELS.len();
                self.anthropic_model = ANTHROPIC_MODELS[self.anthropic_model_index].0.to_string();
            }
        }
    }

    pub fn cycle_mode(&mut self) {
        self.ai_mode = match self.ai_mode {
            AiMode::Off => AiMode::LocalOnly,
            AiMode::LocalOnly => AiMode::ApiOnly,
            AiMode::ApiOnly => AiMode::Hybrid,
            AiMode::Hybrid => AiMode::Off,
        };
    }

    pub fn toggle_auto_suggest(&mut self) {
        self.auto_suggest = !self.auto_suggest;
    }
}
