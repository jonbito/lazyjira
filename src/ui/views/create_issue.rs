//! Create issue view for adding new JIRA issues.
//!
//! This module provides the modal form for creating new JIRA issues from
//! the issue list view. It follows the same patterns as `ProfileFormView`
//! for form structure and keyboard navigation.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::api::types::{IssueTypeMeta, Priority, User};
use crate::app::{App, CreateIssueFormData, CreateIssueFormField};
use crate::ui::components::{
    AssigneeAction, AssigneePicker, Dropdown, DropdownAction, DropdownItem, PriorityAction,
    PriorityPicker, TextEditor, TextInput,
};
use crate::ui::theme::theme;

// ============================================================================
// Render Data
// ============================================================================

/// Data needed to render the create issue view.
///
/// This struct owns all its data to avoid borrow checker issues when
/// rendering from App::view() method. The data is cloned from App fields.
pub struct CreateIssueRenderData {
    /// Current focused field.
    pub focus: CreateIssueFormField,
    /// Form data (cloned).
    pub form: CreateIssueFormData,
    /// Available issue types for the selected project (cloned).
    pub issue_types: Vec<IssueTypeMeta>,
    /// Whether issue types are currently being fetched.
    pub is_fetching_issue_types: bool,
    /// Available projects as (key, name) pairs.
    pub projects: Vec<(String, String)>,
    /// Available epics as (key, display_label) pairs.
    pub epics: Vec<(String, String)>,
    /// Validation errors (cloned).
    pub errors: Vec<String>,
}

// ============================================================================
// Create Issue View
// ============================================================================

/// Actions returned from the create issue view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateIssueAction {
    /// Close the form without creating an issue.
    Cancel,
    /// Submit the form to create the issue.
    Submit,
    /// Request to fetch issue types for the selected project.
    FetchIssueTypes(String),
    /// Request to fetch assignable users for the selected project.
    FetchAssignableUsers(String),
    /// Request to fetch available priorities.
    FetchPriorities,
}

/// The create issue view for adding new JIRA issues.
///
/// This view renders as a modal overlay on top of the issue list and provides
/// a form with fields for project, issue type, summary, description, assignee,
/// and priority.
pub struct CreateIssueView {
    /// Summary text input.
    summary_input: TextInput,
    /// Parent issue key text input (for subtasks).
    parent_input: TextInput,
    /// Epic dropdown (for standard issues).
    epic_dropdown: Dropdown,
    /// Description text editor.
    description_editor: TextEditor,
    /// Project dropdown.
    project_dropdown: Dropdown,
    /// Issue type dropdown.
    issue_type_dropdown: Dropdown,
    /// Assignee picker.
    assignee_picker: AssigneePicker,
    /// Priority picker.
    priority_picker: PriorityPicker,
    /// Whether the form is submitting.
    submitting: bool,
}

impl Default for CreateIssueView {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateIssueView {
    /// Create a new create issue view.
    pub fn new() -> Self {
        let mut summary_input = TextInput::new();
        summary_input.set_placeholder("Enter issue summary...");

        let mut parent_input = TextInput::new();
        parent_input.set_placeholder("e.g., PROJ-123");

        let mut epic_dropdown = Dropdown::new("Epic");
        epic_dropdown.set_required(false);
        epic_dropdown.set_placeholder("None (optional)");

        let mut project_dropdown = Dropdown::new("Project");
        project_dropdown.set_required(true);
        project_dropdown.set_placeholder("Select project...");

        let mut issue_type_dropdown = Dropdown::new("Issue Type");
        issue_type_dropdown.set_required(true);
        issue_type_dropdown.set_placeholder("Select issue type...");

        Self {
            summary_input,
            parent_input,
            epic_dropdown,
            description_editor: TextEditor::empty(),
            project_dropdown,
            issue_type_dropdown,
            assignee_picker: AssigneePicker::new(),
            priority_picker: PriorityPicker::new(),
            submitting: false,
        }
    }

    /// Reset the view to initial state.
    pub fn reset(&mut self) {
        self.summary_input.clear();
        self.parent_input.clear();
        self.epic_dropdown.reset();
        self.description_editor = TextEditor::empty();
        self.project_dropdown.reset();
        self.issue_type_dropdown.reset();
        self.assignee_picker.hide();
        self.priority_picker.hide();
        self.submitting = false;
    }

    /// Set the summary value.
    pub fn set_summary(&mut self, value: &str) {
        self.summary_input.set_value(value);
    }

    /// Get the summary value.
    pub fn summary(&self) -> &str {
        self.summary_input.value()
    }

    /// Set the description value.
    pub fn set_description(&mut self, content: &str) {
        self.description_editor = TextEditor::new(content);
    }

    /// Get the description value.
    pub fn description(&self) -> String {
        self.description_editor.content()
    }

    /// Set the submitting state.
    pub fn set_submitting(&mut self, submitting: bool) {
        self.submitting = submitting;
    }

    /// Check if currently submitting.
    pub fn is_submitting(&self) -> bool {
        self.submitting
    }

    /// Check if the project dropdown is expanded.
    pub fn is_project_dropdown_expanded(&self) -> bool {
        self.project_dropdown.is_expanded()
    }

    /// Get a reference to the project dropdown.
    pub fn project_dropdown(&self) -> &Dropdown {
        &self.project_dropdown
    }

    /// Get a mutable reference to the project dropdown.
    pub fn project_dropdown_mut(&mut self) -> &mut Dropdown {
        &mut self.project_dropdown
    }

    /// Set the items in the project dropdown.
    pub fn set_project_items(&mut self, items: Vec<DropdownItem>) {
        self.project_dropdown.set_items(items);
    }

    /// Select a project by ID in the dropdown.
    pub fn select_project_by_id(&mut self, id: &str) {
        self.project_dropdown.select_by_id(id);
    }

    /// Get the selected project index (for backwards compatibility).
    pub fn project_picker_index(&self) -> Option<usize> {
        self.project_dropdown.selected_index()
    }

    /// Handle input for the project dropdown and return any action.
    pub fn handle_project_dropdown_input(&mut self, key: KeyEvent) -> Option<DropdownAction> {
        self.project_dropdown.handle_input(key)
    }

    /// Check if the issue type dropdown is expanded.
    pub fn is_issue_type_dropdown_expanded(&self) -> bool {
        self.issue_type_dropdown.is_expanded()
    }

    /// Get the selected issue type index.
    pub fn issue_type_picker_index(&self) -> Option<usize> {
        self.issue_type_dropdown.selected_index()
    }

    /// Set the items in the issue type dropdown.
    pub fn set_issue_type_items(&mut self, items: Vec<DropdownItem>) {
        self.issue_type_dropdown.set_items(items);
    }

    /// Select an issue type by ID in the dropdown.
    pub fn select_issue_type_by_id(&mut self, id: &str) {
        self.issue_type_dropdown.select_by_id(id);
    }

    /// Handle input for the issue type dropdown and return any action.
    pub fn handle_issue_type_dropdown_input(&mut self, key: KeyEvent) -> Option<DropdownAction> {
        self.issue_type_dropdown.handle_input(key)
    }

    /// Handle input for the summary field.
    pub fn handle_summary_input(&mut self, key: KeyEvent) {
        self.summary_input.handle_input(key);
    }

    /// Handle input for the description field.
    pub fn handle_description_input(&mut self, key: KeyEvent) {
        self.description_editor.handle_input(key);
    }

    /// Set the parent issue key value.
    pub fn set_parent(&mut self, value: &str) {
        self.parent_input.set_value(value);
    }

    /// Get the parent issue key value.
    pub fn parent(&self) -> &str {
        self.parent_input.value()
    }

    /// Handle input for the parent issue field.
    pub fn handle_parent_input(&mut self, key: KeyEvent) {
        self.parent_input.handle_input(key);
    }

    /// Check if the epic dropdown is expanded.
    pub fn is_epic_dropdown_expanded(&self) -> bool {
        self.epic_dropdown.is_expanded()
    }

    /// Get the selected epic key.
    pub fn selected_epic_key(&self) -> Option<String> {
        self.epic_dropdown
            .selected_item()
            .map(|item| item.id.clone())
    }

    /// Set the items in the epic dropdown.
    pub fn set_epic_items(&mut self, items: Vec<DropdownItem>) {
        self.epic_dropdown.set_items(items);
    }

    /// Select an epic by key in the dropdown.
    pub fn select_epic_by_id(&mut self, id: &str) {
        self.epic_dropdown.select_by_id(id);
    }

    /// Clear the epic selection.
    pub fn clear_epic_selection(&mut self) {
        self.epic_dropdown.reset();
    }

    /// Handle input for the epic dropdown and return any action.
    pub fn handle_epic_dropdown_input(&mut self, key: KeyEvent) -> Option<DropdownAction> {
        self.epic_dropdown.handle_input(key)
    }

    // ========================================================================
    // Assignee Picker Methods
    // ========================================================================

    /// Check if the assignee picker is visible.
    pub fn is_assignee_picker_visible(&self) -> bool {
        self.assignee_picker.is_visible()
    }

    /// Check if assignable users are loading.
    pub fn is_assignees_loading(&self) -> bool {
        self.assignee_picker.is_loading()
    }

    /// Show the assignee picker in loading state.
    ///
    /// This should be called when the user presses Enter on the assignee field,
    /// and then the API call to get assignable users should be made.
    pub fn show_assignee_picker_loading(&mut self, current_assignee: &str) {
        self.assignee_picker.show_loading(current_assignee);
    }

    /// Set the available users in the assignee picker.
    ///
    /// Call this after receiving the users from the API.
    pub fn set_assignable_users(&mut self, users: Vec<User>, current_assignee: &str) {
        self.assignee_picker.show(users, current_assignee);
    }

    /// Hide the assignee picker.
    pub fn hide_assignee_picker(&mut self) {
        self.assignee_picker.hide();
    }

    // ========================================================================
    // Priority Picker Methods
    // ========================================================================

    /// Check if the priority picker is visible.
    pub fn is_priority_picker_visible(&self) -> bool {
        self.priority_picker.is_visible()
    }

    /// Check if priorities are loading.
    pub fn is_priorities_loading(&self) -> bool {
        self.priority_picker.is_loading()
    }

    /// Show the priority picker in loading state.
    ///
    /// This should be called when the user presses Enter on the priority field,
    /// and then the API call to get priorities should be made.
    pub fn show_priority_picker_loading(&mut self, current_priority: &str) {
        self.priority_picker.show_loading(current_priority);
    }

    /// Set the available priorities in the priority picker.
    ///
    /// Call this after receiving the priorities from the API.
    pub fn set_priorities(&mut self, priorities: Vec<Priority>, current_priority: &str) {
        self.priority_picker.show(priorities, current_priority);
    }

    /// Hide the priority picker.
    pub fn hide_priority_picker(&mut self) {
        self.priority_picker.hide();
    }

    /// Handle input when the priority picker is visible.
    ///
    /// Returns the action from the picker (if any) for the caller to handle.
    pub fn handle_priority_picker_input(&mut self, key: KeyEvent) -> Option<PriorityAction> {
        self.priority_picker.handle_input(key)
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the parent.
    pub fn handle_input(&mut self, app: &mut App, key: KeyEvent) -> Option<CreateIssueAction> {
        // Don't handle input while submitting
        if self.submitting {
            return None;
        }

        let focus = app.create_issue_focus();

        // Note: Assignee picker input is handled by App::handle_create_issue_assignee_picker_input
        // which is called before this method when the picker is visible.

        // If the project dropdown is expanded, handle its input first
        if self.project_dropdown.is_expanded() {
            return self.handle_project_input(app, key);
        }

        // If the issue type dropdown is expanded, handle its input first
        if self.issue_type_dropdown.is_expanded() {
            return self.handle_issue_type_input(app, key);
        }

        // If the epic dropdown is expanded, handle its input first
        if self.epic_dropdown.is_expanded() {
            return self.handle_epic_input(app, key);
        }

        match (key.code, key.modifiers) {
            // Tab - next field
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.sync_to_app(app);
                app.create_issue_focus_next();
                self.sync_from_app(app);
                None
            }
            // Shift+Tab or BackTab - previous field
            (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                self.sync_to_app(app);
                app.create_issue_focus_prev();
                self.sync_from_app(app);
                None
            }
            // Escape - cancel
            (KeyCode::Esc, _) => {
                self.reset();
                Some(CreateIssueAction::Cancel)
            }
            // Enter on submit button - validate and submit
            (KeyCode::Enter, KeyModifiers::NONE) if focus == CreateIssueFormField::Submit => {
                self.sync_to_app(app);
                if app.validate_create_issue_form() {
                    self.submitting = true;
                    app.set_pending_create_issue(true);
                    Some(CreateIssueAction::Submit)
                } else {
                    None
                }
            }
            // Enter in other fields - move to next field (except description and dropdowns which use Enter)
            (KeyCode::Enter, KeyModifiers::NONE)
                if focus != CreateIssueFormField::Description
                    && focus != CreateIssueFormField::Project
                    && focus != CreateIssueFormField::IssueType
                    && focus != CreateIssueFormField::EpicParent =>
            {
                self.sync_to_app(app);
                app.create_issue_focus_next();
                self.sync_from_app(app);
                None
            }
            // Handle field-specific input
            _ => self.handle_field_input(app, key, focus),
        }
    }

    /// Handle input for specific fields.
    fn handle_field_input(
        &mut self,
        app: &mut App,
        key: KeyEvent,
        focus: CreateIssueFormField,
    ) -> Option<CreateIssueAction> {
        match focus {
            CreateIssueFormField::Project => self.handle_project_input(app, key),
            CreateIssueFormField::IssueType => self.handle_issue_type_input(app, key),
            CreateIssueFormField::Parent => {
                self.parent_input.handle_input(key);
                None
            }
            CreateIssueFormField::EpicParent => self.handle_epic_input(app, key),
            CreateIssueFormField::Summary => {
                self.summary_input.handle_input(key);
                None
            }
            CreateIssueFormField::Description => {
                self.description_editor.handle_input(key);
                None
            }
            CreateIssueFormField::Assignee => self.handle_assignee_input(app, key),
            CreateIssueFormField::Priority => self.handle_priority_input(app, key),
            CreateIssueFormField::Submit => None,
        }
    }

    /// Handle project dropdown input.
    fn handle_project_input(&mut self, app: &mut App, key: KeyEvent) -> Option<CreateIssueAction> {
        // Sync projects to the dropdown if not already set
        if self.project_dropdown.is_empty() {
            let projects = Self::get_available_projects(app);
            let items: Vec<DropdownItem> = projects
                .into_iter()
                .map(|(key, name)| DropdownItem::new(key, name))
                .collect();
            self.project_dropdown.set_items(items);

            // Sync current selection if exists
            let current_key = app.create_issue_form().project_key.clone();
            if !current_key.is_empty() {
                self.project_dropdown.select_by_id(&current_key);
            }
        }

        if let Some(action) = self.project_dropdown.handle_input(key) {
            match action {
                DropdownAction::Select(key, name) => {
                    self.update_selected_project(app, &key, &name);
                }
                DropdownAction::Cancel => {
                    // Just close the dropdown, no action needed
                }
            }
        }
        None
    }

    /// Handle issue type dropdown input.
    fn handle_issue_type_input(
        &mut self,
        app: &mut App,
        key: KeyEvent,
    ) -> Option<CreateIssueAction> {
        // Sync issue types to the dropdown if not already set or if they've changed
        let issue_types = app.available_issue_types();
        if self.issue_type_dropdown.item_count() != issue_types.len() {
            let items: Vec<DropdownItem> = issue_types
                .iter()
                .map(|t| DropdownItem::new(t.id.clone(), t.name.clone()))
                .collect();
            self.issue_type_dropdown.set_items(items);

            // Sync current selection if exists
            let current_id = app.create_issue_form().issue_type_id.clone();
            if !current_id.is_empty() {
                self.issue_type_dropdown.select_by_id(&current_id);
            }
        }

        if let Some(action) = self.issue_type_dropdown.handle_input(key) {
            match action {
                DropdownAction::Select(id, name) => {
                    self.update_selected_issue_type_by_id(app, &id, &name);
                }
                DropdownAction::Cancel => {
                    // Just close the dropdown, no action needed
                }
            }
        }
        None
    }

    /// Handle epic dropdown input.
    fn handle_epic_input(&mut self, app: &mut App, key: KeyEvent) -> Option<CreateIssueAction> {
        // Sync epics to the dropdown if not already set
        if self.epic_dropdown.is_empty() {
            let epics = Self::get_available_epics(app);
            let items: Vec<DropdownItem> = epics
                .into_iter()
                .map(|(key, label)| DropdownItem::new(key, label))
                .collect();
            self.epic_dropdown.set_items(items);

            // Sync current selection if exists
            if let Some(ref current_key) = app.create_issue_form().epic_parent_key {
                self.epic_dropdown.select_by_id(current_key);
            }
        }

        if let Some(action) = self.epic_dropdown.handle_input(key) {
            match action {
                DropdownAction::Select(key, _label) => {
                    self.update_selected_epic(app, &key);
                }
                DropdownAction::Cancel => {
                    // Just close the dropdown, no action needed
                }
            }
        }
        None
    }

    /// Handle assignee field input - opens the picker on Enter.
    fn handle_assignee_input(
        &mut self,
        app: &mut App,
        key: KeyEvent,
    ) -> Option<CreateIssueAction> {
        match (key.code, key.modifiers) {
            (KeyCode::Enter, KeyModifiers::NONE) => {
                // Check if a project is selected
                let form = app.create_issue_form();
                if form.project_key.is_empty() {
                    // Can't fetch assignees without a project
                    return None;
                }

                let project_key = form.project_key.clone();
                let current_assignee = form
                    .assignee_name
                    .clone()
                    .unwrap_or_else(|| "Unassigned".to_string());

                // Show picker in loading state
                self.assignee_picker.show_loading(&current_assignee);

                // Return action to fetch assignable users
                Some(CreateIssueAction::FetchAssignableUsers(project_key))
            }
            _ => None,
        }
    }

    /// Handle input when the assignee picker is visible.
    ///
    /// Returns the action from the picker (if any) for the caller to handle.
    pub fn handle_assignee_picker_input(&mut self, key: KeyEvent) -> Option<AssigneeAction> {
        self.assignee_picker.handle_input(key)
    }

    /// Handle priority field input - opens the picker on Enter.
    fn handle_priority_input(
        &mut self,
        app: &mut App,
        key: KeyEvent,
    ) -> Option<CreateIssueAction> {
        match (key.code, key.modifiers) {
            (KeyCode::Enter, KeyModifiers::NONE) => {
                // Get current priority name for display
                let form = app.create_issue_form();
                let current_priority = form
                    .priority_name
                    .clone()
                    .unwrap_or_else(|| "Default".to_string());

                // Show picker in loading state
                self.priority_picker.show_loading(&current_priority);

                // Return action to fetch priorities
                Some(CreateIssueAction::FetchPriorities)
            }
            _ => None,
        }
    }

    /// Get available projects from the filter options.
    ///
    /// Returns a vector of (project_key, project_name) tuples.
    /// Note: FilterOption uses `id` for the key and `label` for the display name.
    fn get_available_projects(app: &App) -> Vec<(String, String)> {
        if let Some(options) = app.filter_options() {
            options
                .projects
                .iter()
                .map(|p| (p.id.clone(), p.label.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Update the selected project in app state.
    fn update_selected_project(&mut self, app: &mut App, key: &str, name: &str) {
        let form = app.create_issue_form_mut();
        form.project_key = key.to_string();
        form.project_name = name.to_string();

        // Clear issue type when project changes
        form.issue_type_id.clear();
        form.issue_type_name.clear();
        self.issue_type_dropdown.reset();

        // Clear assignee when project changes (assignees are project-specific)
        form.assignee_id = None;
        form.assignee_name = None;

        // Request to fetch issue types for this project
        app.set_pending_fetch_issue_types(true);
    }

    /// Update the selected issue type in app state by ID.
    fn update_selected_issue_type_by_id(&mut self, app: &mut App, id: &str, name: &str) {
        let form = app.create_issue_form_mut();
        form.issue_type_id = id.to_string();
        form.issue_type_name = name.to_string();
    }

    /// Get available epics from the filter options.
    ///
    /// Returns a vector of (epic_key, display_label) tuples.
    fn get_available_epics(app: &App) -> Vec<(String, String)> {
        if let Some(options) = app.filter_options() {
            options
                .epics
                .iter()
                .map(|e| (e.id.clone(), e.label.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Update the selected epic in app state.
    fn update_selected_epic(&mut self, app: &mut App, key: &str) {
        let form = app.create_issue_form_mut();
        if key.is_empty() {
            form.epic_parent_key = None;
        } else {
            form.epic_parent_key = Some(key.to_string());
        }
    }

    /// Sync local state to app state.
    fn sync_to_app(&self, app: &mut App) {
        let form = app.create_issue_form_mut();
        form.summary = self.summary_input.value().to_string();
        form.description = self.description_editor.content();
    }

    /// Sync from app state to local state.
    fn sync_from_app(&mut self, app: &App) {
        let form = app.create_issue_form();
        if self.summary_input.value() != form.summary {
            self.summary_input.set_value(&form.summary);
        }
        if self.description_editor.content() != form.description {
            self.description_editor = TextEditor::new(&form.description);
        }

        // Update project dropdown items and selection
        let projects = Self::get_available_projects(app);
        let items: Vec<DropdownItem> = projects
            .into_iter()
            .map(|(key, name)| DropdownItem::new(key, name))
            .collect();
        self.project_dropdown.set_items(items);
        if !form.project_key.is_empty() {
            self.project_dropdown.select_by_id(&form.project_key);
        }

        // Update issue type dropdown items and selection
        let issue_types = app.available_issue_types();
        let type_items: Vec<DropdownItem> = issue_types
            .iter()
            .map(|t| DropdownItem::new(t.id.clone(), t.name.clone()))
            .collect();
        self.issue_type_dropdown.set_items(type_items);
        if !form.issue_type_id.is_empty() {
            self.issue_type_dropdown.select_by_id(&form.issue_type_id);
        }

        // Update epic dropdown items and selection
        let epics = Self::get_available_epics(app);
        let epic_items: Vec<DropdownItem> = epics
            .into_iter()
            .map(|(key, label)| DropdownItem::new(key, label))
            .collect();
        self.epic_dropdown.set_items(epic_items);
        if let Some(ref epic_key) = form.epic_parent_key {
            self.epic_dropdown.select_by_id(epic_key);
        }
    }

    /// Render the create issue view as a modal overlay.
    pub fn render(&mut self, data: &CreateIssueRenderData, frame: &mut Frame, area: Rect) {
        // Calculate dialog size - form needs height for all fields:
        // Project(3) + IssueType(3) + Parent/Epic(3 if applicable) + Summary(3) + Description(6) + Assignee(3) + Priority(3) + Errors(2) + Submit(1)
        // Plus borders(2) and margin(2)
        let dialog_width = 70u16.min(area.width.saturating_sub(4));

        // Determine if we need an extra row for parent/epic field
        let has_parent_field = data.form.is_subtask || data.form.can_have_epic_parent;
        let base_height = if has_parent_field { 33u16 } else { 30u16 };
        let dialog_height = base_height.min(area.height.saturating_sub(4));

        let dialog_area = centered_rect(area, dialog_width, dialog_height);

        // Clear the dialog area
        frame.render_widget(Clear, dialog_area);

        // Create the outer block
        let t = theme();
        let block = Block::default()
            .title(Span::styled(
                " Create New Issue ",
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.accent));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let focus = data.focus;

        // Create layout for form fields - different layout based on issue type
        if data.form.is_subtask {
            // Subtask layout with required Parent field
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3), // Project
                    Constraint::Length(3), // Issue Type
                    Constraint::Length(3), // Parent Issue (required for subtasks)
                    Constraint::Length(3), // Summary
                    Constraint::Length(6), // Description (multi-line)
                    Constraint::Length(3), // Assignee (optional)
                    Constraint::Length(3), // Priority (optional)
                    Constraint::Length(2), // Errors
                    Constraint::Length(1), // Submit button
                ])
                .split(inner);

            // Render fields
            let project_area = chunks[0];
            let issue_type_area = chunks[1];
            self.render_project_field(
                data,
                frame,
                project_area,
                focus == CreateIssueFormField::Project,
            );
            self.render_issue_type_field(
                data,
                frame,
                issue_type_area,
                focus == CreateIssueFormField::IssueType,
            );
            self.render_parent_field(
                data,
                frame,
                chunks[2],
                focus == CreateIssueFormField::Parent,
            );
            self.render_summary_field(frame, chunks[3], focus == CreateIssueFormField::Summary);
            self.render_description_field(
                frame,
                chunks[4],
                focus == CreateIssueFormField::Description,
            );
            self.render_assignee_field(
                data,
                frame,
                chunks[5],
                focus == CreateIssueFormField::Assignee,
            );
            self.render_priority_field(
                data,
                frame,
                chunks[6],
                focus == CreateIssueFormField::Priority,
            );
            self.render_errors(data, frame, chunks[7]);
            self.render_submit_button(frame, chunks[8], focus == CreateIssueFormField::Submit);

            // Render dropdown overlays LAST so they appear on top
            self.render_project_dropdown_overlay(frame, project_area, area);
            self.render_issue_type_dropdown_overlay(frame, issue_type_area, area);

            // Render picker overlays (on top of everything)
            self.assignee_picker.render(frame, area);
            self.priority_picker.render(frame, area);
        } else if data.form.can_have_epic_parent {
            // Standard issue layout with optional Epic field
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3), // Project
                    Constraint::Length(3), // Issue Type
                    Constraint::Length(3), // Epic (optional)
                    Constraint::Length(3), // Summary
                    Constraint::Length(6), // Description (multi-line)
                    Constraint::Length(3), // Assignee (optional)
                    Constraint::Length(3), // Priority (optional)
                    Constraint::Length(2), // Errors
                    Constraint::Length(1), // Submit button
                ])
                .split(inner);

            // Render fields
            let project_area = chunks[0];
            let issue_type_area = chunks[1];
            self.render_project_field(
                data,
                frame,
                project_area,
                focus == CreateIssueFormField::Project,
            );
            self.render_issue_type_field(
                data,
                frame,
                issue_type_area,
                focus == CreateIssueFormField::IssueType,
            );
            let epic_area = chunks[2];
            self.render_epic_parent_field(
                data,
                frame,
                epic_area,
                focus == CreateIssueFormField::EpicParent,
            );
            self.render_summary_field(frame, chunks[3], focus == CreateIssueFormField::Summary);
            self.render_description_field(
                frame,
                chunks[4],
                focus == CreateIssueFormField::Description,
            );
            self.render_assignee_field(
                data,
                frame,
                chunks[5],
                focus == CreateIssueFormField::Assignee,
            );
            self.render_priority_field(
                data,
                frame,
                chunks[6],
                focus == CreateIssueFormField::Priority,
            );
            self.render_errors(data, frame, chunks[7]);
            self.render_submit_button(frame, chunks[8], focus == CreateIssueFormField::Submit);

            // Render dropdown overlays LAST so they appear on top
            self.render_project_dropdown_overlay(frame, project_area, area);
            self.render_issue_type_dropdown_overlay(frame, issue_type_area, area);
            self.render_epic_dropdown_overlay(frame, epic_area, area);

            // Render picker overlays (on top of everything)
            self.assignee_picker.render(frame, area);
            self.priority_picker.render(frame, area);
        } else {
            // Epic or other top-level type layout without parent field
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3), // Project
                    Constraint::Length(3), // Issue Type
                    Constraint::Length(3), // Summary
                    Constraint::Length(6), // Description (multi-line)
                    Constraint::Length(3), // Assignee (optional)
                    Constraint::Length(3), // Priority (optional)
                    Constraint::Length(2), // Errors
                    Constraint::Length(1), // Submit button
                ])
                .split(inner);

            // Render fields
            let project_area = chunks[0];
            let issue_type_area = chunks[1];
            self.render_project_field(
                data,
                frame,
                project_area,
                focus == CreateIssueFormField::Project,
            );
            self.render_issue_type_field(
                data,
                frame,
                issue_type_area,
                focus == CreateIssueFormField::IssueType,
            );
            self.render_summary_field(frame, chunks[2], focus == CreateIssueFormField::Summary);
            self.render_description_field(
                frame,
                chunks[3],
                focus == CreateIssueFormField::Description,
            );
            self.render_assignee_field(
                data,
                frame,
                chunks[4],
                focus == CreateIssueFormField::Assignee,
            );
            self.render_priority_field(
                data,
                frame,
                chunks[5],
                focus == CreateIssueFormField::Priority,
            );
            self.render_errors(data, frame, chunks[6]);
            self.render_submit_button(frame, chunks[7], focus == CreateIssueFormField::Submit);

            // Render dropdown overlays LAST so they appear on top
            self.render_project_dropdown_overlay(frame, project_area, area);
            self.render_issue_type_dropdown_overlay(frame, issue_type_area, area);

            // Render picker overlays (on top of everything)
            self.assignee_picker.render(frame, area);
            self.priority_picker.render(frame, area);
        }
    }

    /// Render the project dropdown field.
    /// Returns the area where the dropdown was rendered (for overlay positioning).
    fn render_project_field(
        &mut self,
        data: &CreateIssueRenderData,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
    ) {
        // Ensure dropdown items are synced from data
        if self.project_dropdown.item_count() != data.projects.len() {
            let items: Vec<DropdownItem> = data
                .projects
                .iter()
                .map(|(key, name)| DropdownItem::new(key.clone(), name.clone()))
                .collect();
            self.project_dropdown.set_items(items);
        }

        // Sync selection only when collapsed (don't interfere with navigation when expanded)
        if !data.form.project_key.is_empty() && !self.project_dropdown.is_expanded() {
            self.project_dropdown.select_by_id(&data.form.project_key);
        }

        // Render the dropdown (collapsed view only)
        self.project_dropdown.render(frame, area, focused);
    }

    /// Render the expanded project dropdown overlay.
    /// This should be called LAST after all other fields are rendered.
    fn render_project_dropdown_overlay(
        &self,
        frame: &mut Frame,
        dropdown_area: Rect,
        screen_area: Rect,
    ) {
        if self.project_dropdown.is_expanded() {
            self.project_dropdown
                .render_expanded_list(frame, dropdown_area, screen_area);
        }
    }

    /// Render the expanded issue type dropdown overlay.
    /// This should be called LAST after all other fields are rendered.
    fn render_issue_type_dropdown_overlay(
        &self,
        frame: &mut Frame,
        dropdown_area: Rect,
        screen_area: Rect,
    ) {
        if self.issue_type_dropdown.is_expanded() {
            self.issue_type_dropdown
                .render_expanded_list(frame, dropdown_area, screen_area);
        }
    }

    /// Render the issue type dropdown field.
    fn render_issue_type_field(
        &mut self,
        data: &CreateIssueRenderData,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
    ) {
        let form = &data.form;

        // Ensure dropdown items are synced from data
        if self.issue_type_dropdown.item_count() != data.issue_types.len() {
            let items: Vec<DropdownItem> = data
                .issue_types
                .iter()
                .map(|t| DropdownItem::new(t.id.clone(), t.name.clone()))
                .collect();
            self.issue_type_dropdown.set_items(items);
        }

        // Sync selection only when collapsed (don't interfere with navigation when expanded)
        if !form.issue_type_id.is_empty() && !self.issue_type_dropdown.is_expanded() {
            self.issue_type_dropdown.select_by_id(&form.issue_type_id);
        }

        // Update placeholder based on state
        if form.project_key.is_empty() {
            self.issue_type_dropdown
                .set_placeholder("Select a project first");
        } else if data.issue_types.is_empty() {
            if data.is_fetching_issue_types {
                self.issue_type_dropdown.set_placeholder("Loading...");
            } else {
                self.issue_type_dropdown
                    .set_placeholder("No issue types available");
            }
        } else {
            self.issue_type_dropdown
                .set_placeholder("Select issue type...");
        }

        // Render the dropdown (collapsed view only)
        self.issue_type_dropdown.render(frame, area, focused);
    }

    /// Render the parent issue input field (for subtasks).
    fn render_parent_field(
        &self,
        _data: &CreateIssueRenderData,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
    ) {
        self.parent_input
            .render_with_label(frame, area, "Parent Issue *", focused);
    }

    /// Render the epic dropdown field (optional).
    fn render_epic_parent_field(
        &mut self,
        data: &CreateIssueRenderData,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
    ) {
        // Ensure dropdown items are synced from data
        if self.epic_dropdown.item_count() != data.epics.len() {
            let items: Vec<DropdownItem> = data
                .epics
                .iter()
                .map(|(key, label)| DropdownItem::new(key.clone(), label.clone()))
                .collect();
            self.epic_dropdown.set_items(items);
        }

        // Sync selection only when collapsed (don't interfere with navigation when expanded)
        if let Some(ref epic_key) = data.form.epic_parent_key {
            if !self.epic_dropdown.is_expanded() {
                self.epic_dropdown.select_by_id(epic_key);
            }
        }

        // Render the dropdown (collapsed view only)
        self.epic_dropdown.render(frame, area, focused);
    }

    /// Render the expanded epic dropdown overlay.
    /// This should be called LAST after all other fields are rendered.
    fn render_epic_dropdown_overlay(
        &self,
        frame: &mut Frame,
        dropdown_area: Rect,
        screen_area: Rect,
    ) {
        if self.epic_dropdown.is_expanded() {
            self.epic_dropdown
                .render_expanded_list(frame, dropdown_area, screen_area);
        }
    }

    /// Render the summary input field.
    fn render_summary_field(&self, frame: &mut Frame, area: Rect, focused: bool) {
        self.summary_input
            .render_with_label(frame, area, "Summary *", focused);
    }

    /// Render the description editor field.
    fn render_description_field(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        self.description_editor
            .render(frame, area, focused, Some(" Description "));
    }

    /// Render the assignee picker field (optional).
    fn render_assignee_field(
        &self,
        data: &CreateIssueRenderData,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
    ) {
        let t = theme();
        let form = &data.form;

        let display_value = if let Some(ref name) = form.assignee_name {
            name.clone()
        } else {
            "Unassigned (optional)".to_string()
        };

        let style = if focused {
            Style::default().fg(t.accent)
        } else if form.assignee_id.is_none() {
            Style::default().fg(t.input_placeholder)
        } else {
            Style::default().fg(t.input_fg)
        };

        let border_style = if focused {
            Style::default().fg(t.border_focused)
        } else {
            Style::default().fg(t.border)
        };

        let title_style = if focused {
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.fg)
        };

        let block = Block::default()
            .title(Span::styled(" Assignee ", title_style))
            .borders(Borders::ALL)
            .border_style(border_style);

        let paragraph = Paragraph::new(display_value).style(style).block(block);

        frame.render_widget(paragraph, area);
    }

    /// Render the priority picker field (optional).
    fn render_priority_field(
        &self,
        data: &CreateIssueRenderData,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
    ) {
        let t = theme();
        let form = &data.form;

        let display_value = if let Some(ref name) = form.priority_name {
            name.clone()
        } else {
            "Default (optional)".to_string()
        };

        let style = if focused {
            Style::default().fg(t.accent)
        } else if form.priority_id.is_none() {
            Style::default().fg(t.input_placeholder)
        } else {
            Style::default().fg(t.input_fg)
        };

        let border_style = if focused {
            Style::default().fg(t.border_focused)
        } else {
            Style::default().fg(t.border)
        };

        let title_style = if focused {
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.fg)
        };

        let block = Block::default()
            .title(Span::styled(" Priority ", title_style))
            .borders(Borders::ALL)
            .border_style(border_style);

        let paragraph = Paragraph::new(display_value).style(style).block(block);

        frame.render_widget(paragraph, area);
    }

    /// Render validation errors.
    fn render_errors(&self, data: &CreateIssueRenderData, frame: &mut Frame, area: Rect) {
        let t = theme();

        if !data.errors.is_empty() {
            let error_text = data.errors.join(", ");
            let paragraph = Paragraph::new(Span::styled(error_text, Style::default().fg(t.error)))
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
        } else if self.submitting {
            let paragraph = Paragraph::new(Span::styled(
                "Creating issue...",
                Style::default().fg(t.warning),
            ))
            .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
        }
    }

    /// Render the submit button.
    fn render_submit_button(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let t = theme();

        let button_style = if focused {
            Style::default()
                .fg(t.selection_fg)
                .bg(t.success)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.success)
        };

        let button_text = if self.submitting {
            " Creating... "
        } else {
            " [Enter] Create Issue "
        };

        let button =
            Paragraph::new(Span::styled(button_text, button_style)).alignment(Alignment::Center);
        frame.render_widget(button, area);
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

    #[test]
    fn test_create_issue_view_new() {
        let view = CreateIssueView::new();
        assert!(view.summary().is_empty());
        assert!(view.description().is_empty());
        assert!(!view.is_submitting());
    }

    #[test]
    fn test_create_issue_view_reset() {
        let mut view = CreateIssueView::new();
        view.set_summary("Test summary");
        view.set_description("Test description");
        view.set_submitting(true);

        view.reset();

        assert!(view.summary().is_empty());
        assert!(view.description().is_empty());
        assert!(!view.is_submitting());
    }

    #[test]
    fn test_create_issue_view_summary() {
        let mut view = CreateIssueView::new();
        view.set_summary("Test summary");
        assert_eq!(view.summary(), "Test summary");
    }

    #[test]
    fn test_create_issue_view_description() {
        let mut view = CreateIssueView::new();
        view.set_description("Line 1\nLine 2");
        assert_eq!(view.description(), "Line 1\nLine 2");
    }

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
    fn test_create_issue_action_equality() {
        assert_eq!(CreateIssueAction::Cancel, CreateIssueAction::Cancel);
        assert_eq!(CreateIssueAction::Submit, CreateIssueAction::Submit);
        assert_eq!(
            CreateIssueAction::FetchIssueTypes("PROJ".to_string()),
            CreateIssueAction::FetchIssueTypes("PROJ".to_string())
        );
        assert_ne!(CreateIssueAction::Cancel, CreateIssueAction::Submit);
    }
}
