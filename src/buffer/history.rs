/// Represents a single edit operation for undo/redo
#[derive(Debug, Clone)]
pub enum EditOperation {
    /// Insert text at position (byte offset in line, line index)
    Insert {
        line: usize,
        col: usize,
        text: String,
    },
    /// Delete text at position
    Delete {
        line: usize,
        col: usize,
        text: String,
    },
    /// Insert a newline, splitting the line
    InsertNewline {
        line: usize,
        col: usize,
    },
    /// Delete a newline, joining lines
    DeleteNewline {
        line: usize,
    },
    /// Batch of operations (for replace, paste, etc.)
    Batch(Vec<EditOperation>),
}

impl EditOperation {
    /// Returns the inverse operation for undo
    pub fn inverse(&self) -> EditOperation {
        match self {
            EditOperation::Insert { line, col, text } => EditOperation::Delete {
                line: *line,
                col: *col,
                text: text.clone(),
            },
            EditOperation::Delete { line, col, text } => EditOperation::Insert {
                line: *line,
                col: *col,
                text: text.clone(),
            },
            EditOperation::InsertNewline { line, col } => EditOperation::DeleteNewline {
                line: *line,
            },
            EditOperation::DeleteNewline { line } => EditOperation::InsertNewline {
                line: *line,
                col: 0, // Will be set correctly during apply
            },
            EditOperation::Batch(ops) => {
                EditOperation::Batch(ops.iter().rev().map(|op| op.inverse()).collect())
            }
        }
    }
}

/// Manages undo/redo history
pub struct History {
    undo_stack: Vec<EditOperation>,
    redo_stack: Vec<EditOperation>,
    max_size: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size: 1000,
        }
    }

    /// Record an operation for undo
    pub fn push(&mut self, op: EditOperation) {
        self.undo_stack.push(op);
        self.redo_stack.clear(); // Clear redo stack on new edit

        // Trim if exceeds max size
        if self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
    }

    /// Pop operation for undo, returns the inverse to apply
    pub fn undo(&mut self) -> Option<EditOperation> {
        if let Some(op) = self.undo_stack.pop() {
            let inverse = op.inverse();
            self.redo_stack.push(op);
            Some(inverse)
        } else {
            None
        }
    }

    /// Pop operation for redo, returns the operation to apply
    pub fn redo(&mut self) -> Option<EditOperation> {
        if let Some(op) = self.redo_stack.pop() {
            self.undo_stack.push(op.clone());
            Some(op)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
