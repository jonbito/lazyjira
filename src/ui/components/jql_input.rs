//! JQL query input component.
//!
//! This module provides a JQL (JIRA Query Language) input component with:
//! - Full text input with cursor navigation
//! - Query history (last 10 queries)
//! - Up/down arrows to cycle through history
//! - Syntax hints displayed below input
//! - Error message display for invalid JQL
//! - Query execution on Enter
//! - Cancel on Escape

use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::TextInput;

/// Actions that can be returned from the JQL input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JqlAction {
    /// Execute the JQL query.
    Execute(String),
    /// Cancel the input.
    Cancel,
}

/// JQL syntax hints for common fields.
const JQL_HINTS: &[(&str, &str)] = &[
    ("project = ", "Filter by project key"),
    ("status = ", "Filter by status name"),
    ("assignee = ", "Filter by assignee (use currentUser() for self)"),
    ("reporter = ", "Filter by reporter"),
    ("priority = ", "Filter by priority"),
    ("labels = ", "Filter by label"),
    ("sprint = ", "Filter by sprint"),
    ("created >= ", "Filter by creation date"),
    ("updated >= ", "Filter by update date"),
    ("ORDER BY ", "Sort results"),
];

/// Get a hint based on the current input.
fn get_hint_for_input(input: &str) -> Option<&'static str> {
    let input_lower = input.to_lowercase();
    let input_trimmed = input_lower.trim_end();

    // Check for partial matches at the end of input
    for (prefix, hint) in JQL_HINTS {
        let prefix_lower = prefix.trim().to_lowercase();
        if input_trimmed.ends_with(&prefix_lower) {
            return Some(hint);
        }
    }
    None
}

/// JQL input component for entering JIRA queries.
pub struct JqlInput {
    /// The text input widget.
    input: TextInput,
    /// Query history (most recent first).
    history: VecDeque<String>,
    /// Current position in history (None = not browsing history).
    history_index: Option<usize>,
    /// Text before history navigation (to restore when exiting history).
    pre_history_text: String,
    /// Whether the input is visible.
    visible: bool,
    /// Error message to display.
    error: Option<String>,
}

impl JqlInput {
    /// Maximum number of queries to keep in history.
    pub const MAX_HISTORY: usize = 10;

    /// Create a new JQL input component.
    pub fn new() -> Self {
        Self {
            input: TextInput::new(),
            history: VecDeque::new(),
            history_index: None,
            pre_history_text: String::new(),
            visible: false,
            error: None,
        }
    }

    /// Create a new JQL input component with existing history.
    pub fn with_history(history: Vec<String>) -> Self {
        let mut jql_input = Self::new();
        jql_input.set_history(history);
        jql_input
    }

    /// Set the query history.
    pub fn set_history(&mut self, history: Vec<String>) {
        self.history = history.into_iter().take(Self::MAX_HISTORY).collect();
    }

    /// Get the current history as a vector.
    pub fn history(&self) -> Vec<String> {
        self.history.iter().cloned().collect()
    }

    /// Show the JQL input.
    pub fn show(&mut self) {
        self.visible = true;
        self.input.clear();
        self.history_index = None;
        self.pre_history_text.clear();
        self.error = None;
    }

    /// Hide the JQL input.
    pub fn hide(&mut self) {
        self.visible = false;
        self.error = None;
    }

    /// Check if the input is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set an error message to display.
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
    }

    /// Clear the error message.
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Get the current input value.
    pub fn value(&self) -> &str {
        self.input.value()
    }

    /// Handle keyboard input.
    ///
    /// Returns an action if one should be performed.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<JqlAction> {
        // Clear error on any input
        self.error = None;

        match (key.code, key.modifiers) {
            // Enter - execute query
            (KeyCode::Enter, KeyModifiers::NONE) => {
                let query = self.input.value().trim().to_string();
                if !query.is_empty() {
                    self.add_to_history(query.clone());
                    self.hide();
                    return Some(JqlAction::Execute(query));
                }
                None
            }
            // Escape - cancel
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.hide();
                Some(JqlAction::Cancel)
            }
            // Up arrow - previous history entry
            (KeyCode::Up, KeyModifiers::NONE) => {
                self.history_prev();
                None
            }
            // Down arrow - next history entry
            (KeyCode::Down, KeyModifiers::NONE) => {
                self.history_next();
                None
            }
            // Any other input - delegate to TextInput
            _ => {
                let modified = self.input.handle_input(key);
                if modified {
                    // Reset history navigation when user types
                    self.history_index = None;
                }
                None
            }
        }
    }

    /// Move to the previous history entry (older).
    fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                // Save current text before entering history
                self.pre_history_text = self.input.value().to_string();
                self.history_index = Some(0);
                if let Some(query) = self.history.front() {
                    self.input.set_value(query);
                }
            }
            Some(idx) if idx + 1 < self.history.len() => {
                self.history_index = Some(idx + 1);
                if let Some(query) = self.history.get(idx + 1) {
                    self.input.set_value(query);
                }
            }
            _ => {
                // Already at oldest entry, do nothing
            }
        }
    }

    /// Move to the next history entry (newer).
    fn history_next(&mut self) {
        match self.history_index {
            None => {
                // Not in history, do nothing
            }
            Some(0) => {
                // At most recent history entry, restore pre-history text
                self.history_index = None;
                self.input.set_value(&self.pre_history_text);
            }
            Some(idx) => {
                self.history_index = Some(idx - 1);
                if let Some(query) = self.history.get(idx - 1) {
                    self.input.set_value(query);
                }
            }
        }
    }

    /// Add a query to the history.
    fn add_to_history(&mut self, query: String) {
        // Remove duplicate if exists
        self.history.retain(|q| q != &query);

        // Add to front
        self.history.push_front(query);

        // Trim to max size
        while self.history.len() > Self::MAX_HISTORY {
            self.history.pop_back();
        }
    }

    /// Render the JQL input.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate panel size (centered, ~70% width)
        let panel_width = (area.width as f32 * 0.70).min(100.0) as u16;
        let panel_height = 5; // Input (3) + hint/error (1) + padding (1)
        let panel_x = (area.width.saturating_sub(panel_width)) / 2;
        let panel_y = (area.height.saturating_sub(panel_height)) / 2;

        let panel_area = Rect::new(panel_x, panel_y, panel_width, panel_height);

        // Clear the background
        frame.render_widget(Clear, panel_area);

        // Create layout for input and hint
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input box
                Constraint::Length(1), // Hint/error line
            ])
            .split(panel_area);

        // Render input box
        let border_style = if self.error.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Cyan)
        };

        let input_block = Block::default()
            .title(" JQL Query ")
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(border_style);

        let input_inner = input_block.inner(chunks[0]);
        frame.render_widget(input_block, chunks[0]);

        // Render the input text with colon prefix
        let display = format!(":{}", self.input.value());
        let input_paragraph = Paragraph::new(display).style(Style::default().fg(Color::White));
        frame.render_widget(input_paragraph, input_inner);

        // Set cursor position (account for ':' prefix)
        frame.set_cursor_position(Position::new(
            input_inner.x + 1 + self.input.cursor() as u16,
            input_inner.y,
        ));

        // Render hint or error
        let hint_line = if let Some(error) = &self.error {
            Line::from(Span::styled(
                error.as_str(),
                Style::default().fg(Color::Red),
            ))
        } else if let Some(hint) = get_hint_for_input(self.input.value()) {
            Line::from(Span::styled(hint, Style::default().fg(Color::Yellow)))
        } else {
            Line::from(vec![
                Span::styled(
                    "Enter JQL (e.g., project = PROJ AND status = \"In Progress\")",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(" "),
                Span::styled(
                    "↑↓:history",
                    Style::default().fg(Color::Cyan),
                ),
            ])
        };

        let hint_paragraph = Paragraph::new(hint_line);
        frame.render_widget(hint_paragraph, chunks[1]);
    }
}

impl Default for JqlInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let input = JqlInput::new();
        assert!(!input.is_visible());
        assert!(input.history().is_empty());
        assert!(input.value().is_empty());
    }

    #[test]
    fn test_with_history() {
        let history = vec![
            "project = TEST".to_string(),
            "status = Open".to_string(),
        ];
        let input = JqlInput::with_history(history.clone());
        assert_eq!(input.history(), history);
    }

    #[test]
    fn test_show_hide() {
        let mut input = JqlInput::new();
        assert!(!input.is_visible());

        input.show();
        assert!(input.is_visible());
        assert!(input.value().is_empty());

        input.hide();
        assert!(!input.is_visible());
    }

    #[test]
    fn test_show_clears_state() {
        let mut input = JqlInput::new();
        input.set_error("Test error");
        input.show();

        assert!(input.error.is_none());
        assert!(input.history_index.is_none());
    }

    #[test]
    fn test_set_error() {
        let mut input = JqlInput::new();
        input.set_error("Invalid JQL");
        assert_eq!(input.error, Some("Invalid JQL".to_string()));

        input.clear_error();
        assert!(input.error.is_none());
    }

    #[test]
    fn test_execute_empty_query() {
        let mut input = JqlInput::new();
        input.show();

        let action = input.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        // Empty query should not execute
        assert!(action.is_none());
        assert!(input.is_visible());
    }

    #[test]
    fn test_execute_query() {
        let mut input = JqlInput::new();
        input.show();

        // Type a query
        for c in "project = TEST".chars() {
            input.handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }

        let action = input.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, Some(JqlAction::Execute("project = TEST".to_string())));
        assert!(!input.is_visible());
    }

    #[test]
    fn test_cancel() {
        let mut input = JqlInput::new();
        input.show();

        let action = input.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(action, Some(JqlAction::Cancel));
        assert!(!input.is_visible());
    }

    #[test]
    fn test_history_navigation() {
        let mut input = JqlInput::with_history(vec![
            "query1".to_string(),
            "query2".to_string(),
            "query3".to_string(),
        ]);
        input.show();

        // Type some text
        for c in "current".chars() {
            input.handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
        assert_eq!(input.value(), "current");

        // Up arrow should go to most recent history (query1)
        input.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(input.value(), "query1");
        assert_eq!(input.history_index, Some(0));

        // Up again should go to query2
        input.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(input.value(), "query2");
        assert_eq!(input.history_index, Some(1));

        // Down should go back to query1
        input.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(input.value(), "query1");
        assert_eq!(input.history_index, Some(0));

        // Down again should restore original text
        input.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(input.value(), "current");
        assert!(input.history_index.is_none());
    }

    #[test]
    fn test_history_empty() {
        let mut input = JqlInput::new();
        input.show();

        // Up/down should do nothing with empty history
        input.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert!(input.history_index.is_none());

        input.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert!(input.history_index.is_none());
    }

    #[test]
    fn test_add_to_history() {
        let mut input = JqlInput::new();
        input.show();

        // Execute a query
        for c in "query1".chars() {
            input.handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
        input.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        // History should contain the query
        assert_eq!(input.history(), vec!["query1".to_string()]);
    }

    #[test]
    fn test_history_deduplication() {
        let mut input = JqlInput::with_history(vec![
            "query1".to_string(),
            "query2".to_string(),
        ]);

        // Add query1 again - it should move to front
        input.add_to_history("query1".to_string());

        assert_eq!(input.history(), vec!["query1".to_string(), "query2".to_string()]);
    }

    #[test]
    fn test_history_max_size() {
        let mut input = JqlInput::new();

        // Add more than MAX_HISTORY items
        for i in 0..15 {
            input.add_to_history(format!("query{}", i));
        }

        // Should be limited to MAX_HISTORY
        assert_eq!(input.history().len(), JqlInput::MAX_HISTORY);
        // Most recent should be first
        assert!(input.history()[0].starts_with("query14"));
    }

    #[test]
    fn test_typing_resets_history_index() {
        let mut input = JqlInput::with_history(vec!["query1".to_string()]);
        input.show();

        // Navigate to history
        input.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(input.history_index, Some(0));

        // Type a character
        input.handle_input(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));

        // History index should be reset
        assert!(input.history_index.is_none());
    }

    #[test]
    fn test_input_clears_error() {
        let mut input = JqlInput::new();
        input.show();
        input.set_error("Test error");

        // Any key should clear the error
        input.handle_input(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        assert!(input.error.is_none());
    }

    #[test]
    fn test_get_hint_for_input() {
        // Test partial matches at the end - the hint triggers when you finish typing the prefix
        assert_eq!(get_hint_for_input("project ="), Some("Filter by project key"));
        assert_eq!(get_hint_for_input("status ="), Some("Filter by status name"));
        assert_eq!(get_hint_for_input("assignee ="), Some("Filter by assignee (use currentUser() for self)"));
        assert_eq!(get_hint_for_input("ORDER BY"), Some("Sort results"));

        // Case insensitive
        assert_eq!(get_hint_for_input("PROJECT ="), Some("Filter by project key"));

        // After space should still match since we trim the prefix
        assert_eq!(get_hint_for_input("project = "), Some("Filter by project key"));

        // No match for unrelated text
        assert!(get_hint_for_input("random text").is_none());
    }

    #[test]
    fn test_history_at_oldest() {
        let mut input = JqlInput::with_history(vec![
            "query1".to_string(),
            "query2".to_string(),
        ]);
        input.show();

        // Navigate to oldest
        input.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        input.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(input.value(), "query2");

        // Another Up should stay at oldest
        input.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(input.value(), "query2");
    }

    #[test]
    fn test_down_without_history_navigation() {
        let mut input = JqlInput::with_history(vec!["query1".to_string()]);
        input.show();

        // Down without being in history should do nothing
        input.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert!(input.history_index.is_none());
        assert!(input.value().is_empty());
    }
}
