use unicode_width::UnicodeWidthStr;
use super::history::EditOperation;

#[derive(Debug, Clone)]
pub struct TextBuffer {
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    selection_anchor: Option<(usize, usize)>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            selection_anchor: None,
        }
    }

    pub fn from_text(text: &str) -> Self {
        let lines: Vec<String> = text.lines().map(String::from).collect();
        Self {
            lines: if lines.is_empty() { vec![String::new()] } else { lines },
            cursor_x: 0,
            cursor_y: 0,
            selection_anchor: None,
        }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn current_line(&self) -> &str {
        &self.lines[self.cursor_y]
    }

    pub fn current_line_mut(&mut self) -> &mut String {
        &mut self.lines[self.cursor_y]
    }

    // Selection methods
    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub fn start_selection(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_x, self.cursor_y));
        }
    }

    pub fn has_selection(&self) -> bool {
        if let Some((ax, ay)) = self.selection_anchor {
            ax != self.cursor_x || ay != self.cursor_y
        } else {
            false
        }
    }

    pub fn select_all(&mut self) {
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.selection_anchor = Some((0, 0));
        self.cursor_y = self.lines.len().saturating_sub(1);
        self.cursor_x = self.lines[self.cursor_y].len();
    }

    pub fn get_selection(&self) -> Option<String> {
        let (ax, ay) = self.selection_anchor?;
        if !self.has_selection() {
            return None;
        }

        let (start_x, start_y, end_x, end_y) = if (ay, ax) <= (self.cursor_y, self.cursor_x) {
            (ax, ay, self.cursor_x, self.cursor_y)
        } else {
            (self.cursor_x, self.cursor_y, ax, ay)
        };

        let mut result = String::new();
        for y in start_y..=end_y {
            let line = &self.lines[y];
            let line_start = if y == start_y { start_x } else { 0 };
            let line_end = if y == end_y { end_x } else { line.len() };
            result.push_str(&line[line_start.min(line.len())..line_end.min(line.len())]);
            if y < end_y {
                result.push('\n');
            }
        }

        Some(result)
    }

    pub fn delete_selection(&mut self) {
        if let Some((ax, ay)) = self.selection_anchor.take() {
            if !self.has_selection() {
                return;
            }

            let (start_x, start_y, end_x, end_y) = if (ay, ax) <= (self.cursor_y, self.cursor_x) {
                (ax, ay, self.cursor_x, self.cursor_y)
            } else {
                (self.cursor_x, self.cursor_y, ax, ay)
            };

            if start_y == end_y {
                let line = &mut self.lines[start_y];
                let before = line[..start_x.min(line.len())].to_string();
                let after = line[end_x.min(line.len())..].to_string();
                *line = before + &after;
            } else {
                let before = self.lines[start_y][..start_x.min(self.lines[start_y].len())].to_string();
                let after = self.lines[end_y][end_x.min(self.lines[end_y].len())..].to_string();
                self.lines[start_y] = before + &after;
                for _ in start_y + 1..=end_y {
                    if start_y + 1 < self.lines.len() {
                        self.lines.remove(start_y + 1);
                    }
                }
            }

            self.cursor_x = start_x;
            self.cursor_y = start_y;
            self.selection_anchor = None;
        }
    }

    pub fn get_selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let (ax, ay) = self.selection_anchor?;
        if !self.has_selection() {
            return None;
        }

        if (ay, ax) <= (self.cursor_y, self.cursor_x) {
            Some(((ax, ay), (self.cursor_x, self.cursor_y)))
        } else {
            Some(((self.cursor_x, self.cursor_y), (ax, ay)))
        }
    }

    pub fn insert_char(&mut self, c: char) -> EditOperation {
        self.clear_selection();
        let line = &mut self.lines[self.cursor_y];
        let col = self.cursor_x;
        let line_idx = self.cursor_y;
        if self.cursor_x >= line.len() {
            line.push(c);
        } else {
            line.insert(self.cursor_x, c);
        }
        self.cursor_x += c.len_utf8();
        EditOperation::Insert {
            line: line_idx,
            col,
            text: c.to_string(),
        }
    }

    pub fn insert_str(&mut self, s: &str) -> Vec<EditOperation> {
        let mut ops = Vec::new();
        for c in s.chars() {
            if c == '\n' {
                ops.push(self.insert_newline());
            } else {
                ops.push(self.insert_char(c));
            }
        }
        ops
    }

    pub fn insert_newline(&mut self) -> EditOperation {
        self.clear_selection();
        let col = self.cursor_x;
        let line_idx = self.cursor_y;
        let line = &mut self.lines[self.cursor_y];
        let remainder = line[self.cursor_x..].to_string();
        line.truncate(self.cursor_x);
        self.cursor_y += 1;
        self.lines.insert(self.cursor_y, remainder);
        self.cursor_x = 0;
        EditOperation::InsertNewline { line: line_idx, col }
    }

    pub fn backspace(&mut self) -> Option<EditOperation> {
        self.clear_selection();
        if self.cursor_x > 0 {
            let line = &mut self.lines[self.cursor_y];
            let mut chars: Vec<char> = line.chars().collect();
            let char_idx = line[..self.cursor_x].chars().count() - 1;
            let removed_char = chars.remove(char_idx);
            let col = self.cursor_x - removed_char.len_utf8();
            *line = chars.into_iter().collect();
            self.cursor_x = col;
            Some(EditOperation::Delete {
                line: self.cursor_y,
                col,
                text: removed_char.to_string(),
            })
        } else if self.cursor_y > 0 {
            let current_line = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
            self.lines[self.cursor_y].push_str(&current_line);
            Some(EditOperation::DeleteNewline { line: self.cursor_y })
        } else {
            None
        }
    }

    pub fn delete(&mut self) -> Option<EditOperation> {
        self.clear_selection();
        let line = &self.lines[self.cursor_y];
        if self.cursor_x < line.len() {
            let line = &mut self.lines[self.cursor_y];
            let mut chars: Vec<char> = line.chars().collect();
            let char_idx = line[..self.cursor_x].chars().count();
            if char_idx < chars.len() {
                let removed_char = chars.remove(char_idx);
                *line = chars.into_iter().collect();
                Some(EditOperation::Delete {
                    line: self.cursor_y,
                    col: self.cursor_x,
                    text: removed_char.to_string(),
                })
            } else {
                None
            }
        } else if self.cursor_y < self.lines.len() - 1 {
            let next_line = self.lines.remove(self.cursor_y + 1);
            self.lines[self.cursor_y].push_str(&next_line);
            Some(EditOperation::DeleteNewline { line: self.cursor_y })
        } else {
            None
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_x > 0 {
            let line = &self.lines[self.cursor_y];
            let chars_before: Vec<char> = line[..self.cursor_x].chars().collect();
            if let Some(last_char) = chars_before.last() {
                self.cursor_x -= last_char.len_utf8();
            }
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
        }
    }

    pub fn move_right(&mut self) {
        let line = &self.lines[self.cursor_y];
        if self.cursor_x < line.len() {
            if let Some(c) = line[self.cursor_x..].chars().next() {
                self.cursor_x += c.len_utf8();
            }
        } else if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.cursor_x.min(self.lines[self.cursor_y].len());
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = self.cursor_x.min(self.lines[self.cursor_y].len());
        }
    }

    pub fn move_home(&mut self) {
        self.cursor_x = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor_x = self.lines[self.cursor_y].len();
    }

    pub fn move_to_start(&mut self) {
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    pub fn move_to_end(&mut self) {
        self.cursor_y = self.lines.len().saturating_sub(1);
        self.cursor_x = self.lines[self.cursor_y].len();
    }

    pub fn word_at_cursor(&self) -> Option<(String, usize, usize)> {
        let line = &self.lines[self.cursor_y];
        if line.is_empty() {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        let char_pos = line[..self.cursor_x.min(line.len())].chars().count();

        if char_pos == 0 {
            return None;
        }

        let mut start = char_pos.saturating_sub(1);
        while start > 0 && chars[start - 1].is_alphanumeric() {
            start -= 1;
        }

        let mut end = char_pos.saturating_sub(1);
        while end < chars.len() && chars[end].is_alphanumeric() {
            end += 1;
        }

        if start < end && chars[start].is_alphanumeric() {
            let word: String = chars[start..end].iter().collect();
            let byte_start: usize = chars[..start].iter().map(|c| c.len_utf8()).sum();
            let byte_end: usize = chars[..end].iter().map(|c| c.len_utf8()).sum();
            Some((word, byte_start, byte_end))
        } else {
            None
        }
    }

    pub fn replace_word(&mut self, start: usize, end: usize, replacement: &str) {
        let line = &mut self.lines[self.cursor_y];
        let before = &line[..start];
        let after = &line[end..];
        let new_line = format!("{}{}{}", before, replacement, after);
        let new_cursor = start + replacement.len();
        *line = new_line;
        self.cursor_x = new_cursor;
    }

    pub fn text_before_cursor(&self) -> String {
        let mut result = String::new();
        for (i, line) in self.lines.iter().enumerate() {
            if i < self.cursor_y {
                result.push_str(line);
                result.push('\n');
            } else if i == self.cursor_y {
                result.push_str(&line[..self.cursor_x]);
            }
        }
        result
    }

    pub fn last_word(&self) -> Option<String> {
        let text = self.text_before_cursor();
        text.split_whitespace().last().map(String::from)
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    pub fn visual_cursor_x(&self) -> usize {
        let line = &self.lines[self.cursor_y];
        line[..self.cursor_x].width()
    }

    pub fn wrap_line(line: &str, width: usize) -> Vec<String> {
        if width == 0 || line.is_empty() {
            return vec![line.to_string()];
        }

        let mut wrapped = Vec::new();
        let mut current = String::new();
        let mut current_width = 0;

        for word in line.split_inclusive(|c: char| c.is_whitespace()) {
            let word_width = word.width();

            if current_width + word_width > width && !current.is_empty() {
                wrapped.push(current);
                current = String::new();
                current_width = 0;
            }

            current.push_str(word);
            current_width += word_width;
        }

        if !current.is_empty() {
            wrapped.push(current);
        }

        if wrapped.is_empty() {
            wrapped.push(String::new());
        }

        wrapped
    }

    pub fn get_wrapped_lines(&self, width: usize) -> Vec<(usize, String)> {
        let mut result = Vec::new();
        for (line_idx, line) in self.lines.iter().enumerate() {
            let wrapped = Self::wrap_line(line, width);
            for segment in wrapped {
                result.push((line_idx, segment));
            }
        }
        result
    }

    pub fn wrapped_cursor_position(&self, width: usize) -> (usize, usize) {
        if width == 0 {
            return (self.visual_cursor_x(), self.cursor_y);
        }

        let mut visual_y = 0;
        for (line_idx, line) in self.lines.iter().enumerate() {
            if line_idx < self.cursor_y {
                let wrapped = Self::wrap_line(line, width);
                visual_y += wrapped.len();
            } else if line_idx == self.cursor_y {
                let before_cursor = &line[..self.cursor_x];
                let wrapped_before = Self::wrap_line(before_cursor, width);
                visual_y += wrapped_before.len().saturating_sub(1);
                let last_segment = wrapped_before.last().map(|s| s.width()).unwrap_or(0);
                return (last_segment, visual_y);
            }
        }

        (self.visual_cursor_x(), visual_y)
    }

    /// Set cursor position directly
    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.cursor_y = y.min(self.lines.len().saturating_sub(1));
        self.cursor_x = x.min(self.lines[self.cursor_y].len());
        self.selection_anchor = None;
    }

    /// Replace an entire line
    pub fn replace_line(&mut self, line_idx: usize, new_content: String) {
        if line_idx < self.lines.len() {
            self.lines[line_idx] = new_content;
        }
    }

    /// Get mutable access to lines for search/replace
    pub fn lines_mut(&mut self) -> &mut Vec<String> {
        &mut self.lines
    }
}
