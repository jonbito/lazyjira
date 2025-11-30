//! Main application state and event loop.
//!
//! This module implements The Elm Architecture (TEA) pattern for predictable
//! state management in the TUI application.

use tracing::{debug, info, trace, warn};

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::api::types::Issue;
use crate::config::{Config, ConfigError, Profile};
use crate::error::AppError;
use crate::events::Event;
use crate::ui::{
    DetailAction, DetailView, ErrorDialog, ListAction, ListView, LoadingIndicator,
    Notification, NotificationManager, ProfilePicker, ProfilePickerAction,
};

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
    /// Notification manager for toast messages.
    notifications: NotificationManager,
    /// Error dialog for critical errors.
    error_dialog: ErrorDialog,
    /// Global loading indicator.
    loading: LoadingIndicator,
    /// Application configuration.
    config: Config,
    /// The current active profile.
    current_profile: Option<Profile>,
    /// Profile picker popup.
    profile_picker: ProfilePicker,
}

impl App {
    /// Create a new application instance.
    pub fn new() -> Self {
        debug!("Creating new application instance");

        // Load configuration
        let config = Config::load().unwrap_or_else(|e| {
            warn!("Failed to load config, using default: {}", e);
            Config::default()
        });

        // Get the default profile
        let current_profile = config.get_default_profile().cloned();
        let profile_name = current_profile.as_ref().map(|p| p.name.clone());

        let mut list_view = ListView::new();
        list_view.set_loading(true);
        list_view.set_profile_name(profile_name);

        let mut loading = LoadingIndicator::with_message("Loading issues...");
        loading.start();

        Self {
            state: AppState::Loading,
            should_quit: false,
            list_view,
            detail_view: DetailView::new(),
            selected_issue_key: None,
            notifications: NotificationManager::new(),
            error_dialog: ErrorDialog::new(),
            loading,
            config,
            current_profile,
            profile_picker: ProfilePicker::new(),
        }
    }

    /// Create a new application instance with the given configuration.
    ///
    /// This is useful for testing and for custom initialization.
    pub fn with_config(config: Config) -> Self {
        debug!("Creating application with custom config");

        let current_profile = config.get_default_profile().cloned();
        let profile_name = current_profile.as_ref().map(|p| p.name.clone());

        let mut list_view = ListView::new();
        list_view.set_loading(true);
        list_view.set_profile_name(profile_name);

        let mut loading = LoadingIndicator::with_message("Loading issues...");
        loading.start();

        Self {
            state: AppState::Loading,
            should_quit: false,
            list_view,
            detail_view: DetailView::new(),
            selected_issue_key: None,
            notifications: NotificationManager::new(),
            error_dialog: ErrorDialog::new(),
            loading,
            config,
            current_profile,
            profile_picker: ProfilePicker::new(),
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

    // ========================================================================
    // Notification and error handling methods
    // ========================================================================

    /// Get a reference to the notification manager.
    pub fn notifications(&self) -> &NotificationManager {
        &self.notifications
    }

    /// Get a mutable reference to the notification manager.
    pub fn notifications_mut(&mut self) -> &mut NotificationManager {
        &mut self.notifications
    }

    /// Add an info notification.
    pub fn notify_info(&mut self, message: impl Into<String>) {
        self.notifications.info(message);
    }

    /// Add a success notification.
    pub fn notify_success(&mut self, message: impl Into<String>) {
        self.notifications.success(message);
    }

    /// Add a warning notification.
    pub fn notify_warning(&mut self, message: impl Into<String>) {
        self.notifications.warning(message);
    }

    /// Add an error notification (for non-critical errors).
    pub fn notify_error(&mut self, message: impl Into<String>) {
        self.notifications.error(message);
    }

    /// Handle an application error.
    ///
    /// Critical errors are shown in a modal dialog.
    /// Recoverable errors are shown as toast notifications.
    pub fn handle_error(&mut self, error: &AppError) {
        if error.is_critical() {
            warn!(error = %error, "Critical error occurred");
            self.error_dialog.show(error);
        } else {
            debug!(error = %error, "Recoverable error occurred");
            self.notifications.push(Notification::error(error.user_message()));
        }
    }

    /// Show an error dialog with a custom message.
    pub fn show_error_dialog(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.error_dialog.show_message(title, message);
    }

    /// Dismiss the error dialog.
    pub fn dismiss_error_dialog(&mut self) {
        self.error_dialog.dismiss();
    }

    /// Check if an error dialog is visible.
    pub fn is_error_dialog_visible(&self) -> bool {
        self.error_dialog.is_visible()
    }

    /// Get a reference to the loading indicator.
    pub fn loading(&self) -> &LoadingIndicator {
        &self.loading
    }

    /// Get a mutable reference to the loading indicator.
    pub fn loading_mut(&mut self) -> &mut LoadingIndicator {
        &mut self.loading
    }

    /// Start the loading indicator with a message.
    pub fn start_loading(&mut self, message: impl Into<String>) {
        self.loading.start_with_message(message);
    }

    /// Stop the loading indicator.
    pub fn stop_loading(&mut self) {
        self.loading.stop();
    }

    /// Check if the loading indicator is active.
    pub fn is_loading(&self) -> bool {
        self.loading.is_active()
    }

    // ========================================================================
    // Profile management methods
    // ========================================================================

    /// Get a reference to the current configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the current active profile.
    pub fn current_profile(&self) -> Option<&Profile> {
        self.current_profile.as_ref()
    }

    /// Get the current profile name.
    pub fn current_profile_name(&self) -> Option<&str> {
        self.current_profile.as_ref().map(|p| p.name.as_str())
    }

    /// Check if the profile picker is visible.
    pub fn is_profile_picker_visible(&self) -> bool {
        self.profile_picker.is_visible()
    }

    /// Show the profile picker popup.
    pub fn show_profile_picker(&mut self) {
        let profile_names: Vec<String> = self.config.profiles.iter().map(|p| p.name.clone()).collect();

        if profile_names.is_empty() {
            self.notify_warning("No profiles configured");
            return;
        }

        if profile_names.len() == 1 {
            self.notify_info("Only one profile configured");
            return;
        }

        // Clone the current profile name to avoid borrow conflict
        let current = self
            .current_profile
            .as_ref()
            .map(|p| p.name.as_str())
            .unwrap_or("");
        self.profile_picker.show(profile_names, current);
    }

    /// Switch to a profile by name.
    ///
    /// This clears session data (issue list, client) and sets the new profile.
    /// Returns an error if the profile is not found.
    pub fn switch_profile(&mut self, profile_name: &str) -> Result<(), ConfigError> {
        let profile = self
            .config
            .profiles
            .iter()
            .find(|p| p.name == profile_name)
            .ok_or_else(|| ConfigError::ProfileNotFound(profile_name.to_string()))?
            .clone();

        // Check if we're switching to the same profile
        if self.current_profile.as_ref().map(|p| &p.name) == Some(&profile.name) {
            debug!("Already on profile {}, not switching", profile_name);
            return Ok(());
        }

        info!(profile = %profile_name, "Switching profile");

        // Clear session data
        self.list_view.set_issues(Vec::new());
        self.list_view.set_loading(true);
        self.detail_view.clear();
        self.selected_issue_key = None;

        // Set new profile
        self.current_profile = Some(profile);
        self.list_view
            .set_profile_name(Some(profile_name.to_string()));

        // Notify user
        self.notify_success(format!("Switched to profile: {}", profile_name));

        // Note: The API client will be recreated on the next API call
        // This is handled externally by whatever is managing the client

        Ok(())
    }

    /// Get the number of configured profiles.
    pub fn profile_count(&self) -> usize {
        self.config.profiles.len()
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
                info!("Quit event received");
                self.should_quit = true;
                self.state = AppState::Exiting;
            }
            Event::Key(key_event) => {
                trace!(key = ?key_event.code, modifiers = ?key_event.modifiers, "Key event");
                self.handle_key_event(key_event);
            }
            Event::Resize(width, height) => {
                trace!(width, height, "Terminal resize event");
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

        // Handle error dialog first (blocks all other input)
        if self.error_dialog.is_visible() {
            match key_event.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.error_dialog.dismiss();
                }
                _ => {}
            }
            return;
        }

        // Handle profile picker second (blocks other input when visible)
        if self.profile_picker.is_visible() {
            if let Some(action) = self.profile_picker.handle_input(key_event) {
                match action {
                    ProfilePickerAction::Select(profile_name) => {
                        debug!(profile = %profile_name, "Profile selected");
                        if let Err(e) = self.switch_profile(&profile_name) {
                            self.notify_error(format!("Failed to switch profile: {}", e));
                        }
                    }
                    ProfilePickerAction::Cancel => {
                        debug!("Profile selection cancelled");
                    }
                }
            }
            return;
        }

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
            // Profile switcher on 'p' (available in most views)
            (KeyCode::Char('p'), KeyModifiers::NONE)
                if self.state == AppState::IssueList || self.state == AppState::Loading =>
            {
                debug!("Opening profile picker");
                self.show_profile_picker();
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
                            debug!(issue_key = %key, "Opening issue detail");
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
                            info!("Refreshing issue list");
                            self.list_view.set_loading(true);
                            // TODO: Trigger async refresh
                        }
                        ListAction::OpenFilter => {
                            debug!("Opening filter panel");
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
                            debug!("Going back to issue list");
                            self.state = AppState::IssueList;
                            self.detail_view.clear();
                        }
                        DetailAction::EditIssue => {
                            debug!("Edit issue requested (not yet implemented)");
                            // TODO: Implement editing in Phase 3
                        }
                        DetailAction::AddComment => {
                            debug!("Add comment requested (not yet implemented)");
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
        // Tick animations and timers
        self.loading.tick();
        self.notifications.tick();

        // Transition from Loading to IssueList after initial setup
        if self.state == AppState::Loading {
            debug!("Transitioning from Loading to IssueList");
            self.state = AppState::IssueList;
            self.loading.stop();
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

        // Render notifications (on top of everything except dialogs)
        self.notifications.render(frame, area);

        // Render profile picker (on top of everything except error dialogs)
        self.profile_picker.render(frame, area);

        // Render error dialog (on top of everything)
        self.error_dialog.render(frame, area);
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
            Line::raw("  p       - Switch profile"),
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

    // ========================================================================
    // Notification and error handling tests
    // ========================================================================

    #[test]
    fn test_notify_info() {
        let mut app = App::new();
        app.notify_info("Test info message");
        assert_eq!(app.notifications().len(), 1);
    }

    #[test]
    fn test_notify_success() {
        let mut app = App::new();
        app.notify_success("Operation completed");
        assert_eq!(app.notifications().len(), 1);
    }

    #[test]
    fn test_notify_warning() {
        let mut app = App::new();
        app.notify_warning("Warning message");
        assert_eq!(app.notifications().len(), 1);
    }

    #[test]
    fn test_notify_error() {
        let mut app = App::new();
        app.notify_error("Error message");
        assert_eq!(app.notifications().len(), 1);
    }

    #[test]
    fn test_error_dialog_show_hide() {
        let mut app = App::new();
        assert!(!app.is_error_dialog_visible());

        app.show_error_dialog("Error", "Something went wrong");
        assert!(app.is_error_dialog_visible());

        app.dismiss_error_dialog();
        assert!(!app.is_error_dialog_visible());
    }

    #[test]
    fn test_error_dialog_blocks_input() {
        let mut app = App::new();
        app.update(Event::Tick); // Transition to IssueList
        assert_eq!(app.state(), AppState::IssueList);

        // Show error dialog
        app.show_error_dialog("Error", "Test error");
        assert!(app.is_error_dialog_visible());

        // Try to quit with 'q' - should be blocked by error dialog
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        app.update(Event::Key(key_event));
        assert!(!app.should_quit()); // Should NOT quit
        assert!(app.is_error_dialog_visible()); // Dialog still visible

        // Dismiss with Esc
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.update(Event::Key(key_event));
        assert!(!app.is_error_dialog_visible());
    }

    #[test]
    fn test_error_dialog_dismiss_with_enter() {
        let mut app = App::new();
        app.show_error_dialog("Error", "Test");

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.update(Event::Key(key_event));
        assert!(!app.is_error_dialog_visible());
    }

    #[test]
    fn test_loading_indicator() {
        let mut app = App::new();

        // App starts with loading active
        assert!(app.is_loading());

        // Stop loading
        app.stop_loading();
        assert!(!app.is_loading());

        // Start loading with message
        app.start_loading("Fetching data...");
        assert!(app.is_loading());
    }

    #[test]
    fn test_loading_stops_on_tick() {
        let mut app = App::new();
        assert!(app.is_loading());
        assert_eq!(app.state(), AppState::Loading);

        // Tick should transition state and stop loading
        app.update(Event::Tick);
        assert_eq!(app.state(), AppState::IssueList);
        assert!(!app.is_loading());
    }

    #[test]
    fn test_notifications_tick() {
        let mut app = App::new();
        app.notify_info("Test");
        assert_eq!(app.notifications().len(), 1);

        // Notifications with short duration will be cleared after tick
        // (Our default is 3 seconds, so this test just verifies tick runs)
        app.update(Event::Tick);
        // Notification should still exist (hasn't expired yet)
        assert_eq!(app.notifications().len(), 1);
    }

    #[test]
    fn test_notifications_mut() {
        let mut app = App::new();
        app.notifications_mut().info("Direct access");
        assert_eq!(app.notifications().len(), 1);
    }

    #[test]
    fn test_loading_mut() {
        let mut app = App::new();
        app.loading_mut().set_message("Custom message");
        assert_eq!(app.loading().message(), "Custom message");
    }

    // ========================================================================
    // Profile switching tests
    // ========================================================================

    fn create_test_config_with_profiles() -> Config {
        Config {
            settings: crate::config::Settings {
                default_profile: Some("work".to_string()),
                ..Default::default()
            },
            profiles: vec![
                Profile::new(
                    "work".to_string(),
                    "https://work.atlassian.net".to_string(),
                    "work@example.com".to_string(),
                ),
                Profile::new(
                    "personal".to_string(),
                    "https://personal.atlassian.net".to_string(),
                    "personal@example.com".to_string(),
                ),
                Profile::new(
                    "client".to_string(),
                    "https://client.atlassian.net".to_string(),
                    "client@example.com".to_string(),
                ),
            ],
        }
    }

    #[test]
    fn test_with_config() {
        let config = create_test_config_with_profiles();
        let app = App::with_config(config);

        assert_eq!(app.profile_count(), 3);
        assert_eq!(app.current_profile_name(), Some("work"));
    }

    #[test]
    fn test_current_profile() {
        let config = create_test_config_with_profiles();
        let app = App::with_config(config);

        let profile = app.current_profile().expect("should have current profile");
        assert_eq!(profile.name, "work");
        assert_eq!(profile.url, "https://work.atlassian.net");
    }

    #[test]
    fn test_switch_profile_success() {
        let config = create_test_config_with_profiles();
        let mut app = App::with_config(config);

        // Add some issues to verify clearing
        app.list_view
            .set_issues(vec![create_test_issue("TEST-1", "Test issue")]);
        assert_eq!(app.list_view.issue_count(), 1);

        // Switch profile
        let result = app.switch_profile("personal");
        assert!(result.is_ok());

        // Verify profile changed
        assert_eq!(app.current_profile_name(), Some("personal"));

        // Verify session data cleared
        assert_eq!(app.list_view.issue_count(), 0);
        assert!(app.list_view.is_loading());

        // Verify notification was created
        assert!(app.notifications().len() > 0);
    }

    #[test]
    fn test_switch_profile_not_found() {
        let config = create_test_config_with_profiles();
        let mut app = App::with_config(config);

        let result = app.switch_profile("nonexistent");
        assert!(result.is_err());

        // Profile should not change
        assert_eq!(app.current_profile_name(), Some("work"));
    }

    #[test]
    fn test_switch_to_same_profile() {
        let config = create_test_config_with_profiles();
        let mut app = App::with_config(config);

        // Add some issues
        app.list_view
            .set_issues(vec![create_test_issue("TEST-1", "Test issue")]);
        let initial_notification_count = app.notifications().len();

        // Switch to same profile should be a no-op
        let result = app.switch_profile("work");
        assert!(result.is_ok());

        // Issues should NOT be cleared
        assert_eq!(app.list_view.issue_count(), 1);

        // No new notification
        assert_eq!(app.notifications().len(), initial_notification_count);
    }

    #[test]
    fn test_show_profile_picker_multiple_profiles() {
        let config = create_test_config_with_profiles();
        let mut app = App::with_config(config);

        assert!(!app.is_profile_picker_visible());

        app.show_profile_picker();

        assert!(app.is_profile_picker_visible());
    }

    #[test]
    fn test_show_profile_picker_single_profile() {
        let config = Config {
            settings: crate::config::Settings::default(),
            profiles: vec![Profile::new(
                "only".to_string(),
                "https://only.atlassian.net".to_string(),
                "only@example.com".to_string(),
            )],
        };
        let mut app = App::with_config(config);

        app.show_profile_picker();

        // Picker should not show for single profile
        assert!(!app.is_profile_picker_visible());

        // Should show notification instead
        assert!(app.notifications().len() > 0);
    }

    #[test]
    fn test_show_profile_picker_no_profiles() {
        let config = Config::default();
        let mut app = App::with_config(config);

        app.show_profile_picker();

        // Picker should not show for no profiles
        assert!(!app.is_profile_picker_visible());

        // Should show warning notification
        assert!(app.notifications().len() > 0);
    }

    #[test]
    fn test_p_key_opens_profile_picker() {
        let config = create_test_config_with_profiles();
        let mut app = App::with_config(config);
        app.update(Event::Tick); // Transition to IssueList

        let key_event = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        app.update(Event::Key(key_event));

        assert!(app.is_profile_picker_visible());
    }

    #[test]
    fn test_profile_picker_select() {
        let config = create_test_config_with_profiles();
        let mut app = App::with_config(config);
        app.update(Event::Tick); // Transition to IssueList

        // Open profile picker
        app.show_profile_picker();
        assert!(app.is_profile_picker_visible());

        // Navigate down (from work to personal)
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        app.update(Event::Key(key));

        // Select
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.update(Event::Key(key));

        // Picker should be hidden
        assert!(!app.is_profile_picker_visible());

        // Profile should have switched
        assert_eq!(app.current_profile_name(), Some("personal"));
    }

    #[test]
    fn test_profile_picker_cancel() {
        let config = create_test_config_with_profiles();
        let mut app = App::with_config(config);
        app.update(Event::Tick); // Transition to IssueList

        // Open profile picker
        app.show_profile_picker();
        assert!(app.is_profile_picker_visible());

        // Cancel with Esc
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.update(Event::Key(key));

        // Picker should be hidden
        assert!(!app.is_profile_picker_visible());

        // Profile should NOT have changed
        assert_eq!(app.current_profile_name(), Some("work"));
    }

    #[test]
    fn test_profile_picker_blocks_other_input() {
        let config = create_test_config_with_profiles();
        let mut app = App::with_config(config);
        app.update(Event::Tick); // Transition to IssueList

        // Open profile picker
        app.show_profile_picker();

        // Try to quit - should be blocked
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        app.update(Event::Key(key));

        // Should not quit (q cancels picker instead)
        // Picker is hidden due to q being cancel
        assert!(!app.is_profile_picker_visible());
        // But we should not have quit
        assert_eq!(app.current_profile_name(), Some("work"));
    }

    #[test]
    fn test_profile_clears_detail_view() {
        let config = create_test_config_with_profiles();
        let mut app = App::with_config(config);

        // Set a detail issue
        app.set_detail_issue(create_test_issue("TEST-123", "Detail issue"));
        assert!(app.selected_issue_key().is_some());

        // Switch profile
        app.switch_profile("personal").unwrap();

        // Detail should be cleared
        assert!(app.selected_issue_key().is_none());
    }

    #[test]
    fn test_profile_count() {
        let config = create_test_config_with_profiles();
        let app = App::with_config(config);
        assert_eq!(app.profile_count(), 3);
    }

    #[test]
    fn test_config_accessor() {
        let config = create_test_config_with_profiles();
        let app = App::with_config(config);

        let config = app.config();
        assert_eq!(config.profiles.len(), 3);
        assert_eq!(config.settings.default_profile, Some("work".to_string()));
    }
}
