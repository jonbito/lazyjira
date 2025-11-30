//! Issue list view.
//!
//! Displays a table of JIRA issues with columns for Key, Summary, Status,
//! Assignee, and Priority. Supports keyboard navigation and visual indicators
//! for issue priority and type.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::api::types::Issue;
use crate::ui::theme::{issue_type_prefix, priority_style, status_style, truncate};

/// Action that can be triggered from the list view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListAction {
    /// Open the selected issue for detailed view.
    OpenIssue(String),
    /// Refresh the issue list.
    Refresh,
    /// Open the filter panel.
    OpenFilter,
    /// Open the JQL query input.
    OpenJqlInput,
}

/// The issue list view state.
pub struct ListView {
    /// The list of issues to display.
    issues: Vec<Issue>,
    /// Currently selected index.
    selected: usize,
    /// Scroll offset for the table.
    scroll_offset: usize,
    /// Table state for ratatui.
    table_state: TableState,
    /// Whether the view is currently loading data.
    loading: bool,
    /// Current profile name (for status bar).
    profile_name: Option<String>,
    /// Pending 'g' key for gg navigation.
    pending_g: bool,
    /// Filter summary to display in the status bar.
    filter_summary: Option<String>,
}

impl ListView {
    /// Create a new list view.
    pub fn new() -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Self {
            issues: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            table_state,
            loading: false,
            profile_name: None,
            pending_g: false,
            filter_summary: None,
        }
    }

    /// Set the list of issues to display.
    pub fn set_issues(&mut self, issues: Vec<Issue>) {
        self.issues = issues;
        self.selected = 0;
        self.scroll_offset = 0;
        self.table_state.select(Some(0));
        self.loading = false;
    }

    /// Set the loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Check if the view is loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Set the current profile name.
    pub fn set_profile_name(&mut self, name: Option<String>) {
        self.profile_name = name;
    }

    /// Set the filter summary to display.
    pub fn set_filter_summary(&mut self, summary: Option<String>) {
        self.filter_summary = summary;
    }

    /// Get the current filter summary.
    pub fn filter_summary(&self) -> Option<&str> {
        self.filter_summary.as_deref()
    }

    /// Get the currently selected issue.
    pub fn selected_issue(&self) -> Option<&Issue> {
        self.issues.get(self.selected)
    }

    /// Get the selected index.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Get the number of issues.
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the application.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<ListAction> {
        // Handle pending 'g' for gg navigation
        if self.pending_g {
            self.pending_g = false;
            if key.code == KeyCode::Char('g') && key.modifiers == KeyModifiers::NONE {
                self.move_to_start();
                return None;
            }
            // If not 'g', fall through to normal handling
        }

        match (key.code, key.modifiers) {
            // Navigation
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                self.move_down();
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                self.move_up();
            }
            (KeyCode::Char('g'), KeyModifiers::NONE) => {
                // Wait for second 'g'
                self.pending_g = true;
            }
            (KeyCode::Char('G'), KeyModifiers::SHIFT) | (KeyCode::Char('G'), KeyModifiers::NONE) => {
                self.move_to_end();
            }
            (KeyCode::Home, _) => {
                self.move_to_start();
            }
            (KeyCode::End, _) => {
                self.move_to_end();
            }
            (KeyCode::PageDown, _) | (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                self.page_down();
            }
            (KeyCode::PageUp, _) | (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                self.page_up();
            }
            // Actions
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(issue) = self.selected_issue() {
                    return Some(ListAction::OpenIssue(issue.key.clone()));
                }
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                return Some(ListAction::Refresh);
            }
            (KeyCode::Char('f'), KeyModifiers::NONE) => {
                return Some(ListAction::OpenFilter);
            }
            // JQL input
            (KeyCode::Char(':'), KeyModifiers::NONE)
            | (KeyCode::Char(':'), KeyModifiers::SHIFT)
            | (KeyCode::Char('/'), KeyModifiers::NONE) => {
                return Some(ListAction::OpenJqlInput);
            }
            _ => {}
        }
        None
    }

    /// Move selection down by one.
    fn move_down(&mut self) {
        if self.issues.is_empty() {
            return;
        }
        if self.selected < self.issues.len() - 1 {
            self.selected += 1;
            self.table_state.select(Some(self.selected));
        }
    }

    /// Move selection up by one.
    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.table_state.select(Some(self.selected));
        }
    }

    /// Move selection to the first item.
    fn move_to_start(&mut self) {
        self.selected = 0;
        self.table_state.select(Some(0));
    }

    /// Move selection to the last item.
    fn move_to_end(&mut self) {
        if !self.issues.is_empty() {
            self.selected = self.issues.len() - 1;
            self.table_state.select(Some(self.selected));
        }
    }

    /// Move selection down by a page (10 items).
    fn page_down(&mut self) {
        if self.issues.is_empty() {
            return;
        }
        let page_size = 10;
        self.selected = (self.selected + page_size).min(self.issues.len() - 1);
        self.table_state.select(Some(self.selected));
    }

    /// Move selection up by a page (10 items).
    fn page_up(&mut self) {
        let page_size = 10;
        self.selected = self.selected.saturating_sub(page_size);
        self.table_state.select(Some(self.selected));
    }

    /// Render the list view.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.loading {
            self.render_loading(frame, area);
        } else if self.issues.is_empty() {
            self.render_empty(frame, area);
        } else {
            self.render_table(frame, area);
        }
    }

    /// Render the loading state.
    fn render_loading(&self, frame: &mut Frame, area: Rect) {
        let loading = Paragraph::new("Loading issues...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));

        // Center the loading message vertically
        let vertical_center = area.y + area.height / 2;
        let centered_area = Rect {
            x: area.x,
            y: vertical_center.saturating_sub(1),
            width: area.width,
            height: 3,
        };

        frame.render_widget(loading, centered_area);
    }

    /// Render the empty state.
    fn render_empty(&self, frame: &mut Frame, area: Rect) {
        let message = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No issues found",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'f' to change filters",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "Press 'r' to refresh",
                Style::default().fg(Color::Gray),
            )),
        ];

        let paragraph = Paragraph::new(message).alignment(Alignment::Center);

        // Center vertically
        let vertical_center = area.y + area.height / 2;
        let centered_area = Rect {
            x: area.x,
            y: vertical_center.saturating_sub(3),
            width: area.width,
            height: 6,
        };

        frame.render_widget(paragraph, centered_area);
    }

    /// Render the issue table.
    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        // Calculate column widths based on available space
        let key_width = 14; // PROJ-12345
        let status_width = 15;
        let assignee_width = 20;
        let priority_width = 10;
        let min_summary_width = 30;

        // Summary gets remaining space
        let fixed_width = key_width + status_width + assignee_width + priority_width + 8; // 8 for borders/spacing
        let summary_width = if area.width as usize > fixed_width + min_summary_width {
            area.width as usize - fixed_width
        } else {
            min_summary_width
        };

        // Create header
        let header_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        let header = Row::new(vec![
            Cell::from("Key").style(header_style),
            Cell::from("Summary").style(header_style),
            Cell::from("Status").style(header_style),
            Cell::from("Assignee").style(header_style),
            Cell::from("Priority").style(header_style),
        ])
        .height(1)
        .bottom_margin(1);

        // Create rows
        let rows: Vec<Row> = self
            .issues
            .iter()
            .map(|issue| {
                // Key with type prefix
                let type_prefix = issue_type_prefix(&issue.fields.issuetype.name);
                let key_text = format!("{} {}", type_prefix, issue.key);

                // Truncate summary to fit
                let summary = truncate(&issue.fields.summary, summary_width);

                // Status with color
                let status_style = status_style(&issue.fields.status);

                // Priority with color
                let priority_style = priority_style(issue.fields.priority.as_ref());

                Row::new(vec![
                    Cell::from(key_text),
                    Cell::from(summary),
                    Cell::from(issue.fields.status.name.clone()).style(status_style),
                    Cell::from(issue.assignee_name().to_string()),
                    Cell::from(issue.priority_name().to_string()).style(priority_style),
                ])
            })
            .collect();

        // Define column constraints
        let widths = [
            Constraint::Length(key_width as u16),
            Constraint::Min(min_summary_width as u16),
            Constraint::Length(status_width as u16),
            Constraint::Length(assignee_width as u16),
            Constraint::Length(priority_width as u16),
        ];

        // Create table
        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::NONE))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    /// Render the status bar.
    pub fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let profile_text = self
            .profile_name
            .as_deref()
            .unwrap_or("No profile");

        let issue_count_text = if self.loading {
            "Loading...".to_string()
        } else {
            format!("{} issues", self.issues.len())
        };

        let selected_text = if !self.issues.is_empty() {
            format!(" [{}/{}]", self.selected + 1, self.issues.len())
        } else {
            String::new()
        };

        let mut spans = vec![
            Span::styled(
                format!(" {} ", profile_text),
                Style::default().fg(Color::Black).bg(Color::Cyan),
            ),
            Span::raw(" "),
            Span::styled(issue_count_text, Style::default().fg(Color::Gray)),
            Span::styled(selected_text, Style::default().fg(Color::DarkGray)),
        ];

        // Add filter summary if active
        if let Some(summary) = &self.filter_summary {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("[Filter: {}]", summary),
                Style::default().fg(Color::Yellow),
            ));
        }

        spans.push(Span::raw(" | "));
        spans.push(Span::styled(
            "j/k:navigate  Enter:open  r:refresh  f:filter  :/:jql  p:profile  ?:help",
            Style::default().fg(Color::DarkGray),
        ));

        let status_line = Line::from(spans);
        let paragraph = Paragraph::new(status_line);
        frame.render_widget(paragraph, area);
    }
}

impl Default for ListView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{IssueFields, IssueType, Status};

    fn create_test_issue(key: &str, summary: &str) -> Issue {
        Issue {
            id: "1".to_string(),
            key: key.to_string(),
            self_url: "https://example.com".to_string(),
            fields: IssueFields {
                summary: summary.to_string(),
                description: None,
                status: Status {
                    id: "1".to_string(),
                    name: "Open".to_string(),
                    status_category: None,
                },
                issuetype: IssueType {
                    id: "1".to_string(),
                    name: "Bug".to_string(),
                    subtask: false,
                    description: None,
                    icon_url: None,
                },
                priority: None,
                assignee: None,
                reporter: None,
                project: None,
                labels: vec![],
                components: vec![],
                created: None,
                updated: None,
                duedate: None,
                story_points: None,
            },
        }
    }

    #[test]
    fn test_new_list_view() {
        let view = ListView::new();
        assert_eq!(view.selected, 0);
        assert!(view.issues.is_empty());
        assert!(!view.loading);
    }

    #[test]
    fn test_set_issues() {
        let mut view = ListView::new();
        let issues = vec![
            create_test_issue("TEST-1", "First issue"),
            create_test_issue("TEST-2", "Second issue"),
        ];
        view.set_issues(issues);

        assert_eq!(view.issue_count(), 2);
        assert_eq!(view.selected, 0);
        assert!(!view.loading);
    }

    #[test]
    fn test_move_down() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
            create_test_issue("TEST-3", "Third"),
        ]);

        assert_eq!(view.selected, 0);
        view.move_down();
        assert_eq!(view.selected, 1);
        view.move_down();
        assert_eq!(view.selected, 2);
        // Should not go past the end
        view.move_down();
        assert_eq!(view.selected, 2);
    }

    #[test]
    fn test_move_up() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
        ]);

        view.selected = 1;
        view.move_up();
        assert_eq!(view.selected, 0);
        // Should not go below 0
        view.move_up();
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_move_to_start_and_end() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
            create_test_issue("TEST-3", "Third"),
        ]);

        view.move_to_end();
        assert_eq!(view.selected, 2);

        view.move_to_start();
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_page_navigation() {
        let mut view = ListView::new();
        let issues: Vec<Issue> = (0..25)
            .map(|i| create_test_issue(&format!("TEST-{}", i), &format!("Issue {}", i)))
            .collect();
        view.set_issues(issues);

        // Start at 0
        assert_eq!(view.selected, 0);

        // Page down (10 items)
        view.page_down();
        assert_eq!(view.selected, 10);

        // Page down again
        view.page_down();
        assert_eq!(view.selected, 20);

        // Page down - should stop at last item (24)
        view.page_down();
        assert_eq!(view.selected, 24);

        // Page up
        view.page_up();
        assert_eq!(view.selected, 14);
    }

    #[test]
    fn test_handle_input_navigation() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
        ]);

        // j key moves down
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert_eq!(view.selected, 1);

        // k key moves up
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert_eq!(view.selected, 0);

        // Down arrow moves down
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert_eq!(view.selected, 1);
    }

    #[test]
    fn test_handle_input_gg_navigation() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
            create_test_issue("TEST-3", "Third"),
        ]);

        // Move to end
        view.move_to_end();
        assert_eq!(view.selected, 2);

        // First 'g' should set pending
        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert!(view.pending_g);

        // Second 'g' should move to start
        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert_eq!(view.selected, 0);
        assert!(!view.pending_g);
    }

    #[test]
    fn test_handle_input_shift_g() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
            create_test_issue("TEST-3", "Third"),
        ]);

        // G (shift+g) should move to end
        let key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert_eq!(view.selected, 2);
    }

    #[test]
    fn test_handle_input_enter() {
        let mut view = ListView::new();
        view.set_issues(vec![create_test_issue("TEST-1", "First issue")]);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert_eq!(action, Some(ListAction::OpenIssue("TEST-1".to_string())));
    }

    #[test]
    fn test_handle_input_refresh() {
        let mut view = ListView::new();

        let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert_eq!(action, Some(ListAction::Refresh));
    }

    #[test]
    fn test_handle_input_filter() {
        let mut view = ListView::new();

        let key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert_eq!(action, Some(ListAction::OpenFilter));
    }

    #[test]
    fn test_selected_issue() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
        ]);

        assert_eq!(view.selected_issue().unwrap().key, "TEST-1");

        view.move_down();
        assert_eq!(view.selected_issue().unwrap().key, "TEST-2");
    }

    #[test]
    fn test_loading_state() {
        let mut view = ListView::new();
        assert!(!view.is_loading());

        view.set_loading(true);
        assert!(view.is_loading());

        view.set_loading(false);
        assert!(!view.is_loading());
    }

    #[test]
    fn test_profile_name() {
        let mut view = ListView::new();
        assert!(view.profile_name.is_none());

        view.set_profile_name(Some("work".to_string()));
        assert_eq!(view.profile_name, Some("work".to_string()));
    }

    #[test]
    fn test_empty_list_navigation() {
        let mut view = ListView::new();
        // These should not panic on empty list
        view.move_down();
        view.move_up();
        view.move_to_start();
        view.move_to_end();
        view.page_down();
        view.page_up();
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_handle_input_jql_colon() {
        let mut view = ListView::new();

        // ':' key opens JQL input
        let key = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert_eq!(action, Some(ListAction::OpenJqlInput));
    }

    #[test]
    fn test_handle_input_jql_slash() {
        let mut view = ListView::new();

        // '/' key opens JQL input
        let key = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert_eq!(action, Some(ListAction::OpenJqlInput));
    }
}
