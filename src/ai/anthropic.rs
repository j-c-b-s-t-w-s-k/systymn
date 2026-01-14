use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use super::{Suggestion, SuggestionSource};

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: String,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

pub struct AnthropicClient {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicClient {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            api_key: config.anthropic_api_key.unwrap_or_default(),
            model: "claude-3-haiku-20240307".to_string(), // Fast and cheap
        }
    }

    pub fn with_model(config: Config, model: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: config.anthropic_api_key.unwrap_or_default(),
            model: model.to_string(),
        }
    }

    pub async fn suggest(&self, context: &str) -> Option<Suggestion> {
        if self.api_key.is_empty() {
            return None;
        }

        let system_prompt = "You are a creative writing assistant. Given the text context, suggest the next 1-5 words that would naturally continue the writing. Only output the suggested words, nothing else. No quotes, no explanations.";

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 50,
            messages: vec![Message {
                role: "user".to_string(),
                content: format!("Continue this text with the next few words:\n\n{}", context),
            }],
            system: system_prompt.to_string(),
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            return None;
        }

        let data: AnthropicResponse = response.json().await.ok()?;

        let suggestion_text = data.content.first()?.text.trim().to_string();

        if suggestion_text.is_empty() {
            return None;
        }

        // Ensure suggestion starts with space if context doesn't end with whitespace
        let final_text = if !context.ends_with(char::is_whitespace) && !suggestion_text.starts_with(char::is_whitespace) {
            format!(" {}", suggestion_text)
        } else {
            suggestion_text
        };

        Some(Suggestion {
            text: final_text,
            confidence: 0.85,
            source: SuggestionSource::Api,
        })
    }

    pub async fn suggest_sentence(&self, context: &str) -> Option<Suggestion> {
        if self.api_key.is_empty() {
            return None;
        }

        let system_prompt = "You are a creative writing assistant. Given the text context, suggest a complete sentence or phrase (10-20 words) that would naturally continue the writing. Only output the suggested text, nothing else. No quotes, no explanations.";

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 100,
            messages: vec![Message {
                role: "user".to_string(),
                content: format!("Continue this text with a natural sentence:\n\n{}", context),
            }],
            system: system_prompt.to_string(),
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            return None;
        }

        let data: AnthropicResponse = response.json().await.ok()?;

        let suggestion_text = data.content.first()?.text.trim().to_string();

        if suggestion_text.is_empty() {
            return None;
        }

        let final_text = if !context.ends_with(char::is_whitespace) && !suggestion_text.starts_with(char::is_whitespace) {
            format!(" {}", suggestion_text)
        } else {
            suggestion_text
        };

        Some(Suggestion {
            text: final_text,
            confidence: 0.90,
            source: SuggestionSource::Api,
        })
    }

    pub fn model_name(&self) -> &str {
        &self.model
    }
}
