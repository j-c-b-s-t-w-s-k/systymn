use rand::prelude::*;
use std::collections::HashMap;
use super::{Suggestion, SuggestionSource};

const SEED_TEXT: &str = include_str!("../../data/markov_seed.txt");

pub struct MarkovEngine {
    chains: HashMap<String, Vec<(String, u32)>>,
    word_completions: HashMap<String, Vec<String>>,
}

impl MarkovEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            chains: HashMap::new(),
            word_completions: HashMap::new(),
        };
        engine.train(SEED_TEXT);
        engine
    }

    fn train(&mut self, text: &str) {
        let words: Vec<&str> = text.split_whitespace().collect();

        // Train bigram chains
        for window in words.windows(2) {
            let key = window[0].to_lowercase();
            let next = window[1].to_string();

            let entry = self.chains.entry(key).or_insert_with(Vec::new);
            if let Some(existing) = entry.iter_mut().find(|(w, _)| w == &next) {
                existing.1 += 1;
            } else {
                entry.push((next, 1));
            }
        }

        // Build word completions
        for word in &words {
            let lower = word.to_lowercase();
            for i in 1..=lower.len().min(4) {
                let prefix = &lower[..i];
                let entry = self.word_completions.entry(prefix.to_string()).or_insert_with(Vec::new);
                if !entry.contains(&lower) {
                    entry.push(lower.clone());
                }
            }
        }
    }

    pub fn suggest(&self, context: &str) -> Vec<Suggestion> {
        let words: Vec<&str> = context.split_whitespace().collect();
        let mut suggestions = Vec::new();
        let mut rng = thread_rng();

        if let Some(last_word) = words.last() {
            let last_lower = last_word.to_lowercase();

            // Check if we're mid-word (no trailing space)
            if !context.ends_with(' ') && !context.ends_with('\n') {
                // Word completion
                if let Some(completions) = self.word_completions.get(&last_lower) {
                    for comp in completions.iter().take(3) {
                        if comp.len() > last_lower.len() && comp.starts_with(&last_lower) {
                            let suffix = &comp[last_lower.len()..];
                            suggestions.push(Suggestion {
                                text: suffix.to_string(),
                                confidence: 0.6,
                                source: SuggestionSource::Local,
                            });
                        }
                    }
                }
            } else {
                // Next word prediction
                if let Some(nexts) = self.chains.get(&last_lower) {
                    let total: u32 = nexts.iter().map(|(_, c)| c).sum();
                    let mut sorted: Vec<_> = nexts.iter().collect();
                    sorted.sort_by(|a, b| b.1.cmp(&a.1));

                    for (word, count) in sorted.into_iter().take(3) {
                        let confidence = *count as f32 / total as f32;
                        suggestions.push(Suggestion {
                            text: format!(" {}", word),
                            confidence: confidence.min(0.8),
                            source: SuggestionSource::Local,
                        });
                    }
                }
            }
        }

        // Add some randomness - occasionally suggest unexpected continuations
        if suggestions.is_empty() || rng.gen_bool(0.2) {
            let random_starters = [
                "suddenly", "perhaps", "meanwhile", "beneath", "through",
                "silently", "eventually", "somewhere", "beyond", "within"
            ];
            if let Some(word) = random_starters.choose(&mut rng) {
                suggestions.push(Suggestion {
                    text: format!(" {}", word),
                    confidence: 0.3,
                    source: SuggestionSource::Local,
                });
            }
        }

        suggestions
    }

    pub fn suggest_sentence(&self, context: &str) -> Option<Suggestion> {
        let mut rng = thread_rng();
        let words: Vec<&str> = context.split_whitespace().collect();

        if words.is_empty() {
            return None;
        }

        let last_word = words.last()?.to_lowercase();
        let mut sentence = String::new();
        let mut current = last_word.clone();

        for _ in 0..8 {
            if let Some(nexts) = self.chains.get(&current) {
                let total: u32 = nexts.iter().map(|(_, c)| c).sum();
                let mut pick: u32 = rng.gen_range(0..total.max(1));

                for (word, count) in nexts {
                    if pick < *count {
                        sentence.push(' ');
                        sentence.push_str(word);
                        current = word.to_lowercase();
                        break;
                    }
                    pick -= count;
                }
            } else {
                break;
            }
        }

        if sentence.len() > 10 {
            Some(Suggestion {
                text: sentence,
                confidence: 0.4,
                source: SuggestionSource::Local,
            })
        } else {
            None
        }
    }
}

impl Default for MarkovEngine {
    fn default() -> Self {
        Self::new()
    }
}
