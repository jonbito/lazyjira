//! Text input component.

/// A text input widget.
pub struct Input {
    /// The current input value.
    pub value: String,
    /// Cursor position.
    pub cursor: usize,
}

impl Input {
    /// Create a new empty input.
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
