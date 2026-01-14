mod local;
mod api;
mod anthropic;
mod hybrid;

pub use local::MarkovEngine;
pub use api::OpenAIClient;
pub use anthropic::AnthropicClient;
pub use hybrid::HybridEngine;

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub text: String,
    pub confidence: f32,
    pub source: SuggestionSource,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SuggestionSource {
    Local,
    Api,
}

#[derive(Debug)]
pub enum ApiResponse {
    WordSuggestion(Option<Suggestion>),
    SentenceSuggestion(Option<Suggestion>),
}
