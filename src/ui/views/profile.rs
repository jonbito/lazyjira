//! Profile management views.
//!
//! This module provides TUI views for managing JIRA profiles:
//! - `ProfileListView`: Displays all configured profiles with CRUD operations
//! - `ProfileFormView`: Form for adding or editing a profile

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::config::Profile;
use crate::ui::components::TextInput;
use crate::ui::theme::theme;

// ============================================================================
// Profile List View
// ============================================================================

/// Actions returned from the profile list view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileListAction {
    /// Open the form to add a new profile.
    AddProfile,
    /// Open the form to edit the selected profile.
    EditProfile(usize),
    /// Request to delete the selected profile.
    DeleteProfile(usize),
    /// Set the selected profile as default.
    SetDefault(usize),
    /// Switch to the selected profile.
    SwitchToProfile(usize),
    /// Go back to the previous view.
    GoBack,
}

/// A summary of a profile for display in the list.
#[derive(Debug, Clone)]
pub struct ProfileSummary {
    /// The profile name.
    pub name: String,
    /// The JIRA URL.
    pub url: String,
    /// The user email.
    pub email: String,
    /// Whether this is the default profile.
    pub is_default: bool,
    /// Whether the profile has a token configured.
    pub has_token: bool,
}

impl ProfileSummary {
    /// Create a summary from a profile.
    pub fn from_profile(profile: &Profile, is_default: bool, has_token: bool) -> Self {
        Self {
            name: profile.name.clone(),
            url: profile.url.clone(),
            email: profile.email.clone(),
            is_default,
            has_token,
        }
    }
}

/// The profile list view displaying all configured profiles.
#[derive(Debug)]
pub struct ProfileListView {
    /// The list of profile summaries.
    profiles: Vec<ProfileSummary>,
    /// Currently selected index.
    selected: usize,
    /// List state for ratatui.
    list_state: ListState,
}

impl Default for ProfileListView {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileListView {
    /// Create a new profile list view.
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            selected: 0,
            list_state: ListState::default(),
        }
    }

    /// Set the profiles to display.
    pub fn set_profiles(&mut self, profiles: Vec<ProfileSummary>) {
        self.profiles = profiles;
        if self.selected >= self.profiles.len() {
            self.selected = self.profiles.len().saturating_sub(1);
        }
        self.list_state.select(Some(self.selected));
    }

    /// Get the number of profiles.
    pub fn profile_count(&self) -> usize {
        self.profiles.len()
    }

    /// Get the selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Get the selected profile name.
    pub fn selected_profile_name(&self) -> Option<&str> {
        self.profiles.get(self.selected).map(|p| p.name.as_str())
    }

    /// Move selection down.
    fn move_down(&mut self) {
        if self.profiles.is_empty() {
            return;
        }
        if self.selected < self.profiles.len() - 1 {
            self.selected += 1;
            self.list_state.select(Some(self.selected));
        }
    }

    /// Move selection up.
    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
        }
    }

    /// Handle keyboard input.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<ProfileListAction> {
        match (key.code, key.modifiers) {
            // Navigation
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                self.move_down();
                None
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                self.move_up();
                None
            }
            // Add new profile
            (KeyCode::Char('a'), KeyModifiers::NONE) => Some(ProfileListAction::AddProfile),
            // Edit selected profile
            (KeyCode::Char('e'), KeyModifiers::NONE) | (KeyCode::Enter, _) => {
                if !self.profiles.is_empty() {
                    Some(ProfileListAction::EditProfile(self.selected))
                } else {
                    None
                }
            }
            // Delete selected profile
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                if !self.profiles.is_empty() {
                    Some(ProfileListAction::DeleteProfile(self.selected))
                } else {
                    None
                }
            }
            // Set as default
            (KeyCode::Char('s'), KeyModifiers::NONE) => {
                if !self.profiles.is_empty() {
                    Some(ProfileListAction::SetDefault(self.selected))
                } else {
                    None
                }
            }
            // Switch to profile
            (KeyCode::Char(' '), KeyModifiers::NONE) => {
                if !self.profiles.is_empty() {
                    Some(ProfileListAction::SwitchToProfile(self.selected))
                } else {
                    None
                }
            }
            // Go back
            (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Esc, _) => {
                Some(ProfileListAction::GoBack)
            }
            _ => None,
        }
    }

    /// Render the profile list view.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let t = theme();
        let block = Block::default()
            .title(Span::styled(
                " Profiles ",
                Style::default()
                    .fg(t.accent)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.accent));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.profiles.is_empty() {
            let message = Paragraph::new(vec![
                Line::raw(""),
                Line::styled(
                    "No profiles configured",
                    Style::default().fg(t.dim),
                ),
                Line::raw(""),
                Line::styled(
                    "Press 'a' to add a new profile",
                    Style::default().fg(t.warning),
                ),
            ])
            .alignment(Alignment::Center);
            frame.render_widget(message, inner);
            return;
        }

        // Split area for list and help bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(inner);

        // Create list items
        let items: Vec<ListItem> = self
            .profiles
            .iter()
            .map(|profile| {
                let mut spans = vec![Span::styled(
                    &profile.name,
                    Style::default().add_modifier(Modifier::BOLD),
                )];

                if profile.is_default {
                    spans.push(Span::styled(
                        " (default)",
                        Style::default().fg(t.success),
                    ));
                }

                if !profile.has_token {
                    spans.push(Span::styled(" [no token]", Style::default().fg(t.error)));
                }

                let line1 = Line::from(spans);
                let line2 = Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(&profile.url, Style::default().fg(t.dim)),
                    Span::styled(" - ", Style::default().fg(t.dim)),
                    Span::styled(&profile.email, Style::default().fg(t.dim)),
                ]);

                ListItem::new(vec![line1, line2])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(t.selection_bg)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[0], &mut self.list_state);

        // Render help bar
        let help = Line::from(vec![
            Span::styled("[a]", Style::default().fg(t.warning)),
            Span::raw("dd "),
            Span::styled("[e]", Style::default().fg(t.warning)),
            Span::raw("dit "),
            Span::styled("[d]", Style::default().fg(t.warning)),
            Span::raw("elete "),
            Span::styled("[s]", Style::default().fg(t.warning)),
            Span::raw("et default "),
            Span::styled("[Space]", Style::default().fg(t.warning)),
            Span::raw("switch "),
            Span::styled("[q]", Style::default().fg(t.warning)),
            Span::raw("back"),
        ]);
        let help_para = Paragraph::new(help).alignment(Alignment::Center);
        frame.render_widget(help_para, chunks[1]);
    }
}

// ============================================================================
// Profile Form View
// ============================================================================

/// The mode of the profile form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormMode {
    /// Adding a new profile.
    Add,
    /// Editing an existing profile.
    Edit(String), // Original profile name
}

/// The field currently focused in the form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    Name,
    Url,
    Email,
    Token,
    Submit,
}

impl FormField {
    /// Get the next field.
    fn next(self) -> Self {
        match self {
            Self::Name => Self::Url,
            Self::Url => Self::Email,
            Self::Email => Self::Token,
            Self::Token => Self::Submit,
            Self::Submit => Self::Name,
        }
    }

    /// Get the previous field.
    fn prev(self) -> Self {
        match self {
            Self::Name => Self::Submit,
            Self::Url => Self::Name,
            Self::Email => Self::Url,
            Self::Token => Self::Email,
            Self::Submit => Self::Token,
        }
    }
}

/// Actions returned from the profile form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileFormAction {
    /// Cancel and close the form.
    Cancel,
    /// Submit the form for validation (async).
    Submit(ProfileFormData),
    /// Validation completed successfully.
    ValidationSuccess,
    /// Validation failed with an error message.
    ValidationFailed(String),
}

/// Data from the profile form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileFormData {
    /// Profile name.
    pub name: String,
    /// JIRA URL.
    pub url: String,
    /// User email.
    pub email: String,
    /// API token.
    pub token: String,
    /// Original name if editing.
    pub original_name: Option<String>,
}

/// Validation error for a form field.
#[derive(Debug, Clone)]
pub struct FieldError {
    /// The field with the error.
    pub field: FormField,
    /// The error message.
    pub message: String,
}

/// The profile form view for adding/editing profiles.
#[derive(Debug)]
pub struct ProfileFormView {
    /// The form mode (add or edit).
    mode: FormMode,
    /// The name input field.
    name_input: TextInput,
    /// The URL input field.
    url_input: TextInput,
    /// The email input field.
    email_input: TextInput,
    /// The token input field (masked).
    token_input: TextInput,
    /// The currently focused field.
    focus: FormField,
    /// Current validation error.
    error: Option<FieldError>,
    /// Whether validation is in progress.
    validating: bool,
    /// Whether the form is visible.
    visible: bool,
}

impl Default for ProfileFormView {
    fn default() -> Self {
        Self::new_add()
    }
}

impl ProfileFormView {
    /// Create a new form for adding a profile.
    pub fn new_add() -> Self {
        let mut name_input = TextInput::new();
        name_input.set_placeholder("my-profile");

        let mut url_input = TextInput::new();
        url_input.set_placeholder("https://company.atlassian.net");

        let mut email_input = TextInput::new();
        email_input.set_placeholder("user@company.com");

        let mut token_input = TextInput::masked();
        token_input.set_placeholder("API token from Atlassian");

        Self {
            mode: FormMode::Add,
            name_input,
            url_input,
            email_input,
            token_input,
            focus: FormField::Name,
            error: None,
            validating: false,
            visible: false,
        }
    }

    /// Create a new form for editing a profile.
    pub fn new_edit(profile: &Profile, token: &str) -> Self {
        let mut form = Self::new_add();
        form.mode = FormMode::Edit(profile.name.clone());
        form.name_input.set_value(&profile.name);
        form.url_input.set_value(&profile.url);
        form.email_input.set_value(&profile.email);
        form.token_input.set_value(token);
        form
    }

    /// Show the form.
    pub fn show(&mut self) {
        self.visible = true;
        self.focus = FormField::Name;
        self.error = None;
        self.validating = false;
    }

    /// Show the form for adding a new profile.
    pub fn show_add(&mut self) {
        self.mode = FormMode::Add;
        self.name_input.clear();
        self.url_input.clear();
        self.email_input.clear();
        self.token_input.clear();
        self.show();
    }

    /// Show the form for editing a profile.
    pub fn show_edit(&mut self, profile: &Profile, token: &str) {
        self.mode = FormMode::Edit(profile.name.clone());
        self.name_input.set_value(&profile.name);
        self.url_input.set_value(&profile.url);
        self.email_input.set_value(&profile.email);
        self.token_input.set_value(token);
        self.show();
    }

    /// Hide the form.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if the form is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if validation is in progress.
    pub fn is_validating(&self) -> bool {
        self.validating
    }

    /// Set validation in progress.
    pub fn set_validating(&mut self, validating: bool) {
        self.validating = validating;
    }

    /// Set a validation error.
    pub fn set_error(&mut self, field: FormField, message: impl Into<String>) {
        self.error = Some(FieldError {
            field,
            message: message.into(),
        });
        self.focus = field;
        self.validating = false;
    }

    /// Clear the validation error.
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Get the form mode.
    pub fn mode(&self) -> &FormMode {
        &self.mode
    }

    /// Move to the next field.
    fn next_field(&mut self) {
        self.focus = self.focus.next();
    }

    /// Move to the previous field.
    fn prev_field(&mut self) {
        self.focus = self.focus.prev();
    }

    /// Get a mutable reference to the currently focused input.
    fn current_input(&mut self) -> Option<&mut TextInput> {
        match self.focus {
            FormField::Name => Some(&mut self.name_input),
            FormField::Url => Some(&mut self.url_input),
            FormField::Email => Some(&mut self.email_input),
            FormField::Token => Some(&mut self.token_input),
            FormField::Submit => None,
        }
    }

    /// Validate the form fields locally (before async validation).
    fn validate(&mut self) -> Option<ProfileFormData> {
        self.error = None;

        // Validate name
        let name = self.name_input.value().trim().to_string();
        if name.is_empty() {
            self.set_error(FormField::Name, "Profile name is required");
            return None;
        }
        if name.contains(char::is_whitespace) {
            self.set_error(FormField::Name, "Profile name cannot contain spaces");
            return None;
        }

        // Validate URL
        let url = self.url_input.value().trim().to_string();
        if url.is_empty() {
            self.set_error(FormField::Url, "URL is required");
            return None;
        }
        if !url.starts_with("https://") && !url.starts_with("http://") {
            self.set_error(FormField::Url, "URL must start with https:// or http://");
            return None;
        }

        // Validate email
        let email = self.email_input.value().trim().to_string();
        if email.is_empty() {
            self.set_error(FormField::Email, "Email is required");
            return None;
        }
        if !email.contains('@') {
            self.set_error(FormField::Email, "Invalid email address");
            return None;
        }

        // Validate token
        let token = self.token_input.value().to_string();
        if token.is_empty() {
            self.set_error(FormField::Token, "API token is required");
            return None;
        }

        Some(ProfileFormData {
            name,
            url,
            email,
            token,
            original_name: match &self.mode {
                FormMode::Add => None,
                FormMode::Edit(original) => Some(original.clone()),
            },
        })
    }

    /// Handle keyboard input.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<ProfileFormAction> {
        // Don't handle input while validating
        if self.validating {
            return None;
        }

        match (key.code, key.modifiers) {
            // Tab - next field
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.next_field();
                None
            }
            // Shift+Tab or BackTab - previous field
            (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                self.prev_field();
                None
            }
            // Enter on submit button
            (KeyCode::Enter, KeyModifiers::NONE) if self.focus == FormField::Submit => {
                if let Some(data) = self.validate() {
                    self.validating = true;
                    Some(ProfileFormAction::Submit(data))
                } else {
                    None
                }
            }
            // Enter in text fields - move to next field
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.next_field();
                None
            }
            // Escape - cancel
            (KeyCode::Esc, _) => {
                self.hide();
                Some(ProfileFormAction::Cancel)
            }
            // Delegate to focused input
            _ => {
                if let Some(input) = self.current_input() {
                    input.handle_input(key);
                    // Clear error when user starts typing
                    if self.error.is_some() {
                        self.error = None;
                    }
                }
                None
            }
        }
    }

    /// Render the form as a modal dialog.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate dialog size
        let dialog_width = 60u16.min(area.width.saturating_sub(4));
        let dialog_height = 19u16.min(area.height.saturating_sub(4));

        let dialog_area = centered_rect(area, dialog_width, dialog_height);

        // Clear the dialog area
        frame.render_widget(Clear, dialog_area);

        // Create the outer block
        let t = theme();
        let title = match &self.mode {
            FormMode::Add => " Add Profile ",
            FormMode::Edit(name) => &format!(" Edit Profile: {} ", name),
        };

        let block = Block::default()
            .title(Span::styled(
                title,
                Style::default()
                    .fg(t.accent)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.accent));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Create layout for form fields
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Name
                Constraint::Length(3), // URL
                Constraint::Length(3), // Email
                Constraint::Length(3), // Token
                Constraint::Length(2), // Error
                Constraint::Length(1), // Submit button
            ])
            .split(inner);

        // Render fields
        self.name_input
            .render_with_label(frame, chunks[0], "Name", self.focus == FormField::Name);
        self.url_input
            .render_with_label(frame, chunks[1], "URL", self.focus == FormField::Url);
        self.email_input.render_with_label(
            frame,
            chunks[2],
            "Email",
            self.focus == FormField::Email,
        );
        self.token_input.render_with_label(
            frame,
            chunks[3],
            "API Token",
            self.focus == FormField::Token,
        );

        // Render error if present
        if let Some(ref error) = self.error {
            let error_text = Paragraph::new(Span::styled(
                &error.message,
                Style::default().fg(t.error),
            ))
            .alignment(Alignment::Center);
            frame.render_widget(error_text, chunks[4]);
        } else if self.validating {
            let validating_text = Paragraph::new(Span::styled(
                "Validating connection...",
                Style::default().fg(t.warning),
            ))
            .alignment(Alignment::Center);
            frame.render_widget(validating_text, chunks[4]);
        }

        // Render submit button
        let button_style = if self.focus == FormField::Submit {
            Style::default()
                .fg(t.selection_fg)
                .bg(t.success)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.success)
        };

        let button_text = if self.validating {
            " Validating... "
        } else {
            match &self.mode {
                FormMode::Add => " [Enter] Add Profile ",
                FormMode::Edit(_) => " [Enter] Save Changes ",
            }
        };

        let button =
            Paragraph::new(Span::styled(button_text, button_style)).alignment(Alignment::Center);
        frame.render_widget(button, chunks[5]);
    }
}

// ============================================================================
// Delete Confirmation Dialog
// ============================================================================

/// A confirmation dialog for deleting a profile.
#[derive(Debug, Default)]
pub struct DeleteProfileDialog {
    /// The profile name to delete.
    profile_name: String,
    /// Whether the dialog is visible.
    visible: bool,
    /// Current selection (true = confirm, false = cancel).
    selected_confirm: bool,
}

impl DeleteProfileDialog {
    /// Create a new delete confirmation dialog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the dialog for a profile.
    pub fn show(&mut self, profile_name: impl Into<String>) {
        self.profile_name = profile_name.into();
        self.visible = true;
        self.selected_confirm = false; // Default to cancel for safety
    }

    /// Hide the dialog.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if the dialog is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the profile name being deleted.
    pub fn profile_name(&self) -> &str {
        &self.profile_name
    }

    /// Toggle the selection.
    pub fn toggle_selection(&mut self) {
        self.selected_confirm = !self.selected_confirm;
    }

    /// Handle keyboard input.
    ///
    /// Returns Some(true) if confirmed, Some(false) if cancelled, None otherwise.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<bool> {
        match (key.code, key.modifiers) {
            // Confirm with Y shortcut
            (KeyCode::Char('y'), KeyModifiers::NONE) | (KeyCode::Char('Y'), _) => {
                self.hide();
                Some(true)
            }
            // Cancel with N shortcut or Esc
            (KeyCode::Char('n'), KeyModifiers::NONE)
            | (KeyCode::Char('N'), _)
            | (KeyCode::Esc, _) => {
                self.hide();
                Some(false)
            }
            // Enter confirms current selection
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.hide();
                Some(self.selected_confirm)
            }
            // Tab/Arrow to toggle (no h/l to allow typing those chars in text inputs)
            (KeyCode::Tab, _) | (KeyCode::Left, _) | (KeyCode::Right, _) => {
                self.toggle_selection();
                None
            }
            _ => None,
        }
    }

    /// Render the confirmation dialog.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let dialog_width = 50u16.min(area.width.saturating_sub(4));
        let dialog_height = 8;

        let dialog_area = centered_rect(area, dialog_width, dialog_height);

        let t = theme();

        // Clear the dialog area
        frame.render_widget(Clear, dialog_area);

        // Create the outer block
        let block = Block::default()
            .title(Span::styled(
                " Delete Profile ",
                Style::default().fg(t.error).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.error));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(2), Constraint::Length(1)])
            .split(inner);

        // Render message
        let message = format!(
            "Delete profile '{}'?\nThis action cannot be undone.",
            self.profile_name
        );
        let message_para = Paragraph::new(message)
            .style(Style::default().fg(t.fg))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        frame.render_widget(message_para, chunks[0]);

        // Render buttons
        let confirm_style = if self.selected_confirm {
            Style::default()
                .fg(t.selection_fg)
                .bg(t.error)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.error)
        };

        let cancel_style = if !self.selected_confirm {
            Style::default()
                .fg(t.selection_fg)
                .bg(t.success)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.success)
        };

        let buttons = Line::from(vec![
            Span::styled(" [Y] Delete ", confirm_style),
            Span::raw("  "),
            Span::styled(" [N] Cancel ", cancel_style),
        ]);

        let buttons_para = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(buttons_para, chunks[1]);
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculate a centered rectangle within the given area.
fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile() -> Profile {
        Profile::new(
            "work".to_string(),
            "https://company.atlassian.net".to_string(),
            "user@company.com".to_string(),
        )
    }

    // Profile List View Tests

    #[test]
    fn test_profile_list_new() {
        let view = ProfileListView::new();
        assert_eq!(view.profile_count(), 0);
        assert_eq!(view.selected(), 0);
    }

    #[test]
    fn test_profile_list_set_profiles() {
        let mut view = ProfileListView::new();
        let profiles = vec![
            ProfileSummary {
                name: "work".to_string(),
                url: "https://work.atlassian.net".to_string(),
                email: "user@work.com".to_string(),
                is_default: true,
                has_token: true,
            },
            ProfileSummary {
                name: "personal".to_string(),
                url: "https://personal.atlassian.net".to_string(),
                email: "user@personal.com".to_string(),
                is_default: false,
                has_token: false,
            },
        ];

        view.set_profiles(profiles);
        assert_eq!(view.profile_count(), 2);
        assert_eq!(view.selected_profile_name(), Some("work"));
    }

    #[test]
    fn test_profile_list_navigation() {
        let mut view = ProfileListView::new();
        view.set_profiles(vec![
            ProfileSummary {
                name: "a".to_string(),
                url: "https://a.com".to_string(),
                email: "a@a.com".to_string(),
                is_default: false,
                has_token: true,
            },
            ProfileSummary {
                name: "b".to_string(),
                url: "https://b.com".to_string(),
                email: "b@b.com".to_string(),
                is_default: false,
                has_token: true,
            },
        ]);

        assert_eq!(view.selected(), 0);

        // Move down
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        view.handle_input(key);
        assert_eq!(view.selected(), 1);

        // Move up
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        view.handle_input(key);
        assert_eq!(view.selected(), 0);
    }

    #[test]
    fn test_profile_list_actions() {
        let mut view = ProfileListView::new();
        view.set_profiles(vec![ProfileSummary {
            name: "test".to_string(),
            url: "https://test.com".to_string(),
            email: "test@test.com".to_string(),
            is_default: false,
            has_token: true,
        }]);

        // Add action
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(view.handle_input(key), Some(ProfileListAction::AddProfile));

        // Edit action
        let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert_eq!(
            view.handle_input(key),
            Some(ProfileListAction::EditProfile(0))
        );

        // Delete action
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert_eq!(
            view.handle_input(key),
            Some(ProfileListAction::DeleteProfile(0))
        );

        // Go back
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(view.handle_input(key), Some(ProfileListAction::GoBack));
    }

    #[test]
    fn test_profile_list_empty_no_actions() {
        let mut view = ProfileListView::new();

        // Edit on empty list should do nothing
        let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert_eq!(view.handle_input(key), None);

        // Delete on empty list should do nothing
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert_eq!(view.handle_input(key), None);
    }

    // Profile Form View Tests

    #[test]
    fn test_form_new_add() {
        let form = ProfileFormView::new_add();
        assert_eq!(form.mode(), &FormMode::Add);
        assert!(!form.is_visible());
        assert!(!form.is_validating());
    }

    #[test]
    fn test_form_new_edit() {
        let profile = create_test_profile();
        let form = ProfileFormView::new_edit(&profile, "token123");
        assert_eq!(form.mode(), &FormMode::Edit("work".to_string()));
    }

    #[test]
    fn test_form_show_hide() {
        let mut form = ProfileFormView::new_add();
        assert!(!form.is_visible());

        form.show();
        assert!(form.is_visible());

        form.hide();
        assert!(!form.is_visible());
    }

    #[test]
    fn test_form_field_navigation() {
        let mut form = ProfileFormView::new_add();
        form.show();

        // Initial focus on Name
        assert_eq!(form.focus, FormField::Name);

        // Tab to URL
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        form.handle_input(key);
        assert_eq!(form.focus, FormField::Url);

        // Tab to Email
        form.handle_input(key);
        assert_eq!(form.focus, FormField::Email);

        // Tab to Token
        form.handle_input(key);
        assert_eq!(form.focus, FormField::Token);

        // Tab to Submit
        form.handle_input(key);
        assert_eq!(form.focus, FormField::Submit);

        // Tab wraps to Name
        form.handle_input(key);
        assert_eq!(form.focus, FormField::Name);

        // Shift+Tab goes back to Submit
        let key = KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE);
        form.handle_input(key);
        assert_eq!(form.focus, FormField::Submit);
    }

    #[test]
    fn test_form_cancel() {
        let mut form = ProfileFormView::new_add();
        form.show();

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = form.handle_input(key);

        assert_eq!(action, Some(ProfileFormAction::Cancel));
        assert!(!form.is_visible());
    }

    #[test]
    fn test_form_validation_empty_name() {
        let mut form = ProfileFormView::new_add();
        form.show();

        // Move to submit
        form.focus = FormField::Submit;

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = form.handle_input(key);

        assert!(action.is_none());
        assert!(form.error.is_some());
        assert_eq!(form.error.as_ref().unwrap().field, FormField::Name);
    }

    #[test]
    fn test_form_validation_invalid_url() {
        let mut form = ProfileFormView::new_add();
        form.show();

        form.name_input.set_value("test");
        form.url_input.set_value("not-a-url");
        form.focus = FormField::Submit;

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        form.handle_input(key);

        assert!(form.error.is_some());
        assert_eq!(form.error.as_ref().unwrap().field, FormField::Url);
    }

    #[test]
    fn test_form_validation_invalid_email() {
        let mut form = ProfileFormView::new_add();
        form.show();

        form.name_input.set_value("test");
        form.url_input.set_value("https://test.com");
        form.email_input.set_value("not-an-email");
        form.focus = FormField::Submit;

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        form.handle_input(key);

        assert!(form.error.is_some());
        assert_eq!(form.error.as_ref().unwrap().field, FormField::Email);
    }

    #[test]
    fn test_form_validation_success() {
        let mut form = ProfileFormView::new_add();
        form.show();

        form.name_input.set_value("test");
        form.url_input.set_value("https://test.atlassian.net");
        form.email_input.set_value("user@test.com");
        form.token_input.set_value("secret-token");
        form.focus = FormField::Submit;

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = form.handle_input(key);

        assert!(matches!(action, Some(ProfileFormAction::Submit(_))));
        assert!(form.is_validating());
    }

    // Delete Dialog Tests

    #[test]
    fn test_delete_dialog_new() {
        let dialog = DeleteProfileDialog::new();
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_delete_dialog_show() {
        let mut dialog = DeleteProfileDialog::new();
        dialog.show("work");

        assert!(dialog.is_visible());
        assert_eq!(dialog.profile_name(), "work");
        assert!(!dialog.selected_confirm); // Default to cancel
    }

    #[test]
    fn test_delete_dialog_confirm_with_y() {
        let mut dialog = DeleteProfileDialog::new();
        dialog.show("work");

        let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        let result = dialog.handle_input(key);

        assert_eq!(result, Some(true));
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_delete_dialog_cancel_with_n() {
        let mut dialog = DeleteProfileDialog::new();
        dialog.show("work");

        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        let result = dialog.handle_input(key);

        assert_eq!(result, Some(false));
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_delete_dialog_cancel_with_esc() {
        let mut dialog = DeleteProfileDialog::new();
        dialog.show("work");

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let result = dialog.handle_input(key);

        assert_eq!(result, Some(false));
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_delete_dialog_toggle() {
        let mut dialog = DeleteProfileDialog::new();
        dialog.show("work");

        assert!(!dialog.selected_confirm);

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        dialog.handle_input(key);

        assert!(dialog.selected_confirm);
    }

    // Helper Tests

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = centered_rect(area, 40, 20);

        assert_eq!(centered.x, 30);
        assert_eq!(centered.y, 15);
        assert_eq!(centered.width, 40);
        assert_eq!(centered.height, 20);
    }

    #[test]
    fn test_centered_rect_larger_than_area() {
        let area = Rect::new(0, 0, 30, 20);
        let centered = centered_rect(area, 50, 30);

        // Should be clamped to area size
        assert_eq!(centered.width, 30);
        assert_eq!(centered.height, 20);
    }

    #[test]
    fn test_form_field_next() {
        assert_eq!(FormField::Name.next(), FormField::Url);
        assert_eq!(FormField::Url.next(), FormField::Email);
        assert_eq!(FormField::Email.next(), FormField::Token);
        assert_eq!(FormField::Token.next(), FormField::Submit);
        assert_eq!(FormField::Submit.next(), FormField::Name);
    }

    #[test]
    fn test_form_field_prev() {
        assert_eq!(FormField::Name.prev(), FormField::Submit);
        assert_eq!(FormField::Url.prev(), FormField::Name);
        assert_eq!(FormField::Email.prev(), FormField::Url);
        assert_eq!(FormField::Token.prev(), FormField::Email);
        assert_eq!(FormField::Submit.prev(), FormField::Token);
    }

    #[test]
    fn test_profile_summary_from_profile() {
        let profile = create_test_profile();
        let summary = ProfileSummary::from_profile(&profile, true, true);

        assert_eq!(summary.name, "work");
        assert_eq!(summary.url, "https://company.atlassian.net");
        assert_eq!(summary.email, "user@company.com");
        assert!(summary.is_default);
        assert!(summary.has_token);
    }
}
