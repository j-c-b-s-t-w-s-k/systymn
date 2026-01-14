use std::time::Instant;
use tokio::sync::mpsc;
use super::{MarkovEngine, OpenAIClient, Suggestion};
use crate::config::{Config, AiMode};

pub struct HybridEngine {
    local: MarkovEngine,
    api: OpenAIClient,
    config: Config,
    last_api_call: Option<Instant>,
}

impl HybridEngine {
    pub fn new(config: Config) -> Self {
        Self {
            local: MarkovEngine::new(),
            api: OpenAIClient::new(config.clone()),
            config,
            last_api_call: None,
        }
    }

    pub fn suggest_local(&self, context: &str) -> Vec<Suggestion> {
        self.local.suggest(context)
    }

    pub fn suggest_sentence_local(&self, context: &str) -> Option<Suggestion> {
        self.local.suggest_sentence(context)
    }

    pub fn should_call_api(&self, local_suggestions: &[Suggestion]) -> bool {
        // Only use API in Hybrid or ApiOnly mode
        let use_api = matches!(self.config.ai_mode, AiMode::Hybrid | AiMode::ApiOnly);
        if !self.config.has_api_key() || !use_api {
            return false;
        }

        // Check if enough time has passed since last API call
        if let Some(last_call) = self.last_api_call {
            if last_call.elapsed().as_millis() < self.config.suggestion_delay_ms as u128 {
                return false;
            }
        }

        // Call API if local confidence is low
        let max_confidence = local_suggestions
            .iter()
            .map(|s| s.confidence)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        max_confidence < 0.5
    }

    pub fn start_api_request(&mut self, context: String, tx: mpsc::Sender<Option<Suggestion>>) {
        self.last_api_call = Some(Instant::now());
        let api = OpenAIClient::new(self.config.clone());

        tokio::spawn(async move {
            let result = api.suggest(&context).await;
            let _ = tx.send(result).await;
        });
    }

    pub fn start_sentence_request(&mut self, context: String, tx: mpsc::Sender<Option<Suggestion>>) {
        self.last_api_call = Some(Instant::now());
        let api = OpenAIClient::new(self.config.clone());

        tokio::spawn(async move {
            let result = api.suggest_sentence(&context).await;
            let _ = tx.send(result).await;
        });
    }
}
