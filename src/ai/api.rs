use serde::{Deserialize, Serialize};
use super::{Suggestion, SuggestionSource};
use crate::config::Config;

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

pub struct OpenAIClient {
    client: reqwest::Client,
    config: Config,
}

impl OpenAIClient {
    pub fn new(config: Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    pub async fn suggest(&self, context: &str) -> Option<Suggestion> {
        let api_key = self.config.openai_api_key.as_ref()?;

        if api_key.is_empty() {
            return None;
        }

        let prompt = format!(
            "You are a creative writing assistant. Continue this text with 3-8 words. \
             Be creative, unexpected, and slightly surreal. Only output the continuation, nothing else.\n\n\
             Text: {}",
            context
        );

        let request = ChatRequest {
            model: self.config.openai_model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: 30,
            temperature: 0.9,
        };

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .ok()?;

        let chat_response: ChatResponse = response.json().await.ok()?;
        let text = chat_response.choices.first()?.message.content.trim().to_string();

        if text.is_empty() {
            return None;
        }

        Some(Suggestion {
            text: format!(" {}", text),
            confidence: 0.85,
            source: SuggestionSource::Api,
        })
    }

    pub async fn suggest_sentence(&self, context: &str) -> Option<Suggestion> {
        let api_key = self.config.openai_api_key.as_ref()?;

        if api_key.is_empty() {
            return None;
        }

        let prompt = format!(
            "You are an experimental creative writing assistant. Complete this partial text with one full sentence. \
             Be surreal, dreamlike, and unexpected. Only output the sentence continuation, nothing else.\n\n\
             Text: {}",
            context
        );

        let request = ChatRequest {
            model: self.config.openai_model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: 60,
            temperature: 1.0,
        };

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .ok()?;

        let chat_response: ChatResponse = response.json().await.ok()?;
        let text = chat_response.choices.first()?.message.content.trim().to_string();

        if text.is_empty() {
            return None;
        }

        Some(Suggestion {
            text,
            confidence: 0.9,
            source: SuggestionSource::Api,
        })
    }
}
