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

use crate::events::Event;

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
}

impl App {
    /// Create a new application instance.
    pub fn new() -> Self {
        Self {
            state: AppState::Loading,
            should_quit: false,
        }
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

        match (key_event.code, key_event.modifiers) {
            // Quit on 'q' or Ctrl+C
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.should_quit = true;
                self.state = AppState::Exiting;
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                self.state = AppState::Exiting;
            }
            // Help on '?'
            (KeyCode::Char('?'), KeyModifiers::NONE) => {
                if self.state != AppState::Help {
                    self.state = AppState::Help;
                }
            }
            // Escape to go back or close help
            (KeyCode::Esc, KeyModifiers::NONE) => {
                match self.state {
                    AppState::Help => {
                        // Go back to previous state (default to IssueList for now)
                        self.state = AppState::IssueList;
                    }
                    AppState::IssueDetail => {
                        self.state = AppState::IssueList;
                    }
                    AppState::FilterPanel => {
                        self.state = AppState::IssueList;
                    }
                    AppState::ProfileSelect => {
                        self.state = AppState::IssueList;
                    }
                    _ => {}
                }
            }
            _ => {
                // Other key events will be handled by specific views
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
    pub fn view(&self, frame: &mut Frame) {
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
    fn render_content(&self, frame: &mut Frame, area: Rect) {
        let content = match self.state {
            AppState::Loading => self.render_loading_view(),
            AppState::IssueList => self.render_issue_list_view(),
            AppState::IssueDetail => self.render_issue_detail_view(),
            AppState::ProfileSelect => self.render_profile_select_view(),
            AppState::FilterPanel => self.render_filter_panel_view(),
            AppState::Help => self.render_help_view(),
            AppState::Exiting => self.render_exiting_view(),
        };

        let block = Block::default()
            .borders(Borders::NONE);

        let paragraph = Paragraph::new(content)
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    /// Render the footer/status bar.
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let state_str = match self.state {
            AppState::Loading => "Loading...",
            AppState::IssueList => "Issue List",
            AppState::IssueDetail => "Issue Detail",
            AppState::ProfileSelect => "Profile Select",
            AppState::FilterPanel => "Filter Panel",
            AppState::Help => "Help",
            AppState::Exiting => "Exiting...",
        };

        let footer = Line::from(vec![
            Span::styled(
                format!(" {} ", state_str),
                Style::default().fg(Color::Black).bg(Color::Cyan),
            ),
            Span::raw(" "),
            Span::styled(
                "Press 'q' to quit, '?' for help",
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        let paragraph = Paragraph::new(footer);
        frame.render_widget(paragraph, area);
    }

    /// Render loading view content.
    fn render_loading_view(&self) -> Vec<Line<'static>> {
        vec![
            Line::raw(""),
            Line::styled("Loading...", Style::default().fg(Color::Yellow)),
        ]
    }

    /// Render issue list view content (placeholder).
    fn render_issue_list_view(&self) -> Vec<Line<'static>> {
        vec![
            Line::raw(""),
            Line::styled("Issue List", Style::default().fg(Color::Green)),
            Line::raw(""),
            Line::styled(
                "No issues loaded yet.",
                Style::default().fg(Color::DarkGray),
            ),
        ]
    }

    /// Render issue detail view content (placeholder).
    fn render_issue_detail_view(&self) -> Vec<Line<'static>> {
        vec![
            Line::raw(""),
            Line::styled("Issue Detail", Style::default().fg(Color::Green)),
            Line::raw(""),
            Line::styled(
                "Select an issue to view details.",
                Style::default().fg(Color::DarkGray),
            ),
        ]
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
            Line::styled("Keyboard Shortcuts:", Style::default().fg(Color::Yellow)),
            Line::raw(""),
            Line::raw("  q       - Quit application"),
            Line::raw("  Ctrl+C  - Quit application"),
            Line::raw("  ?       - Show this help"),
            Line::raw("  Esc     - Go back / Close"),
            Line::raw(""),
            Line::styled(
                "Press Esc to close this help screen",
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
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert_eq!(app.state(), AppState::Loading);
        assert!(!app.should_quit());
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
}
