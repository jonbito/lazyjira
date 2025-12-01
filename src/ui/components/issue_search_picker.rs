//! Issue search picker component for selecting target issues when linking.
//!
//! Displays a search input with autocomplete suggestions from the JIRA
//! issue picker API. Users can search by issue key or text and select
//! from the results.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::api::types::IssueSuggestion;

/// Action resulting from issue search picker input.
#[derive(Debug, Clone, PartialEq)]
pub enum IssueSearchPickerAction {
    /// User selected an issue (issue key).
    Select(String),
    /// User wants to search (query string).
    /// Parent should call API and then call set_suggestions().
    Search(String),
    /// User cancelled the picker.
    Cancel,
}

/// Issue search picker component.
///
/// Allows the user to search for issues and select one for linking.
#[derive(Debug)]
pub struct IssueSearchPicker {
    /// Current search query.
    query: String,
    /// Suggestions from the API.
    suggestions: Vec<IssueSuggestion>,
    /// Currently selected suggestion index.
    selected: usize,
    /// Whether the picker is visible.
    visible: bool,
    /// Whether suggestions are loading.
    loading: bool,
    /// The current issue key (to exclude from suggestions).
    current_issue_key: Option<String>,
}

impl IssueSearchPicker {
    /// Create a new issue search picker.
    pub fn new() -> Self {
        Self {
            query: String::new(),
            suggestions: Vec::new(),
            selected: 0,
            visible: false,
            loading: false,
            current_issue_key: None,
        }
    }

    /// Check if the picker is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if suggestions are loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Get the current query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Get the current issue key (for excluding from search).
    pub fn current_issue_key(&self) -> Option<&str> {
        self.current_issue_key.as_deref()
    }

    /// Show the picker, optionally setting the current issue key.
    pub fn show(&mut self, current_issue_key: Option<String>) {
        self.query.clear();
        self.suggestions.clear();
        self.selected = 0;
        self.loading = false;
        self.visible = true;
        self.current_issue_key = current_issue_key;
    }

    /// Hide the picker.
    pub fn hide(&mut self) {
        self.visible = false;
        self.loading = false;
    }

    /// Set loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Set suggestions from API response.
    pub fn set_suggestions(&mut self, suggestions: Vec<IssueSuggestion>) {
        self.suggestions = suggestions;
        self.selected = 0;
        self.loading = false;
    }

    /// Get the number of suggestions.
    pub fn suggestion_count(&self) -> usize {
        self.suggestions.len()
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the parent view.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<IssueSearchPickerAction> {
        if !self.visible {
            return None;
        }

        match (key.code, key.modifiers) {
            // Cancel with Esc
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.hide();
                Some(IssueSearchPickerAction::Cancel)
            }
            // Ignore other input while loading
            _ if self.loading => None,
            // Navigation down with arrow (Ctrl+j also works for some users)
            (KeyCode::Down, _) | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                if !self.suggestions.is_empty() && self.selected < self.suggestions.len() - 1 {
                    self.selected += 1;
                }
                None
            }
            // Navigation up with arrow (Ctrl+k also works)
            (KeyCode::Up, _) | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            // Select suggestion or trigger search
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if self.suggestions.is_empty() {
                    // No suggestions - trigger search if we have a query
                    if !self.query.is_empty() {
                        self.loading = true;
                        Some(IssueSearchPickerAction::Search(self.query.clone()))
                    } else {
                        None
                    }
                } else if let Some(suggestion) = self.suggestions.get(self.selected) {
                    let key = suggestion.key.clone();
                    self.hide();
                    Some(IssueSearchPickerAction::Select(key))
                } else {
                    None
                }
            }
            // Tab to trigger search
            (KeyCode::Tab, KeyModifiers::NONE) => {
                if !self.query.is_empty() {
                    self.loading = true;
                    Some(IssueSearchPickerAction::Search(self.query.clone()))
                } else {
                    None
                }
            }
            // Backspace to remove character
            (KeyCode::Backspace, _) => {
                self.query.pop();
                // Clear suggestions when query changes
                self.suggestions.clear();
                self.selected = 0;
                None
            }
            // Character input
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.query.push(c);
                // Clear old suggestions when typing
                self.suggestions.clear();
                self.selected = 0;
                None
            }
            _ => None,
        }
    }

    /// Render the issue search picker.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate dialog size and position (centered)
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 16.min(area.height.saturating_sub(4));

        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Clear the background
        frame.render_widget(Clear, dialog_area);

        // Create the dialog block
        let block = Block::default()
            .title(" Search for Issue ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Split inner area for input, suggestions, and help
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search input
                Constraint::Min(3),    // Suggestions list
                Constraint::Length(2), // Help text
            ])
            .split(inner);

        // Render search input
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" Issue key or text ");

        let input_inner = input_block.inner(chunks[0]);
        frame.render_widget(input_block, chunks[0]);

        let input_text = Paragraph::new(self.query.as_str())
            .style(Style::default().fg(Color::White));
        frame.render_widget(input_text, input_inner);

        // Show cursor at end of input
        if !self.loading {
            frame.set_cursor_position(Position::new(
                input_inner.x + self.query.len() as u16,
                input_inner.y,
            ));
        }

        // Render loading, empty, or suggestions
        if self.loading {
            let loading_text = Paragraph::new("Searching...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading_text, chunks[1]);
        } else if self.suggestions.is_empty() {
            let hint = if self.query.is_empty() {
                "Type to search, press Tab or Enter to search"
            } else {
                "Press Tab or Enter to search"
            };
            let empty_text = Paragraph::new(hint)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_text, chunks[1]);
        } else {
            // Build list items
            let items: Vec<ListItem> = self
                .suggestions
                .iter()
                .map(|s| {
                    let summary = s.summary_text.as_deref()
                        .or(s.summary.as_deref())
                        .unwrap_or("");
                    let text = format!("{}: {}", s.key, truncate(summary, 40));
                    ListItem::new(text).style(Style::default().fg(Color::White))
                })
                .collect();

            let list = List::new(items)
                .highlight_style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            let mut state = ListState::default();
            state.select(Some(self.selected));

            frame.render_stateful_widget(list, chunks[1], &mut state);
        }

        // Render help text
        let help_text = Line::from(vec![
            Span::styled("Tab/Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": search  "),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(": navigate  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": cancel"),
        ]);
        let help_paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
        frame.render_widget(help_paragraph, chunks[2]);
    }
}

impl Default for IssueSearchPicker {
    fn default() -> Self {
        Self::new()
    }
}

/// Truncate a string to the given maximum length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Create a centered rectangle.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_suggestion(key: &str, summary: &str) -> IssueSuggestion {
        IssueSuggestion {
            key: key.to_string(),
            summary_text: Some(summary.to_string()),
            summary: None,
            id: Some(123),
        }
    }

    #[test]
    fn test_new_picker() {
        let picker = IssueSearchPicker::new();
        assert!(!picker.is_visible());
        assert!(!picker.is_loading());
        assert!(picker.query().is_empty());
        assert_eq!(picker.suggestion_count(), 0);
    }

    #[test]
    fn test_show() {
        let mut picker = IssueSearchPicker::new();
        picker.show(Some("TEST-123".to_string()));

        assert!(picker.is_visible());
        assert!(!picker.is_loading());
        assert!(picker.query().is_empty());
        assert_eq!(picker.current_issue_key(), Some("TEST-123"));
    }

    #[test]
    fn test_hide() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        assert!(picker.is_visible());

        picker.hide();
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_set_suggestions() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_loading(true);

        let suggestions = vec![
            create_test_suggestion("TEST-1", "First issue"),
            create_test_suggestion("TEST-2", "Second issue"),
        ];

        picker.set_suggestions(suggestions);

        assert!(!picker.is_loading());
        assert_eq!(picker.suggestion_count(), 2);
    }

    #[test]
    fn test_typing_query() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
        picker.handle_input(key);
        let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        picker.handle_input(key);
        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        picker.handle_input(key);
        let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
        picker.handle_input(key);

        assert_eq!(picker.query(), "test");
    }

    #[test]
    fn test_backspace() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        // Type "test"
        for c in "test".chars() {
            let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            picker.handle_input(key);
        }

        // Backspace
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        picker.handle_input(key);

        assert_eq!(picker.query(), "tes");
    }

    #[test]
    fn test_navigation_down() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_suggestions(vec![
            create_test_suggestion("TEST-1", "First"),
            create_test_suggestion("TEST-2", "Second"),
        ]);

        assert_eq!(picker.selected, 0);

        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 1);

        // Can't go past end
        picker.handle_input(key);
        assert_eq!(picker.selected, 1);
    }

    #[test]
    fn test_navigation_up() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_suggestions(vec![
            create_test_suggestion("TEST-1", "First"),
            create_test_suggestion("TEST-2", "Second"),
        ]);
        picker.selected = 1;

        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 0);

        // Can't go past beginning
        picker.handle_input(key);
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_select_suggestion() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_suggestions(vec![
            create_test_suggestion("TEST-1", "First"),
            create_test_suggestion("TEST-2", "Second"),
        ]);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(IssueSearchPickerAction::Select("TEST-1".to_string())));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_enter_triggers_search() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        // Type query
        for c in "test".chars() {
            let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            picker.handle_input(key);
        }

        // Press Enter with no suggestions
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(IssueSearchPickerAction::Search("test".to_string())));
        assert!(picker.is_loading());
    }

    #[test]
    fn test_tab_triggers_search() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        // Type query
        for c in "proj".chars() {
            let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            picker.handle_input(key);
        }

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(IssueSearchPickerAction::Search("proj".to_string())));
    }

    #[test]
    fn test_cancel_with_esc() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(IssueSearchPickerAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_esc_while_loading() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_loading(true);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(IssueSearchPickerAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_input_ignored_while_loading() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_loading(true);

        // Typing should be ignored while loading
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert!(picker.query().is_empty());
    }

    #[test]
    fn test_input_ignored_when_not_visible() {
        let mut picker = IssueSearchPicker::new();

        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_typing_clears_old_suggestions() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_suggestions(vec![
            create_test_suggestion("OLD-1", "Old result"),
        ]);
        picker.selected = 0;

        assert_eq!(picker.suggestion_count(), 1);

        // Type new character
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        picker.handle_input(key);

        // Suggestions should be cleared
        assert_eq!(picker.suggestion_count(), 0);
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_default_impl() {
        let picker = IssueSearchPicker::default();
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is...");
        assert_eq!(truncate("exact", 5), "exact");
    }

    #[test]
    fn test_navigation_with_ctrl_keys() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_suggestions(vec![
            create_test_suggestion("TEST-1", "First"),
            create_test_suggestion("TEST-2", "Second"),
        ]);

        assert_eq!(picker.selected, 0);

        // Ctrl+N moves down
        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
        picker.handle_input(key);
        assert_eq!(picker.selected, 1);

        // Ctrl+P moves up
        let key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
        picker.handle_input(key);
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_empty_query_no_search() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        // Try to search with empty query
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert!(action.is_none());
        assert!(!picker.is_loading());
    }
}
