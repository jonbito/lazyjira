//! Text input component.
//!
//! This module provides a text input widget with support for:
//! - Character input and deletion
//! - Cursor movement (left/right, home/end)
//! - Password masking for sensitive fields
//! - Visual focus indication
//!
//! ## Input Mode
//!
//! The `InputMode` enum indicates whether a component is in navigation
//! mode (where keys like j/k control movement) or insert mode (where
//! all character keys are sent to the text input).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Input mode for components that support both navigation and text input.
///
/// This enum helps standardize behavior across the application:
/// - In `Normal` mode, keys like j/k can be used for navigation
/// - In `Insert` mode, all character keys go to the focused text input
///
/// Components should switch to `Insert` mode when a text field gains focus,
/// and back to `Normal` mode when the user presses Esc or the field loses focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal/navigation mode - j/k/q and other keys work as shortcuts.
    #[default]
    Normal,
    /// Insert/editing mode - all character keys go to text input.
    Insert,
}
use ratatui::{
    layout::{Position, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// A text input widget.
#[derive(Debug, Clone)]
pub struct TextInput {
    /// The current input value.
    value: String,
    /// Cursor position within the value.
    cursor: usize,
    /// Whether to mask the input (for passwords).
    masked: bool,
    /// Placeholder text shown when empty.
    placeholder: String,
}

impl TextInput {
    /// Create a new empty input.
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            masked: false,
            placeholder: String::new(),
        }
    }

    /// Create a new masked input (for passwords/tokens).
    pub fn masked() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            masked: true,
            placeholder: String::new(),
        }
    }

    /// Create a new input with an initial value.
    pub fn with_value(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.len();
        Self {
            value,
            cursor,
            masked: false,
            placeholder: String::new(),
        }
    }

    /// Set whether the input is masked.
    pub fn set_masked(&mut self, masked: bool) {
        self.masked = masked;
    }

    /// Set the placeholder text.
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Get the current value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set the value and move cursor to end.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.len();
    }

    /// Clear the input.
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    /// Check if the input is empty.
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Get the cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Get the display value (masked if needed).
    pub fn display_value(&self) -> String {
        if self.masked {
            "•".repeat(self.value.len())
        } else {
            self.value.clone()
        }
    }

    /// Handle keyboard input.
    ///
    /// Returns true if the input was modified.
    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        match (key.code, key.modifiers) {
            // Character input
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.insert_char(c);
                true
            }
            // Backspace - delete character before cursor
            (KeyCode::Backspace, _) => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.value.remove(self.cursor);
                    true
                } else {
                    false
                }
            }
            // Delete - delete character at cursor
            (KeyCode::Delete, _) => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                    true
                } else {
                    false
                }
            }
            // Left arrow - move cursor left
            (KeyCode::Left, KeyModifiers::NONE) => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                false
            }
            // Right arrow - move cursor right
            (KeyCode::Right, KeyModifiers::NONE) => {
                if self.cursor < self.value.len() {
                    self.cursor += 1;
                }
                false
            }
            // Home - move to beginning
            (KeyCode::Home, _) => {
                self.cursor = 0;
                false
            }
            // End - move to end
            (KeyCode::End, _) => {
                self.cursor = self.value.len();
                false
            }
            // Ctrl+A - select all (move to start, then we could implement selection)
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                self.cursor = 0;
                false
            }
            // Ctrl+E - move to end
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                self.cursor = self.value.len();
                false
            }
            // Ctrl+U - clear line
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                if !self.value.is_empty() {
                    self.value.clear();
                    self.cursor = 0;
                    true
                } else {
                    false
                }
            }
            // Ctrl+W - delete word before cursor
            (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                if self.cursor > 0 {
                    let before = &self.value[..self.cursor];
                    // Find the start of the previous word
                    let word_start = before
                        .rfind(|c: char| !c.is_alphanumeric())
                        .map(|i| i + 1)
                        .unwrap_or(0);
                    self.value.replace_range(word_start..self.cursor, "");
                    self.cursor = word_start;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Insert a character at the cursor position.
    fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Render the input field.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `area` - The area to render in
    /// * `focused` - Whether this input is currently focused
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let display = if self.value.is_empty() && !self.placeholder.is_empty() {
            self.placeholder.clone()
        } else {
            self.display_value()
        };

        let style = if focused {
            Style::default().fg(Color::Yellow)
        } else if self.value.is_empty() && !self.placeholder.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let input = Paragraph::new(display).style(style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_widget(input, area);

        // Show cursor if focused
        if focused {
            // Calculate cursor position accounting for the border
            let cursor_x = area.x + 1 + self.cursor as u16;
            let cursor_y = area.y + 1;

            // Only show cursor if it's within the visible area
            if cursor_x < area.x + area.width - 1 {
                frame.set_cursor_position(Position::new(cursor_x, cursor_y));
            }
        }
    }

    /// Render the input field with a label.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `area` - The area to render in
    /// * `label` - The label to display
    /// * `focused` - Whether this input is currently focused
    pub fn render_with_label(&self, frame: &mut Frame, area: Rect, label: &str, focused: bool) {
        let display = if self.value.is_empty() && !self.placeholder.is_empty() {
            self.placeholder.clone()
        } else {
            self.display_value()
        };

        let style = if focused {
            Style::default().fg(Color::Yellow)
        } else if self.value.is_empty() && !self.placeholder.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let title_style = if focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .title(ratatui::text::Span::styled(
                format!(" {} ", label),
                title_style,
            ))
            .borders(Borders::ALL)
            .border_style(border_style);

        let input = Paragraph::new(display).style(style).block(block);

        frame.render_widget(input, area);

        // Show cursor if focused
        if focused {
            let cursor_x = area.x + 1 + self.cursor as u16;
            let cursor_y = area.y + 1;

            if cursor_x < area.x + area.width - 1 {
                frame.set_cursor_position(Position::new(cursor_x, cursor_y));
            }
        }
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_input() {
        let input = TextInput::new();
        assert!(input.is_empty());
        assert_eq!(input.cursor(), 0);
        assert_eq!(input.value(), "");
    }

    #[test]
    fn test_masked_input() {
        let mut input = TextInput::masked();
        input.set_value("secret");
        assert_eq!(input.value(), "secret");
        assert_eq!(input.display_value(), "••••••");
    }

    #[test]
    fn test_with_value() {
        let input = TextInput::with_value("hello");
        assert_eq!(input.value(), "hello");
        assert_eq!(input.cursor(), 5);
    }

    #[test]
    fn test_set_value() {
        let mut input = TextInput::new();
        input.set_value("test");
        assert_eq!(input.value(), "test");
        assert_eq!(input.cursor(), 4);
    }

    #[test]
    fn test_clear() {
        let mut input = TextInput::with_value("hello");
        input.clear();
        assert!(input.is_empty());
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_character_input() {
        let mut input = TextInput::new();

        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(input.handle_input(key));
        assert_eq!(input.value(), "a");
        assert_eq!(input.cursor(), 1);

        let key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        input.handle_input(key);
        assert_eq!(input.value(), "ab");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn test_backspace() {
        let mut input = TextInput::with_value("abc");

        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(input.handle_input(key));
        assert_eq!(input.value(), "ab");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn test_backspace_at_start() {
        let mut input = TextInput::new();
        input.set_value("abc");
        input.cursor = 0;

        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(!input.handle_input(key));
        assert_eq!(input.value(), "abc");
    }

    #[test]
    fn test_delete() {
        let mut input = TextInput::with_value("abc");
        input.cursor = 0;

        let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(input.handle_input(key));
        assert_eq!(input.value(), "bc");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_delete_at_end() {
        let mut input = TextInput::with_value("abc");

        let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(!input.handle_input(key));
        assert_eq!(input.value(), "abc");
    }

    #[test]
    fn test_cursor_left() {
        let mut input = TextInput::with_value("abc");
        assert_eq!(input.cursor(), 3);

        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        input.handle_input(key);
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn test_cursor_left_at_start() {
        let mut input = TextInput::with_value("abc");
        input.cursor = 0;

        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        input.handle_input(key);
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_cursor_right() {
        let mut input = TextInput::with_value("abc");
        input.cursor = 0;

        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        input.handle_input(key);
        assert_eq!(input.cursor(), 1);
    }

    #[test]
    fn test_cursor_right_at_end() {
        let mut input = TextInput::with_value("abc");

        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        input.handle_input(key);
        assert_eq!(input.cursor(), 3);
    }

    #[test]
    fn test_home_key() {
        let mut input = TextInput::with_value("abc");

        let key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        input.handle_input(key);
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_end_key() {
        let mut input = TextInput::with_value("abc");
        input.cursor = 0;

        let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
        input.handle_input(key);
        assert_eq!(input.cursor(), 3);
    }

    #[test]
    fn test_ctrl_u_clear() {
        let mut input = TextInput::with_value("hello");

        let key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        assert!(input.handle_input(key));
        assert!(input.is_empty());
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_ctrl_u_empty() {
        let mut input = TextInput::new();

        let key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        assert!(!input.handle_input(key));
    }

    #[test]
    fn test_ctrl_w_delete_word() {
        let mut input = TextInput::with_value("hello world");

        let key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert!(input.handle_input(key));
        assert_eq!(input.value(), "hello ");
    }

    #[test]
    fn test_insert_in_middle() {
        let mut input = TextInput::with_value("ac");
        input.cursor = 1;

        let key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        input.handle_input(key);
        assert_eq!(input.value(), "abc");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn test_placeholder() {
        let mut input = TextInput::new();
        input.set_placeholder("Enter text...");

        // Placeholder should be used for display when empty
        assert!(input.is_empty());
    }

    #[test]
    fn test_display_value_not_masked() {
        let input = TextInput::with_value("hello");
        assert_eq!(input.display_value(), "hello");
    }

    #[test]
    fn test_display_value_masked() {
        let mut input = TextInput::new();
        input.set_masked(true);
        input.set_value("secret");
        assert_eq!(input.display_value(), "••••••");
    }
}
