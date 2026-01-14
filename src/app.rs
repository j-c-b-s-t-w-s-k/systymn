use std::fs;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::buffer::TextBuffer;
use crate::ai::{ApiResponse, HybridEngine, OpenAIClient, AnthropicClient, Suggestion};
use crate::config::{AiProvider, AiMode};
use crate::commands::{CommandParser, Generators};
use crate::config::Config;
use crate::emoji::EmojiEngine;
use crate::search::{SearchState, SearchMode};
use crate::ui::synonyms::get_synonyms;

const MAX_UNDO_HISTORY: usize = 100;

#[derive(Clone)]
struct EditorState {
    buffer: TextBuffer,
    cursor: (usize, usize),
}

pub struct App {
    pub buffer: TextBuffer,
    pub config: Config,
    pub current_suggestion: Option<Suggestion>,
    pub sentence_suggestion: Option<Suggestion>,
    pub api_suggestion: Option<Suggestion>,
    pub command_preview: Option<String>,
    pub pulse_phase: f32,
    pub show_synonyms: bool,
    pub synonyms: Vec<String>,
    pub synonym_index: usize,
    pub synonym_word: Option<String>,
    pub synonym_range: Option<(usize, usize)>,
    pub show_help: bool,
    pub file_path: Option<PathBuf>,
    pub emoji_mode: bool,
    pub scroll_offset: usize,
    pub wrap_width: usize,
    pub api_loading: bool,
    pub status_message: Option<String>,
    ai: HybridEngine,
    emoji: EmojiEngine,
    tick_count: u64,
    api_tx: mpsc::Sender<ApiResponse>,
    undo_stack: Vec<EditorState>,
    redo_stack: Vec<EditorState>,
    // Clipboard
    pub clipboard: String,
    // Search
    pub search: SearchState,
}

impl App {
    pub fn new(api_tx: mpsc::Sender<ApiResponse>) -> Self {
        let config = Config::default();
        Self {
            buffer: TextBuffer::new(),
            ai: HybridEngine::new(config.clone()),
            emoji: EmojiEngine::new(),
            config,
            current_suggestion: None,
            sentence_suggestion: None,
            api_suggestion: None,
            command_preview: None,
            pulse_phase: 0.0,
            show_synonyms: false,
            synonyms: Vec::new(),
            synonym_index: 0,
            synonym_word: None,
            synonym_range: None,
            show_help: false,
            file_path: None,
            emoji_mode: false,
            scroll_offset: 0,
            wrap_width: 80,
            api_loading: false,
            status_message: None,
            tick_count: 0,
            api_tx,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: String::new(),
            search: SearchState::new(),
        }
    }

    fn save_state(&mut self) {
        let state = EditorState {
            buffer: self.buffer.clone(),
            cursor: self.buffer.cursor(),
        };
        self.undo_stack.push(state);
        if self.undo_stack.len() > MAX_UNDO_HISTORY {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            let current = EditorState {
                buffer: self.buffer.clone(),
                cursor: self.buffer.cursor(),
            };
            self.redo_stack.push(current);
            self.buffer = state.buffer;
            self.update_suggestions();
        }
    }

    pub fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            let current = EditorState {
                buffer: self.buffer.clone(),
                cursor: self.buffer.cursor(),
            };
            self.undo_stack.push(current);
            self.buffer = state.buffer;
            self.update_suggestions();
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;

        // Update pulse phase (0.0 to 1.0 over ~800ms at 50ms tick rate)
        self.pulse_phase = (self.tick_count as f32 * 0.0625) % 1.0;

        // Clear status message after a while
        if self.tick_count % 60 == 0 {
            self.status_message = None;
        }

        // Regenerate suggestions periodically
        if self.tick_count % 10 == 0 {
            self.update_suggestions();
        }

        // Auto-fetch API suggestions when local confidence is low (every ~5 seconds)
        if self.tick_count % 100 == 0 && self.config.has_api_key() && !self.api_loading {
            let context = self.buffer.text_before_cursor();
            if context.len() > 30 {
                if let Some(suggestion) = &self.current_suggestion {
                    if suggestion.confidence < 0.4 {
                        self.fetch_api_suggestion();
                    }
                }
            }
        }
    }

    fn update_suggestions(&mut self) {
        let context = self.buffer.text_before_cursor();

        // Check for slash commands first
        if let Some(partial) = CommandParser::is_partial_command(&context) {
            // Check for emoji commands
            if partial.starts_with("emoji") || partial == "e" {
                self.command_preview = Some("emoji categories: face, nature, animal, object, food, heart".to_string());
            } else {
                self.command_preview = Some(format!("/{} (press Enter)", partial));
            }
            self.current_suggestion = None;
            return;
        }

        self.command_preview = None;

        // Get local suggestions
        let suggestions = self.ai.suggest_local(&context);
        self.current_suggestion = suggestions.into_iter().next();

        // Add emoji suggestion if emoji mode is on
        if self.emoji_mode {
            if let Some(emoji) = self.emoji.suggest_emoji(&context) {
                self.current_suggestion = Some(Suggestion {
                    text: format!(" {}", emoji),
                    confidence: 0.7,
                    source: crate::ai::SuggestionSource::Local,
                });
            }
        }

        // Occasionally generate sentence suggestion
        if self.tick_count % 40 == 0 && context.len() > 20 {
            self.sentence_suggestion = self.ai.suggest_sentence_local(&context);
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.save_state();
        self.buffer.insert_char(c);
        self.update_suggestions();
    }

    pub fn handle_enter(&mut self) {
        if self.show_synonyms {
            self.select_synonym();
            return;
        }

        // Check for command execution
        let context = self.buffer.text_before_cursor();
        if let Some((cmd, start, end)) = CommandParser::parse(&context) {
            self.save_state();

            // Handle emoji commands separately
            let generated = match &cmd {
                crate::commands::Command::Emoji(Some(category)) => {
                    self.emoji.emoji_by_category(category).unwrap_or("\u{2728}").to_string()
                }
                crate::commands::Command::Emoji(None) => {
                    self.emoji.random_emoji().to_string()
                }
                _ => Generators::generate(&cmd),
            };

            // Replace command with generated text
            let current_line = self.buffer.cursor().1;
            let line_start = self.buffer.lines()[..current_line]
                .iter()
                .map(|l| l.len() + 1)
                .sum::<usize>();
            let rel_start = start.saturating_sub(line_start);
            let rel_end = end.saturating_sub(line_start);

            self.buffer.replace_word(rel_start, rel_end, &generated);
            self.command_preview = None;
            self.update_suggestions();
        } else {
            self.save_state();
            self.buffer.insert_newline();
            self.update_suggestions();
        }
    }

    pub fn handle_backspace(&mut self) {
        self.save_state();
        self.buffer.backspace();
        self.update_suggestions();
    }

    pub fn handle_delete(&mut self) {
        self.save_state();
        self.buffer.delete();
        self.update_suggestions();
    }

    pub fn move_cursor_left(&mut self) {
        self.buffer.move_left();
    }

    pub fn move_cursor_right(&mut self) {
        self.buffer.move_right();
    }

    pub fn move_cursor_up(&mut self) {
        if self.show_synonyms {
            self.synonym_up();
        } else {
            self.buffer.move_up();
            self.adjust_scroll();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.show_synonyms {
            self.synonym_down();
        } else {
            self.buffer.move_down();
            self.adjust_scroll();
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.buffer.move_home();
    }

    pub fn move_cursor_end(&mut self) {
        self.buffer.move_end();
    }

    pub fn move_to_start(&mut self) {
        self.buffer.move_to_start();
        self.scroll_offset = 0;
    }

    pub fn move_to_end(&mut self) {
        self.buffer.move_to_end();
        self.adjust_scroll();
    }

    pub fn page_up(&mut self) {
        for _ in 0..20 {
            self.buffer.move_up();
        }
        self.adjust_scroll();
    }

    pub fn page_down(&mut self) {
        for _ in 0..20 {
            self.buffer.move_down();
        }
        self.adjust_scroll();
    }

    fn adjust_scroll(&mut self) {
        let (_, cursor_y) = self.buffer.cursor();
        if cursor_y < self.scroll_offset {
            self.scroll_offset = cursor_y;
        }
        // This will be properly set based on visible height in render
    }

    // ========== Selection & Clipboard ==========

    pub fn start_selection(&mut self) {
        self.buffer.start_selection();
    }

    pub fn select_left(&mut self) {
        if !self.buffer.has_selection() {
            self.buffer.start_selection();
        }
        self.buffer.move_left();
    }

    pub fn select_right(&mut self) {
        if !self.buffer.has_selection() {
            self.buffer.start_selection();
        }
        self.buffer.move_right();
    }

    pub fn select_up(&mut self) {
        if !self.buffer.has_selection() {
            self.buffer.start_selection();
        }
        self.buffer.move_up();
        self.adjust_scroll();
    }

    pub fn select_down(&mut self) {
        if !self.buffer.has_selection() {
            self.buffer.start_selection();
        }
        self.buffer.move_down();
        self.adjust_scroll();
    }

    pub fn select_all(&mut self) {
        self.buffer.select_all();
    }

    pub fn copy(&mut self) {
        if let Some(text) = self.buffer.get_selection() {
            self.clipboard = text;
            self.status_message = Some("Copied to clipboard".to_string());
        }
    }

    pub fn cut(&mut self) {
        if self.buffer.has_selection() {
            if let Some(text) = self.buffer.get_selection() {
                self.clipboard = text;
                self.save_state();
                self.buffer.delete_selection();
                self.status_message = Some("Cut to clipboard".to_string());
                self.update_suggestions();
            }
        }
    }

    pub fn paste(&mut self) {
        if !self.clipboard.is_empty() {
            self.save_state();
            // Delete selection if any before pasting
            if self.buffer.has_selection() {
                self.buffer.delete_selection();
            }
            self.buffer.insert_str(&self.clipboard);
            self.status_message = Some("Pasted from clipboard".to_string());
            self.update_suggestions();
        }
    }

    // ========== Search & Replace ==========

    pub fn open_search(&mut self) {
        self.search.open_find();
    }

    pub fn open_replace(&mut self) {
        self.search.open_replace();
    }

    pub fn close_search(&mut self) {
        self.search.close();
    }

    pub fn search_add_char(&mut self, c: char) {
        self.search.add_char(c);
        self.update_search_matches();
    }

    pub fn search_backspace(&mut self) {
        self.search.backspace();
        self.update_search_matches();
    }

    pub fn update_search_matches(&mut self) {
        self.search.find_matches(self.buffer.lines());
        // Jump to first match if any
        if !self.search.matches.is_empty() {
            self.jump_to_current_match();
        }
    }

    pub fn search_next(&mut self) {
        self.search.next_match();
        self.jump_to_current_match();
    }

    pub fn search_prev(&mut self) {
        self.search.prev_match();
        self.jump_to_current_match();
    }

    fn jump_to_current_match(&mut self) {
        if let Some((line, col, _)) = self.search.current_match_position() {
            self.buffer.set_cursor(col, line);
            self.adjust_scroll();
        }
    }

    pub fn replace_current(&mut self) {
        if self.search.mode != SearchMode::Replace {
            return;
        }
        if let Some((line, start, end)) = self.search.current_match_position() {
            self.save_state();
            // Move cursor to match and replace
            self.buffer.set_cursor(start, line);
            let current_line = &mut self.buffer.lines()[line].clone();
            let before = &current_line[..start];
            let after = &current_line[end..];
            let new_line = format!("{}{}{}", before, self.search.replace_text, after);
            // Update the line directly
            self.buffer.replace_line(line, new_line);
            // Re-run search to update matches
            self.update_search_matches();
            self.status_message = Some("Replaced 1 occurrence".to_string());
        }
    }

    pub fn replace_all(&mut self) {
        if self.search.mode != SearchMode::Replace {
            return;
        }
        if self.search.matches.is_empty() {
            return;
        }

        self.save_state();
        let count = self.search.matches.len();

        // Replace from end to start to preserve positions
        let mut matches = self.search.matches.clone();
        matches.reverse();

        for m in matches {
            let line = &self.buffer.lines()[m.line].clone();
            let before = &line[..m.start];
            let after = &line[m.end..];
            let new_line = format!("{}{}{}", before, self.search.replace_text, after);
            self.buffer.replace_line(m.line, new_line);
        }

        self.update_search_matches();
        self.status_message = Some(format!("Replaced {} occurrences", count));
    }

    pub fn toggle_search_case(&mut self) {
        self.search.toggle_case_sensitivity();
        self.update_search_matches();
    }

    pub fn accept_suggestion(&mut self) {
        // Prefer API suggestion if available, otherwise local
        if let Some(suggestion) = self.api_suggestion.take() {
            self.save_state();
            self.buffer.insert_str(&suggestion.text);
            self.update_suggestions();
        } else if let Some(suggestion) = self.current_suggestion.take() {
            self.save_state();
            self.buffer.insert_str(&suggestion.text);
            self.update_suggestions();
        }
    }

    pub fn accept_sentence_suggestion(&mut self) {
        if let Some(suggestion) = self.sentence_suggestion.take() {
            self.save_state();
            self.buffer.insert_str(&suggestion.text);
            self.update_suggestions();
        }
    }

    pub fn dismiss_or_exit(&mut self) {
        if self.show_synonyms {
            self.show_synonyms = false;
            self.synonyms.clear();
        } else if self.show_help {
            self.show_help = false;
        } else {
            self.current_suggestion = None;
            self.sentence_suggestion = None;
            self.api_suggestion = None;
            self.command_preview = None;
        }
    }

    pub fn toggle_synonym_selector(&mut self) {
        if self.show_synonyms {
            self.show_synonyms = false;
            self.synonyms.clear();
            return;
        }

        if let Some((word, start, end)) = self.buffer.word_at_cursor() {
            let syns = get_synonyms(&word);
            if !syns.is_empty() && syns[0] != word {
                self.synonyms = syns;
                self.synonym_word = Some(word);
                self.synonym_range = Some((start, end));
                self.synonym_index = 0;
                self.show_synonyms = true;
            }
        }
    }

    pub fn synonym_up(&mut self) {
        if !self.synonyms.is_empty() && self.synonym_index > 0 {
            self.synonym_index -= 1;
        }
    }

    pub fn synonym_down(&mut self) {
        if !self.synonyms.is_empty() && self.synonym_index < self.synonyms.len() - 1 {
            self.synonym_index += 1;
        }
    }

    fn select_synonym(&mut self) {
        if let Some((start, end)) = self.synonym_range {
            if let Some(syn) = self.synonyms.get(self.synonym_index).cloned() {
                self.save_state();
                self.buffer.replace_word(start, end, &syn);
            }
        }
        self.show_synonyms = false;
        self.synonyms.clear();
        self.synonym_range = None;
        self.synonym_word = None;
        self.update_suggestions();
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_emoji_mode(&mut self) {
        self.emoji_mode = !self.emoji_mode;
        self.status_message = Some(if self.emoji_mode {
            "Emoji mode ON - suggestions include emojis".to_string()
        } else {
            "Emoji mode OFF".to_string()
        });
        self.update_suggestions();
    }

    pub fn fetch_api_suggestion(&mut self) {
        // If using Local provider, just update local suggestions
        if self.config.ai_provider == AiProvider::Local {
            self.update_suggestions();
            self.status_message = Some("Using local suggestions".to_string());
            return;
        }

        if !self.config.has_api_key() {
            let provider = self.config.ai_provider;
            self.status_message = Some(format!("No {} API key - set env var", provider));
            return;
        }

        if self.api_loading {
            return;
        }

        let context = self.buffer.text_before_cursor();
        if context.len() < 10 {
            self.status_message = Some("Need more context for API suggestion".to_string());
            return;
        }

        self.api_loading = true;
        let model = self.config.current_model_display();
        self.status_message = Some(format!("Asking {}...", model));

        let config = self.config.clone();
        let tx = self.api_tx.clone();
        let ctx = context.clone();

        tokio::spawn(async move {
            let result = match config.ai_provider {
                AiProvider::Local => None, // Handled above
                AiProvider::OpenAI => {
                    let client = OpenAIClient::new(config);
                    client.suggest(&ctx).await
                }
                AiProvider::Anthropic => {
                    let client = AnthropicClient::new(config);
                    client.suggest(&ctx).await
                }
            };
            let _ = tx.send(ApiResponse::WordSuggestion(result)).await;
        });
    }

    pub fn cycle_ai_provider(&mut self) {
        self.config.cycle_provider();
        let model = self.config.current_model_display();
        let has_key = self.config.has_api_key();
        self.status_message = Some(format!(
            "AI: {} {}",
            model,
            if !has_key { "(no key)" } else { "" }
        ));
    }

    pub fn cycle_ai_model(&mut self) {
        self.config.cycle_model();
        let model = self.config.current_model_display();
        self.status_message = Some(format!("Model: {}", model));
    }

    pub fn cycle_ai_mode(&mut self) {
        self.config.cycle_mode();
        self.status_message = Some(format!("AI Mode: {}", self.config.ai_mode));
    }

    pub fn toggle_auto_suggest(&mut self) {
        self.config.toggle_auto_suggest();
        self.status_message = Some(format!(
            "Auto-suggest: {}",
            if self.config.auto_suggest { "ON" } else { "OFF" }
        ));
    }

    pub fn current_ai_provider(&self) -> &str {
        match self.config.ai_provider {
            AiProvider::Local => "Local",
            AiProvider::OpenAI => "GPT",
            AiProvider::Anthropic => "Claude",
        }
    }

    pub fn ai_status_text(&self) -> String {
        let provider = match self.config.ai_provider {
            AiProvider::Local => "Local",
            AiProvider::OpenAI => "GPT",
            AiProvider::Anthropic => "Claude",
        };
        let mode = match self.config.ai_mode {
            AiMode::Off => "Off",
            AiMode::LocalOnly => "L",
            AiMode::ApiOnly => "A",
            AiMode::Hybrid => "H",
        };
        format!("[{}|{}]", provider, mode)
    }

    pub fn handle_api_response(&mut self, response: ApiResponse) {
        self.api_loading = false;
        match response {
            ApiResponse::WordSuggestion(Some(suggestion)) => {
                self.api_suggestion = Some(suggestion);
                self.status_message = Some("AI suggestion ready (Tab to accept)".to_string());
            }
            ApiResponse::WordSuggestion(None) => {
                self.status_message = Some("No AI suggestion available".to_string());
            }
            ApiResponse::SentenceSuggestion(Some(suggestion)) => {
                self.sentence_suggestion = Some(suggestion);
                self.status_message = Some("AI sentence ready (Ctrl+Space to accept)".to_string());
            }
            ApiResponse::SentenceSuggestion(None) => {
                self.status_message = Some("No AI sentence available".to_string());
            }
        }
    }

    pub fn open_file_dialog(&mut self) {
        // Simple file open - in a real app this would be a dialog
        // For now, we'll just load a file if one exists
        if let Ok(content) = fs::read_to_string("untitled.txt") {
            self.buffer = TextBuffer::from_text(&content);
            self.file_path = Some(PathBuf::from("untitled.txt"));
            self.undo_stack.clear();
            self.redo_stack.clear();
            self.status_message = Some("File loaded: untitled.txt".to_string());
        }
    }

    pub fn save_file(&mut self) {
        let path = self.file_path.clone().unwrap_or_else(|| PathBuf::from("untitled.txt"));
        let content = self.buffer.to_string();
        if fs::write(&path, content).is_ok() {
            self.file_path = Some(path.clone());
            self.status_message = Some(format!("Saved: {}", path.display()));
        } else {
            self.status_message = Some("Failed to save file".to_string());
        }
    }

    pub fn word_count(&self) -> usize {
        self.buffer.to_string().split_whitespace().count()
    }

    pub fn char_count(&self) -> usize {
        self.buffer.to_string().chars().count()
    }

    pub fn line_count(&self) -> usize {
        self.buffer.lines().len()
    }

    pub fn set_visible_height(&mut self, height: usize) {
        let (_, cursor_y) = self.buffer.cursor();
        if cursor_y >= self.scroll_offset + height.saturating_sub(2) {
            self.scroll_offset = cursor_y.saturating_sub(height.saturating_sub(3));
        }
    }

    pub fn get_emoji_engine(&self) -> &EmojiEngine {
        &self.emoji
    }
}
