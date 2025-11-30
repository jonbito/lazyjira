//! Multi-line text editor component.
//!
//! This module provides a multi-line text editor widget with support for:
//! - Multi-line text editing
//! - Cursor movement (arrows, home/end, page up/down)
//! - Scrolling for content longer than visible area
//! - Change tracking for unsaved changes indicator

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Position, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// A multi-line text editor component.
#[derive(Debug, Clone)]
pub struct TextEditor {
    /// Lines of text content.
    lines: Vec<String>,
    /// Current line (0-indexed).
    cursor_line: usize,
    /// Current column within the line.
    cursor_col: usize,
    /// Scroll offset (first visible line).
    scroll: usize,
    /// Original content for change detection.
    original_content: String,
}

impl TextEditor {
    /// Create a new text editor with the given content.
    pub fn new(content: &str) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(String::from).collect()
        };
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };

        Self {
            lines,
            cursor_line: 0,
            cursor_col: 0,
            scroll: 0,
            original_content: content.to_string(),
        }
    }

    /// Create an empty text editor.
    pub fn empty() -> Self {
        Self::new("")
    }

    /// Get the current content as a string.
    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    /// Check if the content has been modified.
    pub fn has_changes(&self) -> bool {
        self.content() != self.original_content
    }

    /// Get the current cursor line.
    pub fn cursor_line(&self) -> usize {
        self.cursor_line
    }

    /// Get the current cursor column.
    pub fn cursor_col(&self) -> usize {
        self.cursor_col
    }

    /// Get the current scroll position.
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// Get the number of lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get the current line.
    fn current_line(&self) -> &String {
        &self.lines[self.cursor_line]
    }

    /// Get the current line mutably.
    fn current_line_mut(&mut self) -> &mut String {
        &mut self.lines[self.cursor_line]
    }

    /// Ensure the cursor column is within bounds for the current line.
    fn clamp_cursor_col(&mut self) {
        let line_len = self.current_line().len();
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }

    /// Ensure the scroll position keeps the cursor visible.
    fn ensure_cursor_visible(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        // Cursor above viewport
        if self.cursor_line < self.scroll {
            self.scroll = self.cursor_line;
        }
        // Cursor below viewport
        if self.cursor_line >= self.scroll + visible_height {
            self.scroll = self.cursor_line - visible_height + 1;
        }
    }

    /// Handle keyboard input.
    ///
    /// Returns true if the content was modified.
    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        match (key.code, key.modifiers) {
            // Character input
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.insert_char(c);
                true
            }
            // Enter - insert newline
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.insert_newline();
                true
            }
            // Backspace - delete character before cursor
            (KeyCode::Backspace, _) => self.delete_backward(),
            // Delete - delete character at cursor
            (KeyCode::Delete, _) => self.delete_forward(),
            // Arrow keys - movement
            (KeyCode::Left, KeyModifiers::NONE) => {
                self.move_left();
                false
            }
            (KeyCode::Right, KeyModifiers::NONE) => {
                self.move_right();
                false
            }
            (KeyCode::Up, KeyModifiers::NONE) => {
                self.move_up();
                false
            }
            (KeyCode::Down, KeyModifiers::NONE) => {
                self.move_down();
                false
            }
            // Home/End
            (KeyCode::Home, _) => {
                self.cursor_col = 0;
                false
            }
            (KeyCode::End, _) => {
                self.cursor_col = self.current_line().len();
                false
            }
            // Ctrl+A - beginning of line (emacs style)
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                self.cursor_col = 0;
                false
            }
            // Ctrl+E - end of line (emacs style)
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                self.cursor_col = self.current_line().len();
                false
            }
            // Ctrl+U - delete line content before cursor
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                if self.cursor_col > 0 {
                    let cursor = self.cursor_col;
                    self.lines[self.cursor_line].replace_range(..cursor, "");
                    self.cursor_col = 0;
                    true
                } else {
                    false
                }
            }
            // Ctrl+K - delete from cursor to end of line
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                let line_len = self.current_line().len();
                if self.cursor_col < line_len {
                    let cursor = self.cursor_col;
                    self.lines[self.cursor_line].truncate(cursor);
                    true
                } else if self.cursor_line < self.lines.len() - 1 {
                    // Join with next line
                    let next_line = self.lines.remove(self.cursor_line + 1);
                    self.lines[self.cursor_line].push_str(&next_line);
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
        let cursor = self.cursor_col;
        self.lines[self.cursor_line].insert(cursor, c);
        self.cursor_col += 1;
    }

    /// Insert a newline at the cursor position.
    fn insert_newline(&mut self) {
        let cursor = self.cursor_col;
        let line_idx = self.cursor_line;
        let new_line = self.lines[line_idx].split_off(cursor);
        self.lines.insert(line_idx + 1, new_line);
        self.cursor_line += 1;
        self.cursor_col = 0;
    }

    /// Delete the character before the cursor.
    fn delete_backward(&mut self) -> bool {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            let cursor = self.cursor_col;
            self.lines[self.cursor_line].remove(cursor);
            true
        } else if self.cursor_line > 0 {
            // Join with previous line
            let current_line = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
            self.lines[self.cursor_line].push_str(&current_line);
            true
        } else {
            false
        }
    }

    /// Delete the character at the cursor.
    fn delete_forward(&mut self) -> bool {
        let line_len = self.current_line().len();
        if self.cursor_col < line_len {
            let cursor = self.cursor_col;
            self.lines[self.cursor_line].remove(cursor);
            true
        } else if self.cursor_line < self.lines.len() - 1 {
            // Join with next line
            let next_line = self.lines.remove(self.cursor_line + 1);
            self.lines[self.cursor_line].push_str(&next_line);
            true
        } else {
            false
        }
    }

    /// Move cursor left.
    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.current_line().len();
        }
    }

    /// Move cursor right.
    fn move_right(&mut self) {
        let line_len = self.current_line().len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    /// Move cursor up.
    fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.clamp_cursor_col();
        }
    }

    /// Move cursor down.
    fn move_down(&mut self) {
        if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            self.clamp_cursor_col();
        }
    }

    /// Render the text editor.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `area` - The area to render in
    /// * `focused` - Whether this editor is focused
    /// * `title` - Optional title for the block
    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool, title: Option<&str>) {
        // Calculate visible area (accounting for borders)
        let visible_height = area.height.saturating_sub(2) as usize;

        // Ensure cursor is visible
        self.ensure_cursor_visible(visible_height);

        // Build lines to display
        let display_lines: Vec<Line> = self
            .lines
            .iter()
            .skip(self.scroll)
            .take(visible_height)
            .enumerate()
            .map(|(i, line)| {
                let line_num = self.scroll + i;
                if focused && line_num == self.cursor_line {
                    // Highlight current line
                    Line::from(line.as_str()).style(Style::default().bg(Color::DarkGray))
                } else {
                    Line::from(line.as_str())
                }
            })
            .collect();

        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = if let Some(title) = title {
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style)
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
        };

        let paragraph = Paragraph::new(display_lines).block(block);

        frame.render_widget(paragraph, area);

        // Render cursor if focused
        if focused {
            let cursor_x = area.x + 1 + self.cursor_col as u16;
            let cursor_y = area.y + 1 + (self.cursor_line - self.scroll) as u16;

            // Only show cursor if it's within the visible area
            if cursor_y < area.y + area.height - 1 && cursor_x < area.x + area.width - 1 {
                frame.set_cursor_position(Position::new(cursor_x, cursor_y));
            }
        }
    }

    /// Get scroll info for display (current line / total lines).
    pub fn scroll_info(&self) -> (usize, usize) {
        (self.cursor_line + 1, self.lines.len())
    }
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_editor() {
        let editor = TextEditor::new("hello\nworld");
        assert_eq!(editor.line_count(), 2);
        assert_eq!(editor.content(), "hello\nworld");
        assert_eq!(editor.cursor_line(), 0);
        assert_eq!(editor.cursor_col(), 0);
    }

    #[test]
    fn test_empty_editor() {
        let editor = TextEditor::empty();
        assert_eq!(editor.line_count(), 1);
        assert_eq!(editor.content(), "");
        assert!(!editor.has_changes());
    }

    #[test]
    fn test_insert_char() {
        let mut editor = TextEditor::empty();

        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(editor.handle_input(key));
        assert_eq!(editor.content(), "a");
        assert!(editor.has_changes());
    }

    #[test]
    fn test_insert_newline() {
        let mut editor = TextEditor::new("hello");
        editor.cursor_col = 5;

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(editor.handle_input(key));
        assert_eq!(editor.line_count(), 2);
        assert_eq!(editor.cursor_line(), 1);
        assert_eq!(editor.cursor_col(), 0);
    }

    #[test]
    fn test_insert_newline_in_middle() {
        let mut editor = TextEditor::new("helloworld");
        editor.cursor_col = 5;

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        editor.handle_input(key);
        assert_eq!(editor.content(), "hello\nworld");
    }

    #[test]
    fn test_backspace() {
        let mut editor = TextEditor::new("abc");
        editor.cursor_col = 3;

        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(editor.handle_input(key));
        assert_eq!(editor.content(), "ab");
        assert_eq!(editor.cursor_col(), 2);
    }

    #[test]
    fn test_backspace_at_line_start() {
        let mut editor = TextEditor::new("hello\nworld");
        editor.cursor_line = 1;
        editor.cursor_col = 0;

        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(editor.handle_input(key));
        assert_eq!(editor.content(), "helloworld");
        assert_eq!(editor.line_count(), 1);
        assert_eq!(editor.cursor_col(), 5);
    }

    #[test]
    fn test_backspace_at_start() {
        let mut editor = TextEditor::new("hello");

        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(!editor.handle_input(key));
        assert_eq!(editor.content(), "hello");
    }

    #[test]
    fn test_delete() {
        let mut editor = TextEditor::new("abc");
        editor.cursor_col = 0;

        let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(editor.handle_input(key));
        assert_eq!(editor.content(), "bc");
    }

    #[test]
    fn test_delete_at_line_end() {
        let mut editor = TextEditor::new("hello\nworld");
        editor.cursor_col = 5;

        let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(editor.handle_input(key));
        assert_eq!(editor.content(), "helloworld");
        assert_eq!(editor.line_count(), 1);
    }

    #[test]
    fn test_delete_at_end() {
        let mut editor = TextEditor::new("hello");
        editor.cursor_col = 5;

        let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(!editor.handle_input(key));
        assert_eq!(editor.content(), "hello");
    }

    #[test]
    fn test_move_left() {
        let mut editor = TextEditor::new("abc");
        editor.cursor_col = 2;

        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        editor.handle_input(key);
        assert_eq!(editor.cursor_col(), 1);
    }

    #[test]
    fn test_move_left_wrap() {
        let mut editor = TextEditor::new("hello\nworld");
        editor.cursor_line = 1;
        editor.cursor_col = 0;

        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        editor.handle_input(key);
        assert_eq!(editor.cursor_line(), 0);
        assert_eq!(editor.cursor_col(), 5);
    }

    #[test]
    fn test_move_right() {
        let mut editor = TextEditor::new("abc");
        editor.cursor_col = 1;

        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        editor.handle_input(key);
        assert_eq!(editor.cursor_col(), 2);
    }

    #[test]
    fn test_move_right_wrap() {
        let mut editor = TextEditor::new("hello\nworld");
        editor.cursor_col = 5;

        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        editor.handle_input(key);
        assert_eq!(editor.cursor_line(), 1);
        assert_eq!(editor.cursor_col(), 0);
    }

    #[test]
    fn test_move_up() {
        let mut editor = TextEditor::new("hello\nworld");
        editor.cursor_line = 1;
        editor.cursor_col = 3;

        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        editor.handle_input(key);
        assert_eq!(editor.cursor_line(), 0);
        assert_eq!(editor.cursor_col(), 3);
    }

    #[test]
    fn test_move_up_clamps_column() {
        let mut editor = TextEditor::new("hi\nhello");
        editor.cursor_line = 1;
        editor.cursor_col = 4;

        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        editor.handle_input(key);
        assert_eq!(editor.cursor_line(), 0);
        assert_eq!(editor.cursor_col(), 2); // Clamped to "hi" length
    }

    #[test]
    fn test_move_down() {
        let mut editor = TextEditor::new("hello\nworld");
        editor.cursor_col = 3;

        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        editor.handle_input(key);
        assert_eq!(editor.cursor_line(), 1);
        assert_eq!(editor.cursor_col(), 3);
    }

    #[test]
    fn test_home_end() {
        let mut editor = TextEditor::new("hello world");
        editor.cursor_col = 5;

        let key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        editor.handle_input(key);
        assert_eq!(editor.cursor_col(), 0);

        let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
        editor.handle_input(key);
        assert_eq!(editor.cursor_col(), 11);
    }

    #[test]
    fn test_ctrl_u() {
        let mut editor = TextEditor::new("hello world");
        editor.cursor_col = 6;

        let key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        assert!(editor.handle_input(key));
        assert_eq!(editor.content(), "world");
        assert_eq!(editor.cursor_col(), 0);
    }

    #[test]
    fn test_ctrl_k() {
        let mut editor = TextEditor::new("hello world");
        editor.cursor_col = 5;

        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
        assert!(editor.handle_input(key));
        assert_eq!(editor.content(), "hello");
    }

    #[test]
    fn test_ctrl_k_joins_lines() {
        let mut editor = TextEditor::new("hello\nworld");
        editor.cursor_col = 5;

        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
        assert!(editor.handle_input(key));
        assert_eq!(editor.content(), "helloworld");
    }

    #[test]
    fn test_has_changes() {
        let mut editor = TextEditor::new("hello");
        assert!(!editor.has_changes());

        let key = KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE);
        editor.handle_input(key);
        assert!(editor.has_changes());
    }

    #[test]
    fn test_scroll_info() {
        let editor = TextEditor::new("line1\nline2\nline3");
        assert_eq!(editor.scroll_info(), (1, 3)); // Line 1 of 3
    }
}
