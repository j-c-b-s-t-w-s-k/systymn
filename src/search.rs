/// Search and Replace functionality

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Find,
    Replace,
}

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub replace_text: String,
    pub matches: Vec<SearchMatch>,
    pub current_match: usize,
    pub case_sensitive: bool,
    pub is_active: bool,
    pub mode: SearchMode,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            replace_text: String::new(),
            matches: Vec::new(),
            current_match: 0,
            case_sensitive: false,
            is_active: false,
            mode: SearchMode::Find,
        }
    }

    pub fn open_find(&mut self) {
        self.is_active = true;
        self.mode = SearchMode::Find;
        self.query.clear();
        self.replace_text.clear();
        self.matches.clear();
        self.current_match = 0;
    }

    pub fn open_replace(&mut self) {
        self.is_active = true;
        self.mode = SearchMode::Replace;
        self.query.clear();
        self.replace_text.clear();
        self.matches.clear();
        self.current_match = 0;
    }

    pub fn close(&mut self) {
        self.is_active = false;
        self.matches.clear();
    }

    pub fn add_char(&mut self, c: char) {
        self.query.push(c);
    }

    pub fn add_replace_char(&mut self, c: char) {
        self.replace_text.push(c);
    }

    pub fn backspace(&mut self) {
        self.query.pop();
    }

    pub fn backspace_replace(&mut self) {
        self.replace_text.pop();
    }

    /// Find all matches in the given lines
    pub fn find_matches(&mut self, lines: &[String]) {
        self.matches.clear();
        self.current_match = 0;

        if self.query.is_empty() {
            return;
        }

        let query = if self.case_sensitive {
            self.query.clone()
        } else {
            self.query.to_lowercase()
        };

        for (line_idx, line) in lines.iter().enumerate() {
            let search_line = if self.case_sensitive {
                line.clone()
            } else {
                line.to_lowercase()
            };

            let mut start = 0;
            while let Some(pos) = search_line[start..].find(&query) {
                let match_start = start + pos;
                let match_end = match_start + self.query.len();
                self.matches.push(SearchMatch {
                    line: line_idx,
                    start: match_start,
                    end: match_end,
                });
                start = match_end;
            }
        }
    }

    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.matches.len();
        }
    }

    pub fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = if self.current_match == 0 {
                self.matches.len() - 1
            } else {
                self.current_match - 1
            };
        }
    }

    pub fn current_match_position(&self) -> Option<(usize, usize, usize)> {
        self.matches.get(self.current_match).map(|m| (m.line, m.start, m.end))
    }

    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    pub fn current_match_index(&self) -> usize {
        if self.matches.is_empty() {
            0
        } else {
            self.current_match + 1
        }
    }

    pub fn toggle_case_sensitivity(&mut self) {
        self.case_sensitive = !self.case_sensitive;
    }
}
