//! Main application state and event loop.
//!
//! This module implements The Elm Architecture (TEA) pattern for predictable
//! state management in the TUI application.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::api::types::Issue;
use crate::events::Event;
use crate::ui::{DetailAction, DetailView, ListAction, ListView};

/// The current view/screen state of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppState {
    /// Application is loading initial data.
    #[default]
    Loading,
    /// Displaying the list of issues.
    IssueList,
    /// Displaying details of a single issue.
    IssueDetail,
    /// Profile selection/management screen.
    ProfileSelect,
    /// Filter panel is open.
    FilterPanel,
    /// Help screen is displayed.
    Help,
    /// Application is in the process of exiting.
    Exiting,
}

/// The main application struct that holds all state.
///
/// This implements the Model part of The Elm Architecture (TEA).
pub struct App {
    /// The current view state.
    state: AppState,
    /// Whether the application should quit.
    should_quit: bool,
    /// The issue list view.
    list_view: ListView,
    /// The issue detail view.
    detail_view: DetailView,
    /// The currently selected issue key (for detail view).
    selected_issue_key: Option<String>,
}

impl App {
    /// Create a new application instance.
    pub fn new() -> Self {
        let mut list_view = ListView::new();
        list_view.set_loading(true);
        Self {
            state: AppState::Loading,
            should_quit: false,
            list_view,
            detail_view: DetailView::new(),
            selected_issue_key: None,
        }
    }

    /// Get a mutable reference to the list view.
    pub fn list_view_mut(&mut self) -> &mut ListView {
        &mut self.list_view
    }

    /// Get a reference to the list view.
    pub fn list_view(&self) -> &ListView {
        &self.list_view
    }

    /// Get the currently selected issue key.
    pub fn selected_issue_key(&self) -> Option<&String> {
        self.selected_issue_key.as_ref()
    }

    /// Get a mutable reference to the detail view.
    pub fn detail_view_mut(&mut self) -> &mut DetailView {
        &mut self.detail_view
    }

    /// Get a reference to the detail view.
    pub fn detail_view(&self) -> &DetailView {
        &self.detail_view
    }

    /// Set the selected issue for the detail view.
    ///
    /// This method is called when an issue is selected from the list view
    /// to populate the detail view with the full issue data.
    pub fn set_detail_issue(&mut self, issue: Issue) {
        self.selected_issue_key = Some(issue.key.clone());
        self.detail_view.set_issue(issue);
    }

    /// Returns whether the application should quit.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Returns the current application state.
    pub fn state(&self) -> AppState {
        self.state
    }

    /// Update the application state based on an event.
    ///
    /// This implements the Update part of The Elm Architecture (TEA).
    /// All state changes flow through this method for predictable behavior.
    pub fn update(&mut self, event: Event) {
        match event {
            Event::Quit => {
                self.should_quit = true;
                self.state = AppState::Exiting;
            }
            Event::Key(key_event) => {
                self.handle_key_event(key_event);
            }
            Event::Resize(_, _) => {
                // Terminal resize is handled automatically by ratatui
            }
            Event::Tick => {
                // Handle periodic tick for animations or background tasks
                self.handle_tick();
            }
        }
    }

    /// Handle keyboard input events.
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Global key bindings (always available)
        match (key_event.code, key_event.modifiers) {
            // Quit on Ctrl+C (always works)
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                self.state = AppState::Exiting;
                return;
            }
            // Help on '?' (unless in detail view where we handle it there)
            (KeyCode::Char('?'), KeyModifiers::NONE) if self.state != AppState::IssueDetail => {
                if self.state != AppState::Help {
                    self.state = AppState::Help;
                }
                return;
            }
            _ => {}
        }

        // State-specific key handling
        match self.state {
            AppState::IssueList | AppState::Loading => {
                // Handle 'q' to quit only in list view
                if key_event.code == KeyCode::Char('q') && key_event.modifiers == KeyModifiers::NONE
                {
                    self.should_quit = true;
                    self.state = AppState::Exiting;
                    return;
                }

                if let Some(action) = self.list_view.handle_input(key_event) {
                    match action {
                        ListAction::OpenIssue(key) => {
                            // Find the issue in the list and set it in detail view
                            if let Some(issue) = self
                                .list_view
                                .selected_issue()
                                .filter(|i| i.key == key)
                                .cloned()
                            {
                                self.set_detail_issue(issue);
                            } else {
                                self.selected_issue_key = Some(key);
                            }
                            self.state = AppState::IssueDetail;
                        }
                        ListAction::Refresh => {
                            self.list_view.set_loading(true);
                            // TODO: Trigger async refresh
                        }
                        ListAction::OpenFilter => {
                            self.state = AppState::FilterPanel;
                        }
                    }
                }
            }
            AppState::IssueDetail => {
                // Handle detail view input
                if let Some(action) = self.detail_view.handle_input(key_event) {
                    match action {
                        DetailAction::GoBack => {
                            self.state = AppState::IssueList;
                            self.detail_view.clear();
                        }
                        DetailAction::EditIssue => {
                            // TODO: Implement editing in Phase 3
                        }
                        DetailAction::AddComment => {
                            // TODO: Implement commenting in Phase 3
                        }
                    }
                }
            }
            AppState::Help => {
                // Escape or 'q' to close help
                if key_event.code == KeyCode::Esc
                    || (key_event.code == KeyCode::Char('q')
                        && key_event.modifiers == KeyModifiers::NONE)
                {
                    self.state = AppState::IssueList;
                }
            }
            AppState::FilterPanel => {
                if key_event.code == KeyCode::Esc {
                    self.state = AppState::IssueList;
                }
            }
            AppState::ProfileSelect => {
                if key_event.code == KeyCode::Esc {
                    self.state = AppState::IssueList;
                }
            }
            AppState::Exiting => {
                // No input handling while exiting
            }
        }
    }

    /// Handle periodic tick events.
    fn handle_tick(&mut self) {
        // Transition from Loading to IssueList after initial setup
        if self.state == AppState::Loading {
            self.state = AppState::IssueList;
        }
    }

    /// Render the application UI.
    ///
    /// This implements the View part of The Elm Architecture (TEA).
    /// The view is a pure function of the current state.
    pub fn view(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Create the main layout with header, content, and footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(1),    // Content
                Constraint::Length(1), // Footer/Status bar
            ])
            .split(area);

        // Render header
        self.render_header(frame, chunks[0]);

        // Render main content based on current state
        self.render_content(frame, chunks[1]);

        // Render footer/status bar
        self.render_footer(frame, chunks[2]);
    }

    /// Render the application header.
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let title = Paragraph::new("LazyJira")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(title, area);
    }

    /// Render the main content area based on current state.
    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        match self.state {
            AppState::Loading | AppState::IssueList => {
                // Use the ListView for both loading and issue list states
                self.list_view.render(frame, area);
            }
            AppState::IssueDetail => {
                // Use the DetailView for issue detail state
                self.detail_view.render(frame, area);
            }
            _ => {
                // For other states, use the placeholder rendering
                let content = match self.state {
                    AppState::ProfileSelect => self.render_profile_select_view(),
                    AppState::FilterPanel => self.render_filter_panel_view(),
                    AppState::Help => self.render_help_view(),
                    AppState::Exiting => self.render_exiting_view(),
                    _ => vec![],
                };

                let block = Block::default().borders(Borders::NONE);

                let paragraph = Paragraph::new(content)
                    .block(block)
                    .alignment(Alignment::Center);

                frame.render_widget(paragraph, area);
            }
        }
    }

    /// Render the footer/status bar.
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        match self.state {
            AppState::Loading | AppState::IssueList => {
                // Use ListView's status bar
                self.list_view.render_status_bar(frame, area);
            }
            AppState::IssueDetail => {
                // Use DetailView's status bar
                self.detail_view.render_status_bar(frame, area);
            }
            _ => {
                // Default status bar for other states
                let state_str = match self.state {
                    AppState::ProfileSelect => "Profile Select",
                    AppState::FilterPanel => "Filter Panel",
                    AppState::Help => "Help",
                    AppState::Exiting => "Exiting...",
                    _ => "",
                };

                let footer = Line::from(vec![
                    Span::styled(
                        format!(" {} ", state_str),
                        Style::default().fg(Color::Black).bg(Color::Cyan),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        "Press 'q' to quit, '?' for help, Esc to go back",
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                let paragraph = Paragraph::new(footer);
                frame.render_widget(paragraph, area);
            }
        }
    }

    /// Render profile select view content (placeholder).
    fn render_profile_select_view(&self) -> Vec<Line<'static>> {
        vec![
            Line::raw(""),
            Line::styled("Profile Select", Style::default().fg(Color::Green)),
            Line::raw(""),
            Line::styled(
                "No profiles configured yet.",
                Style::default().fg(Color::DarkGray),
            ),
        ]
    }

    /// Render filter panel view content (placeholder).
    fn render_filter_panel_view(&self) -> Vec<Line<'static>> {
        vec![
            Line::raw(""),
            Line::styled("Filter Panel", Style::default().fg(Color::Green)),
            Line::raw(""),
            Line::styled(
                "Filter options will appear here.",
                Style::default().fg(Color::DarkGray),
            ),
        ]
    }

    /// Render help view content.
    fn render_help_view(&self) -> Vec<Line<'static>> {
        vec![
            Line::raw(""),
            Line::styled("Help", Style::default().fg(Color::Cyan)),
            Line::raw(""),
            Line::styled("Global:", Style::default().fg(Color::Yellow)),
            Line::raw("  Ctrl+C  - Quit application"),
            Line::raw("  ?       - Show this help"),
            Line::raw(""),
            Line::styled("Issue List:", Style::default().fg(Color::Yellow)),
            Line::raw("  j / ↓   - Move down"),
            Line::raw("  k / ↑   - Move up"),
            Line::raw("  gg      - Go to first issue"),
            Line::raw("  G       - Go to last issue"),
            Line::raw("  Ctrl+d  - Page down"),
            Line::raw("  Ctrl+u  - Page up"),
            Line::raw("  Enter   - Open issue details"),
            Line::raw("  r       - Refresh issues"),
            Line::raw("  f       - Open filter panel"),
            Line::raw("  q       - Quit application"),
            Line::raw(""),
            Line::styled("Issue Detail:", Style::default().fg(Color::Yellow)),
            Line::raw("  j / ↓   - Scroll down"),
            Line::raw("  k / ↑   - Scroll up"),
            Line::raw("  g       - Go to top"),
            Line::raw("  G       - Go to bottom"),
            Line::raw("  Ctrl+d  - Page down"),
            Line::raw("  Ctrl+u  - Page up"),
            Line::raw("  q / Esc - Go back to list"),
            Line::raw("  e       - Edit issue (coming soon)"),
            Line::raw("  c       - Add comment (coming soon)"),
            Line::raw(""),
            Line::styled(
                "Press Esc or q to close this help screen",
                Style::default().fg(Color::DarkGray),
            ),
        ]
    }

    /// Render exiting view content.
    fn render_exiting_view(&self) -> Vec<Line<'static>> {
        vec![
            Line::raw(""),
            Line::styled("Goodbye!", Style::default().fg(Color::Green)),
        ]
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{Issue, IssueFields, IssueType, Status};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    fn test_app_new() {
        let app = App::new();
        assert_eq!(app.state(), AppState::Loading);
        assert!(!app.should_quit());
        assert!(app.list_view.is_loading());
    }

    #[test]
    fn test_app_default() {
        let app = App::default();
        assert_eq!(app.state(), AppState::Loading);
        assert!(!app.should_quit());
    }

    #[test]
    fn test_quit_on_q_key() {
        let mut app = App::new();
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        app.update(Event::Key(key_event));
        assert!(app.should_quit());
        assert_eq!(app.state(), AppState::Exiting);
    }

    #[test]
    fn test_quit_on_ctrl_c() {
        let mut app = App::new();
        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        app.update(Event::Key(key_event));
        assert!(app.should_quit());
        assert_eq!(app.state(), AppState::Exiting);
    }

    #[test]
    fn test_help_on_question_mark() {
        let mut app = App::new();
        // First transition to IssueList via tick
        app.update(Event::Tick);
        assert_eq!(app.state(), AppState::IssueList);

        // Then open help
        let key_event = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE);
        app.update(Event::Key(key_event));
        assert_eq!(app.state(), AppState::Help);
    }

    #[test]
    fn test_escape_closes_help() {
        let mut app = App::new();
        app.update(Event::Tick); // Transition to IssueList

        // Open help
        let key_event = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE);
        app.update(Event::Key(key_event));
        assert_eq!(app.state(), AppState::Help);

        // Close help with Escape
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.update(Event::Key(key_event));
        assert_eq!(app.state(), AppState::IssueList);
    }

    #[test]
    fn test_tick_transitions_from_loading() {
        let mut app = App::new();
        assert_eq!(app.state(), AppState::Loading);
        app.update(Event::Tick);
        assert_eq!(app.state(), AppState::IssueList);
    }

    #[test]
    fn test_quit_event() {
        let mut app = App::new();
        app.update(Event::Quit);
        assert!(app.should_quit());
        assert_eq!(app.state(), AppState::Exiting);
    }

    #[test]
    fn test_resize_event() {
        let mut app = App::new();
        let initial_state = app.state();
        app.update(Event::Resize(100, 50));
        // Resize should not change state
        assert_eq!(app.state(), initial_state);
        assert!(!app.should_quit());
    }

    #[test]
    fn test_list_view_navigation() {
        let mut app = App::new();
        app.update(Event::Tick); // Transition to IssueList

        // Add some issues
        app.list_view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
        ]);

        // Navigate down
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        app.update(Event::Key(key));
        assert_eq!(app.list_view.selected_index(), 1);

        // Navigate up
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        app.update(Event::Key(key));
        assert_eq!(app.list_view.selected_index(), 0);
    }

    #[test]
    fn test_open_issue_detail() {
        let mut app = App::new();
        app.update(Event::Tick); // Transition to IssueList

        app.list_view
            .set_issues(vec![create_test_issue("TEST-123", "Test issue")]);

        // Press Enter to open issue detail
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.update(Event::Key(key));

        assert_eq!(app.state(), AppState::IssueDetail);
        assert_eq!(app.selected_issue_key(), Some(&"TEST-123".to_string()));
        // Verify the issue was set in the detail view
        assert!(app.detail_view().issue().is_some());
        assert_eq!(app.detail_view().issue().unwrap().key, "TEST-123");
    }

    #[test]
    fn test_escape_from_detail() {
        let mut app = App::new();
        app.update(Event::Tick); // Transition to IssueList

        app.list_view
            .set_issues(vec![create_test_issue("TEST-1", "Test")]);

        // Open issue detail
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.update(Event::Key(key));
        assert_eq!(app.state(), AppState::IssueDetail);

        // Press Esc to go back
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.update(Event::Key(key));
        assert_eq!(app.state(), AppState::IssueList);
        // Detail view should be cleared
        assert!(app.detail_view().issue().is_none());
    }

    #[test]
    fn test_q_from_detail_goes_back() {
        let mut app = App::new();
        app.update(Event::Tick); // Transition to IssueList

        app.list_view
            .set_issues(vec![create_test_issue("TEST-1", "Test")]);

        // Open issue detail
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.update(Event::Key(key));
        assert_eq!(app.state(), AppState::IssueDetail);

        // Press 'q' to go back (not quit, since we're in detail view)
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        app.update(Event::Key(key));
        assert_eq!(app.state(), AppState::IssueList);
        assert!(!app.should_quit()); // Should not quit, just go back
    }

    #[test]
    fn test_detail_view_scroll() {
        let mut app = App::new();
        app.update(Event::Tick); // Transition to IssueList

        app.list_view
            .set_issues(vec![create_test_issue("TEST-1", "Test")]);

        // Open issue detail
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.update(Event::Key(key));

        // Set max_scroll so we can scroll
        app.detail_view_mut().set_max_scroll(10);

        // Scroll down with 'j'
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        app.update(Event::Key(key));
        assert_eq!(app.detail_view().scroll(), 1);

        // Scroll up with 'k'
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        app.update(Event::Key(key));
        assert_eq!(app.detail_view().scroll(), 0);
    }

    #[test]
    fn test_detail_view_accessors() {
        let mut app = App::new();
        let issue = create_test_issue("TEST-1", "Test issue");

        app.set_detail_issue(issue);

        assert!(app.detail_view().issue().is_some());
        assert_eq!(app.detail_view().issue().unwrap().key, "TEST-1");
        assert_eq!(app.selected_issue_key(), Some(&"TEST-1".to_string()));
    }

    #[test]
    fn test_open_filter_panel() {
        let mut app = App::new();
        app.update(Event::Tick); // Transition to IssueList

        // Press 'f' to open filter panel
        let key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
        app.update(Event::Key(key));

        assert_eq!(app.state(), AppState::FilterPanel);
    }

    #[test]
    fn test_refresh_sets_loading() {
        let mut app = App::new();
        app.update(Event::Tick); // Transition to IssueList

        // Clear loading state first
        app.list_view.set_loading(false);
        assert!(!app.list_view.is_loading());

        // Press 'r' to refresh
        let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        app.update(Event::Key(key));

        assert!(app.list_view.is_loading());
    }

    #[test]
    fn test_list_view_accessors() {
        let mut app = App::new();

        // Test mutable accessor
        app.list_view_mut().set_profile_name(Some("work".to_string()));

        // Test immutable accessor
        assert_eq!(app.list_view().issue_count(), 0);
    }
}
