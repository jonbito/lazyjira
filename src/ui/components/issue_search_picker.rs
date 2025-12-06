//! Issue search picker component for selecting target issues when linking.
//!
//! Displays a search input with autocomplete suggestions from the JIRA
//! issue picker API. Users can search by issue key or text and select
//! from the results.
//!
//! Supports two modes (vim-style):
//! - Normal mode: j/k navigate, / starts search, Enter/Space selects, q/Esc cancels
//! - Search mode: type to search, Enter triggers API search, Esc returns to Normal mode

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::InputMode;
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
///
/// Supports two modes (vim-style):
/// - Normal mode: j/k navigate, / starts search, Enter/Space selects, q/Esc cancels
/// - Search mode: type to search, Enter triggers API search, Esc returns to Normal mode
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
    /// Current input mode (Normal for navigation, Insert for typing).
    input_mode: InputMode,
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
            input_mode: InputMode::Normal,
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
        self.current_issue_key = current_issue_key.clone();
        self.input_mode = InputMode::Normal;
    }

    /// Show the picker in loading state (for fetching initial suggestions).
    pub fn show_loading(&mut self, current_issue_key: Option<String>) {
        self.query.clear();
        self.suggestions.clear();
        self.selected = 0;
        self.loading = true;
        self.visible = true;
        self.current_issue_key = current_issue_key;
        self.input_mode = InputMode::Normal;
    }

    /// Hide the picker.
    pub fn hide(&mut self) {
        self.visible = false;
        self.loading = false;
        self.input_mode = InputMode::Normal;
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
        // Return to normal mode after search completes
        self.input_mode = InputMode::Normal;
    }

    /// Get the number of suggestions.
    pub fn suggestion_count(&self) -> usize {
        self.suggestions.len()
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the parent view.
    ///
    /// Two modes (vim-style):
    /// - Normal mode: j/k navigate, / starts search, Enter/Space selects, q/Esc cancel
    /// - Search mode: type to search, Enter triggers API search, Esc returns to Normal mode
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<IssueSearchPickerAction> {
        if !self.visible {
            return None;
        }

        // Don't handle input while loading (except Esc)
        if self.loading {
            if key.code == KeyCode::Esc {
                self.hide();
                return Some(IssueSearchPickerAction::Cancel);
            }
            return None;
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_input(key),
            InputMode::Insert => self.handle_insert_input(key),
        }
    }

    /// Handle input in Normal mode (navigation).
    fn handle_normal_input(&mut self, key: KeyEvent) -> Option<IssueSearchPickerAction> {
        match (key.code, key.modifiers) {
            // Navigation down with j or arrow
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                if !self.suggestions.is_empty() && self.selected < self.suggestions.len() - 1 {
                    self.selected += 1;
                }
                None
            }
            // Navigation up with k or arrow
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            // '/' - enter search mode (vim-style)
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                self.input_mode = InputMode::Insert;
                None
            }
            // Enter or Space - select current suggestion
            (KeyCode::Enter, KeyModifiers::NONE) | (KeyCode::Char(' '), KeyModifiers::NONE) => {
                if let Some(suggestion) = self.suggestions.get(self.selected) {
                    let key = suggestion.key.clone();
                    self.hide();
                    Some(IssueSearchPickerAction::Select(key))
                } else {
                    // No suggestions yet - enter search mode
                    self.input_mode = InputMode::Insert;
                    None
                }
            }
            // Cancel with q or Esc
            (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.hide();
                Some(IssueSearchPickerAction::Cancel)
            }
            _ => None,
        }
    }

    /// Handle input in Insert mode (typing to search).
    fn handle_insert_input(&mut self, key: KeyEvent) -> Option<IssueSearchPickerAction> {
        match (key.code, key.modifiers) {
            // Enter - trigger search and return to Normal mode
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if !self.query.is_empty() {
                    self.loading = true;
                    Some(IssueSearchPickerAction::Search(self.query.clone()))
                } else {
                    self.input_mode = InputMode::Normal;
                    None
                }
            }
            // Esc - return to Normal mode without searching
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.input_mode = InputMode::Normal;
                None
            }
            // Arrow keys still work for navigation in search mode
            (KeyCode::Down, _) => {
                if !self.suggestions.is_empty() && self.selected < self.suggestions.len() - 1 {
                    self.selected += 1;
                }
                None
            }
            (KeyCode::Up, _) => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            // Backspace - delete from query
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                if !self.query.is_empty() {
                    self.query.pop();
                    // Clear suggestions when query changes
                    self.suggestions.clear();
                    self.selected = 0;
                }
                None
            }
            // Character input - add to query
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
        let dialog_height = 18.min(area.height.saturating_sub(4));

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

        // Split inner area for search bar, suggestions, and help
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Search bar
                Constraint::Min(3),    // Suggestions list
                Constraint::Length(2), // Help text
            ])
            .split(inner);

        // Render search bar (vim-style with "/" prompt)
        let search_line = match self.input_mode {
            InputMode::Insert => {
                // Show "/" prompt when in search mode
                Line::from(vec![
                    Span::styled(
                        "/",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&self.query, Style::default().fg(Color::White)),
                    Span::styled("▏", Style::default().fg(Color::Yellow)), // Cursor
                ])
            }
            InputMode::Normal => {
                if self.query.is_empty() {
                    Line::from(Span::styled(
                        "Press / to search...",
                        Style::default().fg(Color::DarkGray),
                    ))
                } else {
                    // Show current query
                    Line::from(vec![
                        Span::styled("/", Style::default().fg(Color::DarkGray)),
                        Span::styled(&self.query, Style::default().fg(Color::Cyan)),
                    ])
                }
            }
        };
        frame.render_widget(Paragraph::new(search_line), chunks[0]);

        // Render loading, empty, or suggestions
        if self.loading {
            let loading_text = Paragraph::new("Loading issues...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading_text, chunks[1]);
        } else if self.suggestions.is_empty() {
            let hint = if self.query.is_empty() {
                "No recent issues. Press / to search"
            } else {
                "No results. Press / to search again"
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
                    let summary = s
                        .summary_text
                        .as_deref()
                        .or(s.summary.as_deref())
                        .unwrap_or("");
                    let text = format!("  {}: {}", s.key, truncate(summary, 40));
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

        // Render help text based on current mode
        let help_text = match self.input_mode {
            InputMode::Normal => Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Yellow)),
                Span::raw(": navigate  "),
                Span::styled("/", Style::default().fg(Color::Cyan)),
                Span::raw(": search  "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(": select  "),
                Span::styled("q", Style::default().fg(Color::Red)),
                Span::raw(": cancel"),
            ]),
            InputMode::Insert => Line::from(vec![
                Span::styled(
                    "-- SEARCH --",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  type query  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(": search  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(": done"),
            ]),
        };
        frame.render_widget(
            Paragraph::new(help_text).alignment(Alignment::Center),
            chunks[2],
        );
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
        assert_eq!(picker.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_show_loading() {
        let mut picker = IssueSearchPicker::new();
        picker.show_loading(Some("TEST-123".to_string()));

        assert!(picker.is_visible());
        assert!(picker.is_loading());
        assert!(picker.query().is_empty());
        assert_eq!(picker.current_issue_key(), Some("TEST-123"));
        assert_eq!(picker.input_mode, InputMode::Normal);
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
        // Should return to Normal mode after search completes
        assert_eq!(picker.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_slash_enters_search_mode() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        // '/' in Normal mode enters search (Insert) mode
        let key = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert!(action.is_none());
        assert_eq!(picker.input_mode, InputMode::Insert);
    }

    #[test]
    fn test_typing_query_in_insert_mode() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        // Enter search mode with '/'
        let slash = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        picker.handle_input(slash);

        // Type "test"
        for c in "test".chars() {
            let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            picker.handle_input(key);
        }

        assert_eq!(picker.query(), "test");
    }

    #[test]
    fn test_backspace_in_insert_mode() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        // Enter search mode and type "test"
        let slash = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        picker.handle_input(slash);

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
    fn test_navigation_down_with_j() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_suggestions(vec![
            create_test_suggestion("TEST-1", "First"),
            create_test_suggestion("TEST-2", "Second"),
        ]);

        assert_eq!(picker.selected, 0);

        // 'j' moves down in Normal mode
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 1);

        // Can't go past end
        picker.handle_input(key);
        assert_eq!(picker.selected, 1);
    }

    #[test]
    fn test_navigation_down_with_arrow() {
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
    }

    #[test]
    fn test_navigation_up_with_k() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_suggestions(vec![
            create_test_suggestion("TEST-1", "First"),
            create_test_suggestion("TEST-2", "Second"),
        ]);
        picker.selected = 1;

        // 'k' moves up in Normal mode
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 0);

        // Can't go past beginning
        picker.handle_input(key);
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_navigation_up_with_arrow() {
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
    }

    #[test]
    fn test_select_suggestion_with_enter() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_suggestions(vec![
            create_test_suggestion("TEST-1", "First"),
            create_test_suggestion("TEST-2", "Second"),
        ]);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(
            action,
            Some(IssueSearchPickerAction::Select("TEST-1".to_string()))
        );
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_select_suggestion_with_space() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_suggestions(vec![
            create_test_suggestion("TEST-1", "First"),
            create_test_suggestion("TEST-2", "Second"),
        ]);

        let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(
            action,
            Some(IssueSearchPickerAction::Select("TEST-1".to_string()))
        );
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_enter_triggers_search_in_insert_mode() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        // Enter search mode and type query
        let slash = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        picker.handle_input(slash);

        for c in "test".chars() {
            let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            picker.handle_input(key);
        }

        // Press Enter to trigger search
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(
            action,
            Some(IssueSearchPickerAction::Search("test".to_string()))
        );
        assert!(picker.is_loading());
    }

    #[test]
    fn test_enter_with_no_suggestions_enters_search_mode() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        // Press Enter with no suggestions - should enter search mode
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert!(action.is_none());
        assert_eq!(picker.input_mode, InputMode::Insert);
    }

    #[test]
    fn test_cancel_with_esc_in_normal_mode() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(IssueSearchPickerAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_cancel_with_q() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(IssueSearchPickerAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_esc_in_insert_mode_returns_to_normal() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        // Enter search mode
        let slash = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        picker.handle_input(slash);
        assert_eq!(picker.input_mode, InputMode::Insert);

        // Esc returns to Normal mode
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert!(action.is_none());
        assert_eq!(picker.input_mode, InputMode::Normal);
        assert!(picker.is_visible()); // Still visible
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
        picker.set_suggestions(vec![create_test_suggestion("OLD-1", "Old result")]);

        assert_eq!(picker.suggestion_count(), 1);

        // Enter search mode and type new character
        let slash = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        picker.handle_input(slash);

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
    fn test_navigation_with_jk_in_normal_mode() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_suggestions(vec![
            create_test_suggestion("TEST-1", "First"),
            create_test_suggestion("TEST-2", "Second"),
        ]);

        assert_eq!(picker.selected, 0);

        // 'j' moves down
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 1);

        // 'k' moves up
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_jk_adds_to_search_in_insert_mode() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        // Enter search mode with '/'
        let slash = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        picker.handle_input(slash);
        assert_eq!(picker.input_mode, InputMode::Insert);

        // 'j' in search mode should add to search query
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        picker.handle_input(key);

        assert_eq!(picker.query(), "j");
        assert!(picker.is_visible());
    }

    #[test]
    fn test_empty_query_no_search() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);

        // Enter search mode
        let slash = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        picker.handle_input(slash);

        // Try to search with empty query - should return to Normal mode
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert!(action.is_none());
        assert!(!picker.is_loading());
        assert_eq!(picker.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_arrow_navigation_in_insert_mode() {
        let mut picker = IssueSearchPicker::new();
        picker.show(None);
        picker.set_suggestions(vec![
            create_test_suggestion("TEST-1", "First"),
            create_test_suggestion("TEST-2", "Second"),
        ]);

        // Enter search mode
        let slash = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        picker.handle_input(slash);

        // Arrow keys still work for navigation in Insert mode
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 1);

        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 0);
    }
}
