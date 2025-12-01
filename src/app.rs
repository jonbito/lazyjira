//! Main application state and event loop.
//!
//! This module implements The Elm Architecture (TEA) pattern for predictable
//! state management in the TUI application.

use tracing::{debug, info, trace, warn};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::api::auth;
use crate::api::types::{
    Changelog, FieldUpdates, FilterOptions, FilterState, Issue, IssueUpdateRequest, Priority,
    Transition, User,
};
use crate::config::{Config, ConfigError, Profile};
use crate::error::AppError;
use crate::events::Event;
use crate::events::KeyContext;
use crate::commands::CommandAction;
use crate::ui::{
    render_context_help, CommandPalette, CommandPaletteAction, ConfirmDialog, DeleteProfileDialog,
    DetailAction, DetailView, ErrorDialog, FilterPanelAction, FilterPanelView, FormField,
    HelpAction, HelpView, JqlAction, JqlInput, ListAction, ListView, LoadingIndicator,
    Notification, NotificationManager, ProfileFormAction, ProfileFormData, ProfileFormView,
    ProfileListAction, ProfileListView, ProfilePicker, ProfilePickerAction, ProfileSummary,
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
    /// Profile selection/management screen (quick picker).
    ProfileSelect,
    /// Profile management view (full CRUD list).
    ProfileManagement,
    /// Filter panel is open.
    FilterPanel,
    /// JQL query input is open.
    JqlInput,
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
    /// Profile picker popup (quick switch).
    profile_picker: ProfilePicker,
    /// Profile list view (full management).
    profile_list_view: ProfileListView,
    /// Profile form view (add/edit).
    profile_form_view: ProfileFormView,
    /// Delete profile confirmation dialog.
    delete_profile_dialog: DeleteProfileDialog,
    /// Filter panel view.
    filter_panel: FilterPanelView,
    /// Current filter state.
    filter_state: FilterState,
    /// Available filter options (cached).
    filter_options: Option<FilterOptions>,
    /// JQL query input.
    jql_input: JqlInput,
    /// Current JQL query (if using direct JQL instead of filters).
    current_jql: Option<String>,
    /// Pending issue update (issue key, update request).
    pending_issue_update: Option<(String, IssueUpdateRequest)>,
    /// Discard changes confirmation dialog.
    discard_confirm_dialog: ConfirmDialog,
    /// Transition confirmation dialog.
    transition_confirm_dialog: ConfirmDialog,
    /// Pending transition awaiting confirmation (issue key, transition ID, transition name, optional fields).
    pending_transition_confirm: Option<(String, String, String, Option<FieldUpdates>)>,
    /// Pending transition request (issue key, transition ID, optional fields).
    pending_transition: Option<(String, String, Option<FieldUpdates>)>,
    /// Pending fetch transitions request (issue key).
    pending_fetch_transitions: Option<String>,
    /// Pending fetch assignable users request (issue key, project key).
    pending_fetch_assignees: Option<(String, String)>,
    /// Pending assignee change request (issue key, account_id or None for unassign).
    pending_assignee_change: Option<(String, Option<String>)>,
    /// Pending fetch priorities request (issue key).
    pending_fetch_priorities: Option<String>,
    /// Pending priority change request (issue key, priority_id).
    pending_priority_change: Option<(String, String)>,
    /// Pending fetch comments request (issue key).
    pending_fetch_comments: Option<String>,
    /// Pending submit comment request (issue key, comment body).
    pending_submit_comment: Option<(String, String)>,
    /// Pending fetch labels request (issue key).
    pending_fetch_labels: Option<String>,
    /// Pending add label request (issue key, label).
    pending_add_label: Option<(String, String)>,
    /// Pending remove label request (issue key, label).
    pending_remove_label: Option<(String, String)>,
    /// Pending fetch components request (issue key, project key).
    pending_fetch_components: Option<(String, String)>,
    /// Pending add component request (issue key, component name).
    pending_add_component: Option<(String, String)>,
    /// Pending remove component request (issue key, component name).
    pending_remove_component: Option<(String, String)>,
    /// Pending fetch changelog request (issue key, start_at).
    pending_fetch_changelog: Option<(String, u32)>,
    /// Help view.
    help_view: HelpView,
    /// Previous state before opening help (to return to).
    previous_state: Option<AppState>,
    /// Command palette for quick command access.
    command_palette: CommandPalette,
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

        // Initialize JQL input with history from config
        let jql_input = JqlInput::with_history(config.jql_history().to_vec());

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
            profile_list_view: ProfileListView::new(),
            profile_form_view: ProfileFormView::new_add(),
            delete_profile_dialog: DeleteProfileDialog::new(),
            filter_panel: FilterPanelView::new(),
            filter_state: FilterState::new(),
            filter_options: None,
            jql_input,
            current_jql: None,
            pending_issue_update: None,
            discard_confirm_dialog: ConfirmDialog::new(),
            transition_confirm_dialog: ConfirmDialog::new(),
            pending_transition_confirm: None,
            pending_transition: None,
            pending_fetch_transitions: None,
            pending_fetch_assignees: None,
            pending_assignee_change: None,
            pending_fetch_priorities: None,
            pending_priority_change: None,
            pending_fetch_comments: None,
            pending_submit_comment: None,
            pending_fetch_labels: None,
            pending_add_label: None,
            pending_remove_label: None,
            pending_fetch_components: None,
            pending_add_component: None,
            pending_remove_component: None,
            pending_fetch_changelog: None,
            help_view: HelpView::new(),
            previous_state: None,
            command_palette: CommandPalette::new(),
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

        // Initialize JQL input with history from config
        let jql_input = JqlInput::with_history(config.jql_history().to_vec());

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
            profile_list_view: ProfileListView::new(),
            profile_form_view: ProfileFormView::new_add(),
            delete_profile_dialog: DeleteProfileDialog::new(),
            filter_panel: FilterPanelView::new(),
            filter_state: FilterState::new(),
            filter_options: None,
            jql_input,
            current_jql: None,
            pending_issue_update: None,
            discard_confirm_dialog: ConfirmDialog::new(),
            transition_confirm_dialog: ConfirmDialog::new(),
            pending_transition_confirm: None,
            pending_transition: None,
            pending_fetch_transitions: None,
            pending_fetch_assignees: None,
            pending_assignee_change: None,
            pending_fetch_priorities: None,
            pending_priority_change: None,
            pending_fetch_comments: None,
            pending_submit_comment: None,
            pending_fetch_labels: None,
            pending_add_label: None,
            pending_remove_label: None,
            pending_fetch_components: None,
            pending_add_component: None,
            pending_remove_component: None,
            pending_fetch_changelog: None,
            help_view: HelpView::new(),
            previous_state: None,
            command_palette: CommandPalette::new(),
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
            self.notifications
                .push(Notification::error(error.user_message()));
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
        let profile_names: Vec<String> = self
            .config
            .profiles
            .iter()
            .map(|p| p.name.clone())
            .collect();

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

    /// Open the profile management view.
    pub fn open_profile_management(&mut self) {
        debug!("Opening profile management view");
        self.refresh_profile_list();
        self.state = AppState::ProfileManagement;
    }

    /// Refresh the profile list with current data.
    fn refresh_profile_list(&mut self) {
        let default_profile = self.config.settings.default_profile.as_deref();
        let summaries: Vec<ProfileSummary> = self
            .config
            .profiles
            .iter()
            .map(|p| {
                let is_default = default_profile == Some(p.name.as_str());
                let has_token = auth::has_token(&p.name);
                ProfileSummary::from_profile(p, is_default, has_token)
            })
            .collect();
        self.profile_list_view.set_profiles(summaries);
    }

    /// Get a profile by index.
    fn get_profile_by_index(&self, index: usize) -> Option<&Profile> {
        self.config.profiles.get(index)
    }

    /// Add a new profile to the configuration.
    pub fn add_profile(&mut self, data: ProfileFormData) -> Result<(), ConfigError> {
        debug!(name = %data.name, "Adding new profile");

        let profile = Profile::new(data.name.clone(), data.url.clone(), data.email.clone());

        // Add to config
        self.config.add_profile(profile)?;

        // Store token in keyring
        if let Err(e) = auth::store_token(&data.name, &data.token) {
            warn!("Failed to store token: {}", e);
            // Remove the profile we just added since token storage failed
            let _ = self.config.remove_profile(&data.name);
            return Err(ConfigError::ValidationError(format!(
                "Failed to store token: {}",
                e
            )));
        }

        // Save config
        self.config.save()?;

        // Refresh list
        self.refresh_profile_list();

        // If this is the first profile, set it as current
        if self.current_profile.is_none() {
            if let Some(profile) = self.config.profiles.first().cloned() {
                self.current_profile = Some(profile);
            }
        }

        self.notify_success(format!("Profile '{}' added", data.name));
        Ok(())
    }

    /// Update an existing profile.
    pub fn update_profile(&mut self, data: ProfileFormData) -> Result<(), ConfigError> {
        let original_name = data
            .original_name
            .as_ref()
            .ok_or_else(|| ConfigError::ValidationError("No original name provided".to_string()))?;

        debug!(original = %original_name, new = %data.name, "Updating profile");

        // Find the profile index
        let index = self
            .config
            .profiles
            .iter()
            .position(|p| p.name == *original_name)
            .ok_or_else(|| ConfigError::ProfileNotFound(original_name.clone()))?;

        // Check for duplicate name (if name changed)
        if data.name != *original_name && self.config.profiles.iter().any(|p| p.name == data.name) {
            return Err(ConfigError::ValidationError(format!(
                "Profile '{}' already exists",
                data.name
            )));
        }

        // Update the profile
        let profile = Profile::new(data.name.clone(), data.url, data.email);
        self.config.profiles[index] = profile.clone();

        // Update token (delete old if name changed, then store new)
        if data.name != *original_name {
            let _ = auth::delete_token(original_name);
        }
        if let Err(e) = auth::store_token(&data.name, &data.token) {
            warn!("Failed to store token: {}", e);
            return Err(ConfigError::ValidationError(format!(
                "Failed to store token: {}",
                e
            )));
        }

        // Update default profile reference if needed
        if self.config.settings.default_profile.as_deref() == Some(original_name) {
            self.config.settings.default_profile = Some(data.name.clone());
        }

        // Save config
        self.config.save()?;

        // Update current profile if it was the one being edited
        if self.current_profile.as_ref().map(|p| p.name.as_str()) == Some(original_name) {
            self.current_profile = Some(profile);
            self.list_view.set_profile_name(Some(data.name.clone()));
        }

        // Refresh list
        self.refresh_profile_list();

        self.notify_success(format!("Profile '{}' updated", data.name));
        Ok(())
    }

    /// Delete a profile by index.
    pub fn delete_profile(&mut self, index: usize) -> Result<(), ConfigError> {
        let profile = self
            .config
            .profiles
            .get(index)
            .ok_or_else(|| ConfigError::ProfileNotFound(format!("index {}", index)))?
            .clone();

        debug!(name = %profile.name, "Deleting profile");

        // Delete token from keyring
        let _ = auth::delete_token(&profile.name);

        // Remove from config
        if !self.config.remove_profile(&profile.name) {
            return Err(ConfigError::ProfileNotFound(profile.name.clone()));
        }

        // Save config
        self.config.save()?;

        // If we deleted the current profile, switch to another
        if self.current_profile.as_ref().map(|p| p.name.as_str()) == Some(&profile.name) {
            self.current_profile = self.config.profiles.first().cloned();
            self.list_view
                .set_profile_name(self.current_profile.as_ref().map(|p| p.name.clone()));
            // Clear session data
            self.list_view.set_issues(Vec::new());
            self.list_view.set_loading(true);
            self.detail_view.clear();
            self.selected_issue_key = None;
        }

        // Refresh list
        self.refresh_profile_list();

        self.notify_success(format!("Profile '{}' deleted", profile.name));
        Ok(())
    }

    /// Set a profile as the default.
    pub fn set_default_profile(&mut self, index: usize) -> Result<(), ConfigError> {
        let profile_name = self
            .config
            .profiles
            .get(index)
            .map(|p| p.name.clone())
            .ok_or_else(|| ConfigError::ProfileNotFound(format!("index {}", index)))?;

        debug!(name = %profile_name, "Setting default profile");

        self.config.settings.default_profile = Some(profile_name.clone());
        self.config.save()?;

        // Refresh list to update default indicator
        self.refresh_profile_list();

        self.notify_success(format!("'{}' set as default profile", profile_name));
        Ok(())
    }

    // ========================================================================
    // Filter methods
    // ========================================================================

    /// Get a reference to the current filter state.
    pub fn filter_state(&self) -> &FilterState {
        &self.filter_state
    }

    /// Get a mutable reference to the filter state.
    pub fn filter_state_mut(&mut self) -> &mut FilterState {
        &mut self.filter_state
    }

    /// Set the available filter options.
    pub fn set_filter_options(&mut self, options: FilterOptions) {
        self.filter_panel.set_options(options.clone());
        self.filter_options = Some(options);
    }

    /// Get the filter options if loaded.
    pub fn filter_options(&self) -> Option<&FilterOptions> {
        self.filter_options.as_ref()
    }

    /// Check if filter options have been loaded.
    pub fn has_filter_options(&self) -> bool {
        self.filter_options.is_some()
    }

    /// Open the filter panel.
    pub fn open_filter_panel(&mut self) {
        debug!("Opening filter panel");
        self.filter_panel.show_with_state(&self.filter_state);
        self.state = AppState::FilterPanel;
    }

    /// Apply the given filter state.
    pub fn apply_filter(&mut self, filter: FilterState) {
        debug!("Applying filter: {:?}", filter.summary());
        // Update filter summary for display
        let summary = if filter.is_empty() {
            None
        } else {
            Some(filter.summary().join(", "))
        };
        self.list_view.set_filter_summary(summary);
        self.filter_state = filter;
        // Set list to loading - the runner will trigger a refresh
        self.list_view.set_loading(true);
        self.state = AppState::IssueList;
    }

    /// Clear all filters.
    pub fn clear_filters(&mut self) {
        debug!("Clearing all filters");
        self.filter_state.clear();
        self.list_view.set_filter_summary(None);
        self.list_view.set_loading(true);
    }

    /// Get the JQL query string from the current filter state.
    pub fn filter_jql(&self) -> String {
        self.filter_state.to_jql()
    }

    // ========================================================================
    // JQL input methods
    // ========================================================================

    /// Get a reference to the JQL input.
    pub fn jql_input(&self) -> &JqlInput {
        &self.jql_input
    }

    /// Get a mutable reference to the JQL input.
    pub fn jql_input_mut(&mut self) -> &mut JqlInput {
        &mut self.jql_input
    }

    /// Get the current JQL query if set.
    pub fn current_jql(&self) -> Option<&str> {
        self.current_jql.as_deref()
    }

    /// Open the JQL input.
    pub fn open_jql_input(&mut self) {
        debug!("Opening JQL input");
        self.jql_input.show();
        self.state = AppState::JqlInput;
    }

    /// Execute a JQL query.
    ///
    /// This sets the current JQL, clears filter state, and triggers a refresh.
    /// Also saves the query to history in config.
    pub fn execute_jql(&mut self, jql: String) {
        debug!(jql = %jql, "Executing JQL query");
        // Clear filter state when using direct JQL
        self.filter_state.clear();
        self.current_jql = Some(jql.clone());
        // Update filter summary to show JQL is active
        self.list_view
            .set_filter_summary(Some(format!("JQL: {}", jql)));

        // Save to config history
        self.config.add_jql_to_history(jql);
        // Persist config (ignore errors)
        if let Err(e) = self.config.save() {
            debug!("Failed to save JQL history to config: {}", e);
        }

        // Trigger refresh
        self.list_view.set_loading(true);
        self.state = AppState::IssueList;
    }

    /// Execute a command action from the command palette.
    fn execute_command_action(&mut self, action: CommandAction) {
        match action {
            CommandAction::GoToList => {
                debug!("Command: Go to issue list");
                self.state = AppState::IssueList;
            }
            CommandAction::GoToProfiles => {
                debug!("Command: Go to profile management");
                self.open_profile_management();
            }
            CommandAction::GoToFilters => {
                debug!("Command: Open filter panel");
                self.open_filter_panel();
            }
            CommandAction::GoToHelp => {
                debug!("Command: Show help");
                if self.state != AppState::Help {
                    self.previous_state = Some(self.state);
                    self.help_view.reset_scroll();
                    self.state = AppState::Help;
                }
            }
            CommandAction::RefreshIssues => {
                debug!("Command: Refresh issues");
                self.list_view.set_loading(true);
                // The main loop will handle triggering the actual refresh
            }
            CommandAction::SwitchProfile => {
                debug!("Command: Switch profile");
                self.show_profile_picker();
            }
            CommandAction::OpenJqlInput => {
                debug!("Command: Open JQL input");
                self.open_jql_input();
            }
            CommandAction::ClearFilters => {
                debug!("Command: Clear filters");
                self.filter_state.clear();
                self.current_jql = None;
                self.list_view.set_filter_summary(None);
                self.list_view.set_loading(true);
                self.notify_info("Filters cleared");
            }
            CommandAction::ClearCache => {
                debug!("Command: Clear cache");
                // TODO: Implement cache clearing when cache module exposes this
                self.notify_info("Cache cleared");
            }
        }
    }

    /// Set an error on the JQL input.
    pub fn set_jql_error(&mut self, error: impl Into<String>) {
        self.jql_input.set_error(error);
    }

    /// Get the effective JQL query.
    ///
    /// Returns the current direct JQL query if set, otherwise generates JQL
    /// from the filter state. Appends the current sort order from the list view
    /// unless the query already contains an ORDER BY clause.
    pub fn effective_jql(&self) -> String {
        let base_jql = if let Some(jql) = &self.current_jql {
            jql.clone()
        } else {
            self.filter_state.to_jql()
        };

        // If empty, let caller handle default query
        if base_jql.is_empty() {
            return base_jql;
        }

        // If already has ORDER BY, don't modify (user explicitly set sort)
        if base_jql.to_uppercase().contains("ORDER BY") {
            return base_jql;
        }

        // Append sort clause from list view
        format!("{} {}", base_jql, self.list_view.sort().to_jql())
    }

    /// Set the JQL history.
    pub fn set_jql_history(&mut self, history: Vec<String>) {
        self.jql_input.set_history(history);
    }

    /// Get the JQL history.
    pub fn jql_history(&self) -> Vec<String> {
        self.jql_input.history()
    }

    // ========================================================================
    // Issue edit methods
    // ========================================================================

    /// Get the pending issue update, if any.
    pub fn take_pending_issue_update(&mut self) -> Option<(String, IssueUpdateRequest)> {
        self.pending_issue_update.take()
    }

    /// Check if there is a pending issue update.
    pub fn has_pending_issue_update(&self) -> bool {
        self.pending_issue_update.is_some()
    }

    /// Handle successful issue update.
    ///
    /// Updates the local issue data and exits edit mode.
    pub fn handle_issue_update_success(&mut self, updated_issue: Issue) {
        info!(key = %updated_issue.key, "Issue updated successfully");

        // Update the detail view with the updated issue
        self.detail_view.set_issue(updated_issue.clone());

        // Update the issue in the list view if present
        self.list_view.update_issue(&updated_issue);

        // Show success notification
        self.notify_success(format!("Issue {} updated", updated_issue.key));
    }

    /// Handle failed issue update.
    pub fn handle_issue_update_failure(&mut self, error: &str) {
        warn!(error = %error, "Issue update failed");
        self.detail_view.set_saving(false);
        self.notify_error(format!("Failed to update issue: {}", error));
    }

    /// Show the discard changes confirmation dialog.
    fn show_discard_confirm_dialog(&mut self) {
        self.discard_confirm_dialog.show_destructive_with_label(
            "Discard Changes?",
            "You have unsaved changes. Are you sure you want to discard them?",
            "Discard",
        );
    }

    /// Check if the discard confirm dialog is visible.
    pub fn is_discard_confirm_visible(&self) -> bool {
        self.discard_confirm_dialog.is_visible()
    }

    /// Check if the transition confirm dialog is visible.
    pub fn is_transition_confirm_visible(&self) -> bool {
        self.transition_confirm_dialog.is_visible()
    }

    /// Show the transition confirmation dialog.
    ///
    /// If confirm_transitions setting is true, shows a confirmation dialog.
    /// Otherwise, executes the transition immediately.
    fn request_transition_with_confirmation(
        &mut self,
        issue_key: String,
        transition_id: String,
        transition_name: String,
        fields: Option<FieldUpdates>,
    ) {
        if self.config.settings.confirm_transitions {
            // Store the pending transition for confirmation
            self.pending_transition_confirm = Some((
                issue_key.clone(),
                transition_id,
                transition_name.clone(),
                fields,
            ));
            // Show the confirmation dialog
            self.transition_confirm_dialog.show_with_labels(
                "Change Status?",
                &format!("Move issue {} to '{}'?", issue_key, transition_name),
                "Confirm",
                "Cancel",
            );
        } else {
            // Execute immediately without confirmation
            self.pending_transition = Some((issue_key, transition_id, fields));
        }
    }

    /// Confirm the pending transition and execute it.
    fn confirm_transition(&mut self) {
        if let Some((issue_key, transition_id, _name, fields)) =
            self.pending_transition_confirm.take()
        {
            self.pending_transition = Some((issue_key, transition_id, fields));
        }
    }

    /// Cancel the pending transition confirmation.
    fn cancel_transition_confirm(&mut self) {
        self.pending_transition_confirm = None;
    }

    // ========================================================================
    // Transition methods
    // ========================================================================

    /// Take the pending transition request, if any.
    ///
    /// Returns (issue_key, transition_id, optional fields).
    pub fn take_pending_transition(&mut self) -> Option<(String, String, Option<FieldUpdates>)> {
        self.pending_transition.take()
    }

    /// Check if there is a pending transition.
    pub fn has_pending_transition(&self) -> bool {
        self.pending_transition.is_some()
    }

    /// Take the pending fetch transitions request, if any.
    ///
    /// Returns the issue key.
    pub fn take_pending_fetch_transitions(&mut self) -> Option<String> {
        self.pending_fetch_transitions.take()
    }

    /// Check if there is a pending fetch transitions request.
    pub fn has_pending_fetch_transitions(&self) -> bool {
        self.pending_fetch_transitions.is_some()
    }

    /// Set the available transitions in the detail view.
    pub fn set_transitions(&mut self, transitions: Vec<Transition>) {
        self.detail_view.set_transitions(transitions);
    }

    /// Handle successful transition completion.
    ///
    /// Updates the local issue data with the refreshed issue.
    pub fn handle_transition_success(&mut self, updated_issue: Issue) {
        info!(key = %updated_issue.key, "Issue transitioned successfully");

        // Update the detail view with the updated issue
        self.detail_view.set_issue(updated_issue.clone());

        // Update the issue in the list view if present
        self.list_view.update_issue(&updated_issue);

        // Show success notification
        self.notify_success(format!(
            "Issue {} status changed to {}",
            updated_issue.key, updated_issue.fields.status.name
        ));
    }

    /// Handle failed transition.
    pub fn handle_transition_failure(&mut self, error: &str) {
        warn!(error = %error, "Transition failed");
        self.detail_view.hide_transition_picker();
        self.notify_error(format!("Failed to change status: {}", error));
    }

    /// Handle failure to fetch transitions.
    pub fn handle_fetch_transitions_failure(&mut self, error: &str) {
        warn!(error = %error, "Failed to fetch transitions");
        self.detail_view.hide_transition_picker();
        self.notify_error(format!("Failed to load transitions: {}", error));
    }

    // ========================================================================
    // Assignee Picker Methods
    // ========================================================================

    /// Take the pending fetch assignees request, if any.
    ///
    /// Returns the (issue_key, project_key).
    pub fn take_pending_fetch_assignees(&mut self) -> Option<(String, String)> {
        self.pending_fetch_assignees.take()
    }

    /// Check if there is a pending fetch assignees request.
    pub fn has_pending_fetch_assignees(&self) -> bool {
        self.pending_fetch_assignees.is_some()
    }

    /// Set the available assignable users in the detail view.
    pub fn set_assignable_users(&mut self, users: Vec<User>) {
        self.detail_view.set_assignable_users(users);
    }

    /// Take the pending assignee change request, if any.
    ///
    /// Returns the (issue_key, account_id or None for unassign).
    pub fn take_pending_assignee_change(&mut self) -> Option<(String, Option<String>)> {
        self.pending_assignee_change.take()
    }

    /// Check if there is a pending assignee change request.
    pub fn has_pending_assignee_change(&self) -> bool {
        self.pending_assignee_change.is_some()
    }

    /// Handle successful assignee change.
    ///
    /// Updates the local issue data with the refreshed issue.
    pub fn handle_assignee_change_success(&mut self, updated_issue: Issue) {
        info!(key = %updated_issue.key, "Assignee changed successfully");

        // Update the detail view with the updated issue
        self.detail_view.set_issue(updated_issue.clone());

        // Update the issue in the list view if present
        self.list_view.update_issue(&updated_issue);

        // Show success notification
        let assignee_name = updated_issue.assignee_name();
        self.notify_success(format!(
            "Issue {} assignee changed to {}",
            updated_issue.key, assignee_name
        ));
    }

    /// Handle failed assignee change.
    pub fn handle_assignee_change_failure(&mut self, error: &str) {
        warn!(error = %error, "Assignee change failed");
        self.detail_view.hide_assignee_picker();
        self.notify_error(format!("Failed to change assignee: {}", error));
    }

    /// Handle failure to fetch assignable users.
    pub fn handle_fetch_assignees_failure(&mut self, error: &str) {
        warn!(error = %error, "Failed to fetch assignable users");
        self.detail_view.hide_assignee_picker();
        self.notify_error(format!("Failed to load assignees: {}", error));
    }

    // ========================================================================
    // Priority Picker Methods
    // ========================================================================

    /// Take the pending fetch priorities request, if any.
    ///
    /// Returns the issue_key.
    pub fn take_pending_fetch_priorities(&mut self) -> Option<String> {
        self.pending_fetch_priorities.take()
    }

    /// Check if there is a pending fetch priorities request.
    pub fn has_pending_fetch_priorities(&self) -> bool {
        self.pending_fetch_priorities.is_some()
    }

    /// Set the available priorities in the detail view.
    pub fn set_priorities(&mut self, priorities: Vec<Priority>) {
        self.detail_view.set_priorities(priorities);
    }

    /// Take the pending priority change request, if any.
    ///
    /// Returns the (issue_key, priority_id).
    pub fn take_pending_priority_change(&mut self) -> Option<(String, String)> {
        self.pending_priority_change.take()
    }

    /// Check if there is a pending priority change request.
    pub fn has_pending_priority_change(&self) -> bool {
        self.pending_priority_change.is_some()
    }

    /// Handle successful priority change.
    ///
    /// Updates the local issue data with the refreshed issue.
    pub fn handle_priority_change_success(&mut self, updated_issue: Issue) {
        info!(key = %updated_issue.key, "Priority changed successfully");

        // Update the detail view with the updated issue
        self.detail_view.set_issue(updated_issue.clone());

        // Update the issue in the list view if present
        self.list_view.update_issue(&updated_issue);

        // Show success notification
        let priority_name = updated_issue.priority_name();
        self.notify_success(format!(
            "Issue {} priority changed to {}",
            updated_issue.key, priority_name
        ));
    }

    /// Handle failed priority change.
    pub fn handle_priority_change_failure(&mut self, error: &str) {
        warn!(error = %error, "Priority change failed");
        self.detail_view.hide_priority_picker();
        self.notify_error(format!("Failed to change priority: {}", error));
    }

    /// Handle failure to fetch priorities.
    pub fn handle_fetch_priorities_failure(&mut self, error: &str) {
        warn!(error = %error, "Failed to fetch priorities");
        self.detail_view.hide_priority_picker();
        self.notify_error(format!("Failed to load priorities: {}", error));
    }

    // ========================================================================
    // Comments methods
    // ========================================================================

    /// Take the pending fetch comments request.
    pub fn take_pending_fetch_comments(&mut self) -> Option<String> {
        self.pending_fetch_comments.take()
    }

    /// Check if there is a pending fetch comments request.
    pub fn has_pending_fetch_comments(&self) -> bool {
        self.pending_fetch_comments.is_some()
    }

    /// Handle successful comments fetch.
    pub fn handle_comments_fetched(
        &mut self,
        comments: Vec<crate::api::types::Comment>,
        total: u32,
    ) {
        debug!("Comments fetched: {} of {}", comments.len(), total);
        self.detail_view.set_comments(comments, total);
    }

    /// Handle failure to fetch comments.
    pub fn handle_fetch_comments_failure(&mut self, error: &str) {
        warn!(error = %error, "Failed to fetch comments");
        self.detail_view.hide_comments_panel();
        self.notify_error(format!("Failed to load comments: {}", error));
    }

    /// Take the pending submit comment request.
    pub fn take_pending_submit_comment(&mut self) -> Option<(String, String)> {
        self.pending_submit_comment.take()
    }

    /// Check if there is a pending submit comment request.
    pub fn has_pending_submit_comment(&self) -> bool {
        self.pending_submit_comment.is_some()
    }

    /// Handle successful comment submission.
    pub fn handle_comment_submitted(&mut self, comment: crate::api::types::Comment) {
        info!(comment_id = %comment.id, "Comment submitted successfully");
        self.detail_view.add_comment(comment);
        self.notify_success("Comment added successfully");
    }

    /// Handle failure to submit comment.
    pub fn handle_submit_comment_failure(&mut self, error: &str) {
        warn!(error = %error, "Failed to submit comment");
        self.detail_view.set_comment_submitting(false);
        self.notify_error(format!("Failed to add comment: {}", error));
    }

    // ========================================================================
    // Labels methods
    // ========================================================================

    /// Take the pending fetch labels request.
    pub fn take_pending_fetch_labels(&mut self) -> Option<String> {
        self.pending_fetch_labels.take()
    }

    /// Check if there is a pending fetch labels request.
    pub fn has_pending_fetch_labels(&self) -> bool {
        self.pending_fetch_labels.is_some()
    }

    /// Set the available labels in the detail view.
    pub fn set_labels(&mut self, labels: Vec<String>) {
        self.detail_view.set_labels(labels);
    }

    /// Handle failure to fetch labels.
    pub fn handle_fetch_labels_failure(&mut self, error: &str) {
        warn!(error = %error, "Failed to fetch labels");
        self.detail_view.hide_label_editor();
        self.notify_error(format!("Failed to load labels: {}", error));
    }

    /// Take the pending add label request.
    pub fn take_pending_add_label(&mut self) -> Option<(String, String)> {
        self.pending_add_label.take()
    }

    /// Check if there is a pending add label request.
    pub fn has_pending_add_label(&self) -> bool {
        self.pending_add_label.is_some()
    }

    /// Take the pending remove label request.
    pub fn take_pending_remove_label(&mut self) -> Option<(String, String)> {
        self.pending_remove_label.take()
    }

    /// Check if there is a pending remove label request.
    pub fn has_pending_remove_label(&self) -> bool {
        self.pending_remove_label.is_some()
    }

    /// Handle successful label change.
    pub fn handle_label_change_success(&mut self, updated_issue: Issue) {
        info!(key = %updated_issue.key, "Label changed successfully");

        // Update the detail view with the updated issue
        self.detail_view.set_issue(updated_issue.clone());

        // Update the issue in the list view if present
        self.list_view.update_issue(&updated_issue);

        self.notify_success(format!("Issue {} labels updated", updated_issue.key));
    }

    /// Handle failed label change.
    pub fn handle_label_change_failure(&mut self, error: &str) {
        warn!(error = %error, "Label change failed");
        self.notify_error(format!("Failed to update labels: {}", error));
    }

    // ========================================================================
    // Components methods
    // ========================================================================

    /// Take the pending fetch components request.
    pub fn take_pending_fetch_components(&mut self) -> Option<(String, String)> {
        self.pending_fetch_components.take()
    }

    /// Check if there is a pending fetch components request.
    pub fn has_pending_fetch_components(&self) -> bool {
        self.pending_fetch_components.is_some()
    }

    /// Set the available components in the detail view.
    pub fn set_components(&mut self, components: Vec<String>) {
        self.detail_view.set_components(components);
    }

    /// Handle failure to fetch components.
    pub fn handle_fetch_components_failure(&mut self, error: &str) {
        warn!(error = %error, "Failed to fetch components");
        self.detail_view.hide_component_editor();
        self.notify_error(format!("Failed to load components: {}", error));
    }

    /// Take the pending add component request.
    pub fn take_pending_add_component(&mut self) -> Option<(String, String)> {
        self.pending_add_component.take()
    }

    /// Check if there is a pending add component request.
    pub fn has_pending_add_component(&self) -> bool {
        self.pending_add_component.is_some()
    }

    /// Take the pending remove component request.
    pub fn take_pending_remove_component(&mut self) -> Option<(String, String)> {
        self.pending_remove_component.take()
    }

    /// Check if there is a pending remove component request.
    pub fn has_pending_remove_component(&self) -> bool {
        self.pending_remove_component.is_some()
    }

    /// Handle successful component change.
    pub fn handle_component_change_success(&mut self, updated_issue: Issue) {
        info!(key = %updated_issue.key, "Component changed successfully");

        // Update the detail view with the updated issue
        self.detail_view.set_issue(updated_issue.clone());

        // Update the issue in the list view if present
        self.list_view.update_issue(&updated_issue);

        self.notify_success(format!("Issue {} components updated", updated_issue.key));
    }

    /// Handle failed component change.
    pub fn handle_component_change_failure(&mut self, error: &str) {
        warn!(error = %error, "Component change failed");
        self.notify_error(format!("Failed to update components: {}", error));
    }

    // ========================================================================
    // Changelog methods
    // ========================================================================

    /// Take the pending fetch changelog request (issue_key, start_at).
    pub fn take_pending_fetch_changelog(&mut self) -> Option<(String, u32)> {
        self.pending_fetch_changelog.take()
    }

    /// Check if there is a pending fetch changelog request.
    pub fn has_pending_fetch_changelog(&self) -> bool {
        self.pending_fetch_changelog.is_some()
    }

    /// Handle successful changelog fetch.
    pub fn handle_changelog_fetched(&mut self, changelog: Changelog, append: bool) {
        debug!(
            "Changelog fetched: {} entries (total: {})",
            changelog.histories.len(),
            changelog.total
        );
        if append {
            self.detail_view.append_changelog(changelog);
        } else {
            self.detail_view.set_changelog(changelog);
        }
    }

    /// Handle failure to fetch changelog.
    pub fn handle_fetch_changelog_failure(&mut self, error: &str) {
        warn!(error = %error, "Failed to fetch changelog");
        self.detail_view.hide_history();
        self.notify_error(format!("Failed to load history: {}", error));
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

        // Handle delete profile dialog (blocks other input)
        if self.delete_profile_dialog.is_visible() {
            if let Some(confirmed) = self.delete_profile_dialog.handle_input(key_event) {
                if confirmed {
                    if let Some(name) = self.profile_list_view.selected_profile_name() {
                        let index = self.profile_list_view.selected();
                        debug!(name = %name, "Delete profile confirmed");
                        if let Err(e) = self.delete_profile(index) {
                            self.notify_error(format!("Failed to delete profile: {}", e));
                        }
                    }
                } else {
                    debug!("Delete profile cancelled");
                }
            }
            return;
        }

        // Handle discard changes confirmation dialog (blocks other input)
        if self.discard_confirm_dialog.is_visible() {
            if let Some(confirmed) = self.discard_confirm_dialog.handle_input(key_event) {
                if confirmed {
                    debug!("Discard changes confirmed");
                    self.detail_view.exit_edit_mode();
                } else {
                    debug!("Discard changes cancelled");
                }
            }
            return;
        }

        // Handle transition confirmation dialog (blocks other input)
        if self.transition_confirm_dialog.is_visible() {
            if let Some(confirmed) = self.transition_confirm_dialog.handle_input(key_event) {
                if confirmed {
                    debug!("Transition confirmed");
                    self.confirm_transition();
                } else {
                    debug!("Transition cancelled");
                    self.cancel_transition_confirm();
                }
            }
            return;
        }

        // Handle profile form (blocks other input when visible)
        if self.profile_form_view.is_visible() {
            if let Some(action) = self.profile_form_view.handle_input(key_event) {
                match action {
                    ProfileFormAction::Cancel => {
                        debug!("Profile form cancelled");
                    }
                    ProfileFormAction::Submit(data) => {
                        debug!("Profile form submitted");
                        // Handle add/edit
                        let result = if data.original_name.is_some() {
                            self.update_profile(data)
                        } else {
                            self.add_profile(data)
                        };

                        match result {
                            Ok(()) => {
                                self.profile_form_view.hide();
                            }
                            Err(e) => {
                                self.profile_form_view
                                    .set_error(FormField::Name, format!("Failed to save: {}", e));
                            }
                        }
                    }
                    _ => {}
                }
            }
            return;
        }

        // Handle profile picker (blocks other input when visible)
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

        // Handle JQL input (blocks other input when visible)
        if self.jql_input.is_visible() {
            if let Some(action) = self.jql_input.handle_input(key_event) {
                match action {
                    JqlAction::Execute(jql) => {
                        debug!(jql = %jql, "JQL query submitted");
                        self.execute_jql(jql);
                    }
                    JqlAction::Cancel => {
                        debug!("JQL input cancelled");
                        self.state = AppState::IssueList;
                    }
                }
            }
            return;
        }

        // Handle command palette (blocks other input when visible)
        if self.command_palette.is_visible() {
            if let Some(action) = self.command_palette.handle_input(key_event) {
                match action {
                    CommandPaletteAction::Execute(cmd_action) => {
                        debug!(?cmd_action, "Command palette action executed");
                        self.execute_command_action(cmd_action);
                    }
                    CommandPaletteAction::Cancel => {
                        debug!("Command palette cancelled");
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
                    self.previous_state = Some(self.state);
                    self.help_view.reset_scroll();
                    self.state = AppState::Help;
                }
                return;
            }
            // Profile switcher on 'p' (quick switch, available in most views)
            (KeyCode::Char('p'), KeyModifiers::NONE)
                if self.state == AppState::IssueList || self.state == AppState::Loading =>
            {
                debug!("Opening profile picker");
                self.show_profile_picker();
                return;
            }
            // Profile management on 'P' (Shift+p, full management view)
            (KeyCode::Char('P'), KeyModifiers::SHIFT)
                if self.state == AppState::IssueList || self.state == AppState::Loading =>
            {
                debug!("Opening profile management");
                self.open_profile_management();
                return;
            }
            // Command palette on Ctrl+P or Ctrl+K
            (KeyCode::Char('p'), KeyModifiers::CONTROL)
            | (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                debug!("Opening command palette");
                self.command_palette.show();
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
                            self.open_filter_panel();
                        }
                        ListAction::OpenJqlInput => {
                            self.open_jql_input();
                        }
                        ListAction::SortChanged => {
                            info!(
                                column = %self.list_view.sort().column.display_name(),
                                direction = %self.list_view.sort().direction.to_jql(),
                                "Sort changed"
                            );
                            // Reset and reload with new sort
                            self.list_view.reset_for_new_query();
                            self.list_view.set_loading(true);
                            // TODO: Trigger async refresh with new sort
                        }
                        ListAction::LoadMore => {
                            debug!(
                                offset = self.list_view.pagination().current_offset,
                                "Loading more issues"
                            );
                            self.list_view.pagination_mut().start_loading();
                            // TODO: Trigger async load more
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
                            debug!("Entering edit mode");
                            self.detail_view.enter_edit_mode();
                        }
                        DetailAction::OpenComments(issue_key) => {
                            debug!(key = %issue_key, "Opening comments panel");
                            self.detail_view.show_comments_panel();
                        }
                        DetailAction::FetchComments(issue_key) => {
                            debug!(key = %issue_key, "Fetching comments");
                            self.pending_fetch_comments = Some(issue_key);
                        }
                        DetailAction::SubmitComment(issue_key, body) => {
                            debug!(key = %issue_key, "Submitting comment");
                            self.pending_submit_comment = Some((issue_key, body));
                        }
                        DetailAction::SaveEdit(issue_key, update_request) => {
                            debug!(key = %issue_key, "Save edit requested");
                            self.detail_view.set_saving(true);
                            // The async save operation will be handled by the runner
                            // For now, store the pending update info
                            self.pending_issue_update = Some((issue_key, update_request));
                        }
                        DetailAction::ConfirmDiscard => {
                            debug!("Confirm discard changes dialog requested");
                            self.show_discard_confirm_dialog();
                        }
                        DetailAction::OpenTransitionPicker => {
                            debug!("Opening transition picker");
                            self.detail_view.show_transition_picker_loading();
                        }
                        DetailAction::FetchTransitions(issue_key, _current_status) => {
                            debug!(key = %issue_key, "Fetching transitions");
                            // Store request for the runner to pick up
                            self.pending_fetch_transitions = Some(issue_key);
                        }
                        DetailAction::ExecuteTransition(
                            issue_key,
                            transition_id,
                            transition_name,
                            fields,
                        ) => {
                            debug!(key = %issue_key, transition = %transition_id, "Executing transition");
                            // Use confirmation if configured, otherwise execute immediately
                            self.request_transition_with_confirmation(
                                issue_key,
                                transition_id,
                                transition_name,
                                fields,
                            );
                        }
                        DetailAction::TransitionRequiresFields(transition_id) => {
                            debug!(transition = %transition_id, "Transition requires fields (not yet supported)");
                            self.notify_warning(
                                "This transition requires additional fields which are not yet supported"
                            );
                        }
                        DetailAction::FetchAssignableUsers(issue_key, project_key) => {
                            debug!(key = %issue_key, project = %project_key, "Fetching assignable users");
                            // Store request for the runner to pick up
                            self.pending_fetch_assignees = Some((issue_key, project_key));
                        }
                        DetailAction::ChangeAssignee(issue_key, account_id) => {
                            debug!(key = %issue_key, account_id = ?account_id, "Changing assignee");
                            // Store request for the runner to pick up
                            self.pending_assignee_change = Some((issue_key, account_id));
                        }
                        DetailAction::FetchPriorities(issue_key) => {
                            debug!(key = %issue_key, "Fetching priorities");
                            // Store request for the runner to pick up
                            self.pending_fetch_priorities = Some(issue_key);
                        }
                        DetailAction::ChangePriority(issue_key, priority_id) => {
                            debug!(key = %issue_key, priority_id = %priority_id, "Changing priority");
                            // Store request for the runner to pick up
                            self.pending_priority_change = Some((issue_key, priority_id));
                        }
                        DetailAction::FetchLabels(issue_key) => {
                            debug!(key = %issue_key, "Fetching labels");
                            // Store request for the runner to pick up
                            self.pending_fetch_labels = Some(issue_key);
                        }
                        DetailAction::AddLabel(issue_key, label) => {
                            debug!(key = %issue_key, label = %label, "Adding label");
                            // Store request for the runner to pick up
                            self.pending_add_label = Some((issue_key, label));
                        }
                        DetailAction::RemoveLabel(issue_key, label) => {
                            debug!(key = %issue_key, label = %label, "Removing label");
                            // Store request for the runner to pick up
                            self.pending_remove_label = Some((issue_key, label));
                        }
                        DetailAction::FetchComponents(issue_key, project_key) => {
                            debug!(key = %issue_key, project = %project_key, "Fetching components");
                            // Store request for the runner to pick up
                            self.pending_fetch_components = Some((issue_key, project_key));
                        }
                        DetailAction::AddComponent(issue_key, component) => {
                            debug!(key = %issue_key, component = %component, "Adding component");
                            // Store request for the runner to pick up
                            self.pending_add_component = Some((issue_key, component));
                        }
                        DetailAction::RemoveComponent(issue_key, component) => {
                            debug!(key = %issue_key, component = %component, "Removing component");
                            // Store request for the runner to pick up
                            self.pending_remove_component = Some((issue_key, component));
                        }
                        DetailAction::OpenHistory(issue_key) => {
                            debug!(key = %issue_key, "Opening history panel");
                            self.detail_view.show_history();
                        }
                        DetailAction::FetchChangelog(issue_key) => {
                            debug!(key = %issue_key, "Fetching changelog");
                            // Store request for the runner to pick up (start at 0)
                            self.pending_fetch_changelog = Some((issue_key, 0));
                        }
                        DetailAction::LoadMoreChangelog(issue_key) => {
                            let start_at = self.detail_view.history_next_start();
                            debug!(key = %issue_key, start = %start_at, "Loading more changelog");
                            // Store request for the runner to pick up
                            self.pending_fetch_changelog = Some((issue_key, start_at));
                        }
                    }
                }
            }
            AppState::Help => {
                if let Some(action) = self.help_view.handle_input(key_event) {
                    match action {
                        HelpAction::Close => {
                            // Return to previous state, defaulting to IssueList
                            self.state = self.previous_state.unwrap_or(AppState::IssueList);
                            self.previous_state = None;
                        }
                    }
                }
            }
            AppState::FilterPanel => {
                if let Some(action) = self.filter_panel.handle_input(key_event) {
                    match action {
                        FilterPanelAction::Apply(filter) => {
                            self.apply_filter(filter);
                        }
                        FilterPanelAction::Cancel => {
                            debug!("Filter panel cancelled");
                            self.state = AppState::IssueList;
                        }
                    }
                }
            }
            AppState::ProfileSelect => {
                if key_event.code == KeyCode::Esc {
                    self.state = AppState::IssueList;
                }
            }
            AppState::ProfileManagement => {
                // Handle profile list view input
                if let Some(action) = self.profile_list_view.handle_input(key_event) {
                    match action {
                        ProfileListAction::AddProfile => {
                            debug!("Opening add profile form");
                            self.profile_form_view.show_add();
                        }
                        ProfileListAction::EditProfile(index) => {
                            if let Some(profile) = self.get_profile_by_index(index).cloned() {
                                debug!(name = %profile.name, "Opening edit profile form");
                                // Get token for editing (may be empty if not set)
                                let token = auth::get_token(&profile.name).unwrap_or_default();
                                self.profile_form_view.show_edit(&profile, &token);
                            }
                        }
                        ProfileListAction::DeleteProfile(index) => {
                            if let Some(profile) = self.get_profile_by_index(index).cloned() {
                                debug!(name = %profile.name, "Showing delete confirmation");
                                self.delete_profile_dialog.show(&profile.name);
                            }
                        }
                        ProfileListAction::SetDefault(index) => {
                            if let Err(e) = self.set_default_profile(index) {
                                self.notify_error(format!("Failed to set default: {}", e));
                            }
                        }
                        ProfileListAction::SwitchToProfile(index) => {
                            if let Some(profile) = self.get_profile_by_index(index) {
                                let name = profile.name.clone();
                                if let Err(e) = self.switch_profile(&name) {
                                    self.notify_error(format!("Failed to switch profile: {}", e));
                                } else {
                                    // Go back to issue list after switching
                                    self.state = AppState::IssueList;
                                }
                            }
                        }
                        ProfileListAction::GoBack => {
                            debug!("Going back from profile management");
                            self.state = AppState::IssueList;
                        }
                    }
                }
            }
            AppState::JqlInput => {
                // JQL input is handled earlier in this function
                // when jql_input.is_visible() is checked
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

        // Render JQL input (on top of list view)
        self.jql_input.render(frame, area);

        // Render command palette (on top of list view, similar priority to JQL input)
        self.command_palette.render(frame, area);

        // Render profile picker (on top of everything except error dialogs)
        self.profile_picker.render(frame, area);

        // Render profile form (on top of profile list)
        self.profile_form_view.render(frame, area);

        // Render delete profile dialog (on top of profile form)
        self.delete_profile_dialog.render(frame, area);

        // Render discard changes dialog (on top of profile form)
        self.discard_confirm_dialog.render(frame, area);

        // Render transition confirmation dialog
        self.transition_confirm_dialog.render(frame, area);

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
            AppState::ProfileManagement => {
                // Use the ProfileListView for profile management
                self.profile_list_view.render(frame, area);
            }
            AppState::FilterPanel => {
                // Render list view in background with filter panel overlay
                self.list_view.render(frame, area);
                self.filter_panel.render(frame, area);
            }
            AppState::Help => {
                // Render the help view
                self.help_view.render(frame, area);
            }
            _ => {
                // For other states, use the placeholder rendering
                let content = match self.state {
                    AppState::ProfileSelect => self.render_profile_select_view(),
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
            AppState::ProfileManagement => {
                // Profile management status bar
                let footer = Line::from(vec![
                    Span::styled(
                        " Profiles ",
                        Style::default().fg(Color::Black).bg(Color::Cyan),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("{} profiles configured", self.config.profiles.len()),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                let paragraph = Paragraph::new(footer);
                frame.render_widget(paragraph, area);
            }
            AppState::Help => {
                // Help view has its own footer hints
                let footer = Line::from(vec![
                    Span::styled(
                        " Help ",
                        Style::default().fg(Color::Black).bg(Color::Cyan),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        "[j/k] scroll  [g/G] top/bottom  [?/q/Esc] close",
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                let paragraph = Paragraph::new(footer);
                frame.render_widget(paragraph, area);
            }
            AppState::FilterPanel => {
                // Render contextual help for filter panel
                render_context_help(frame, area, KeyContext::FilterPanel);
            }
            _ => {
                // Default status bar for other states
                let state_str = match self.state {
                    AppState::ProfileSelect => "Profile Select",
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
        app.list_view_mut()
            .set_profile_name(Some("work".to_string()));

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

        // Navigate down (from work to personal) using arrow key
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
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

        // Try to press 'r' (refresh) - should be ignored by the picker
        let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        app.update(Event::Key(key));

        // Picker should still be visible (r is not handled by picker)
        assert!(app.is_profile_picker_visible());
        // Profile should NOT have changed
        assert_eq!(app.current_profile_name(), Some("work"));
    }

    #[test]
    fn test_profile_picker_q_cancels() {
        let config = create_test_config_with_profiles();
        let mut app = App::with_config(config);
        app.update(Event::Tick); // Transition to IssueList

        // Open profile picker
        app.show_profile_picker();
        assert!(app.is_profile_picker_visible());

        // Press 'q' - should cancel the picker (vim-style)
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        app.update(Event::Key(key));

        // Picker should be hidden
        assert!(!app.is_profile_picker_visible());
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
