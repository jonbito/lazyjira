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

use crate::app::{App, CreateIssueFormField};
use crate::ui::components::{TextEditor, TextInput};
use crate::ui::theme::theme;

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
}

/// The create issue view for adding new JIRA issues.
///
/// This view renders as a modal overlay on top of the issue list and provides
/// a form with fields for project, issue type, summary, description, assignee,
/// and priority.
pub struct CreateIssueView {
    /// Summary text input.
    summary_input: TextInput,
    /// Description text editor.
    description_editor: TextEditor,
    /// Project picker index (for navigating through projects).
    project_picker_index: usize,
    /// Issue type picker index.
    issue_type_picker_index: usize,
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

        Self {
            summary_input,
            description_editor: TextEditor::empty(),
            project_picker_index: 0,
            issue_type_picker_index: 0,
            submitting: false,
        }
    }

    /// Reset the view to initial state.
    pub fn reset(&mut self) {
        self.summary_input.clear();
        self.description_editor = TextEditor::empty();
        self.project_picker_index = 0;
        self.issue_type_picker_index = 0;
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

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the parent.
    pub fn handle_input(&mut self, app: &mut App, key: KeyEvent) -> Option<CreateIssueAction> {
        // Don't handle input while submitting
        if self.submitting {
            return None;
        }

        let focus = app.create_issue_focus();

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
            // Enter in other fields - move to next field (except description which uses Enter)
            (KeyCode::Enter, KeyModifiers::NONE) if focus != CreateIssueFormField::Description => {
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

    /// Handle project picker input.
    fn handle_project_input(&mut self, app: &mut App, key: KeyEvent) -> Option<CreateIssueAction> {
        let projects = Self::get_available_projects(app);
        if projects.is_empty() {
            return None;
        }

        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                if self.project_picker_index > 0 {
                    self.project_picker_index -= 1;
                    self.update_selected_project(app, &projects);
                }
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.project_picker_index < projects.len() - 1 {
                    self.project_picker_index += 1;
                    self.update_selected_project(app, &projects);
                }
                None
            }
            _ => None,
        }
    }

    /// Handle issue type picker input.
    fn handle_issue_type_input(
        &mut self,
        app: &mut App,
        key: KeyEvent,
    ) -> Option<CreateIssueAction> {
        let issue_types = app.available_issue_types();
        if issue_types.is_empty() {
            return None;
        }

        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                if self.issue_type_picker_index > 0 {
                    self.issue_type_picker_index -= 1;
                    self.update_selected_issue_type(app);
                }
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.issue_type_picker_index < issue_types.len() - 1 {
                    self.issue_type_picker_index += 1;
                    self.update_selected_issue_type(app);
                }
                None
            }
            _ => None,
        }
    }

    /// Handle assignee picker input (placeholder - optional field).
    fn handle_assignee_input(
        &mut self,
        _app: &mut App,
        _key: KeyEvent,
    ) -> Option<CreateIssueAction> {
        // TODO: Implement assignee picker in future task
        // For now, assignee is optional and can be left empty
        None
    }

    /// Handle priority picker input (placeholder - optional field).
    fn handle_priority_input(
        &mut self,
        _app: &mut App,
        _key: KeyEvent,
    ) -> Option<CreateIssueAction> {
        // TODO: Implement priority picker in future task
        // For now, priority is optional and can be left empty
        None
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
    fn update_selected_project(&mut self, app: &mut App, projects: &[(String, String)]) {
        if let Some((key, name)) = projects.get(self.project_picker_index) {
            let form = app.create_issue_form_mut();
            form.project_key = key.clone();
            form.project_name = name.clone();

            // Clear issue type when project changes
            form.issue_type_id.clear();
            form.issue_type_name.clear();
            self.issue_type_picker_index = 0;

            // Request to fetch issue types for this project
            app.set_pending_fetch_issue_types(true);
        }
    }

    /// Update the selected issue type in app state.
    fn update_selected_issue_type(&mut self, app: &mut App) {
        // Clone the values first to avoid borrow issues
        let selected = app
            .available_issue_types()
            .get(self.issue_type_picker_index)
            .map(|t| (t.id.clone(), t.name.clone()));

        if let Some((id, name)) = selected {
            let form = app.create_issue_form_mut();
            form.issue_type_id = id;
            form.issue_type_name = name;
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

        // Update picker indices based on selected values
        let projects = Self::get_available_projects(app);
        if let Some(idx) = projects
            .iter()
            .position(|(key, _)| key == &form.project_key)
        {
            self.project_picker_index = idx;
        }

        let issue_types = app.available_issue_types();
        if let Some(idx) = issue_types.iter().position(|t| t.id == form.issue_type_id) {
            self.issue_type_picker_index = idx;
        }
    }

    /// Render the create issue view as a modal overlay.
    pub fn render(&mut self, app: &App, frame: &mut Frame, area: Rect) {
        // Calculate dialog size - form needs more height for description
        let dialog_width = 70u16.min(area.width.saturating_sub(4));
        let dialog_height = 24u16.min(area.height.saturating_sub(4));

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

        // Create layout for form fields
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

        let focus = app.create_issue_focus();

        // Render fields
        self.render_project_field(
            app,
            frame,
            chunks[0],
            focus == CreateIssueFormField::Project,
        );
        self.render_issue_type_field(
            app,
            frame,
            chunks[1],
            focus == CreateIssueFormField::IssueType,
        );
        self.render_summary_field(frame, chunks[2], focus == CreateIssueFormField::Summary);
        self.render_description_field(frame, chunks[3], focus == CreateIssueFormField::Description);
        self.render_assignee_field(
            app,
            frame,
            chunks[4],
            focus == CreateIssueFormField::Assignee,
        );
        self.render_priority_field(
            app,
            frame,
            chunks[5],
            focus == CreateIssueFormField::Priority,
        );

        // Render errors if present
        self.render_errors(app, frame, chunks[6]);

        // Render submit button
        self.render_submit_button(frame, chunks[7], focus == CreateIssueFormField::Submit);
    }

    /// Render the project picker field.
    fn render_project_field(&self, app: &App, frame: &mut Frame, area: Rect, focused: bool) {
        let t = theme();
        let projects = Self::get_available_projects(app);
        let form = app.create_issue_form();

        let display_value = if form.project_key.is_empty() {
            if projects.is_empty() {
                "No projects available".to_string()
            } else {
                "← Select project →".to_string()
            }
        } else {
            format!("{} ({})", form.project_name, form.project_key)
        };

        let style = if focused {
            Style::default().fg(t.accent)
        } else if form.project_key.is_empty() {
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
            .title(Span::styled(" Project * ", title_style))
            .borders(Borders::ALL)
            .border_style(border_style);

        let paragraph = Paragraph::new(display_value).style(style).block(block);

        frame.render_widget(paragraph, area);
    }

    /// Render the issue type picker field.
    fn render_issue_type_field(&self, app: &App, frame: &mut Frame, area: Rect, focused: bool) {
        let t = theme();
        let issue_types = app.available_issue_types();
        let form = app.create_issue_form();

        let display_value = if form.issue_type_id.is_empty() {
            if form.project_key.is_empty() {
                "Select a project first".to_string()
            } else if issue_types.is_empty() {
                if app.is_fetch_issue_types_pending() {
                    "Loading...".to_string()
                } else {
                    "No issue types available".to_string()
                }
            } else {
                "← Select issue type →".to_string()
            }
        } else {
            form.issue_type_name.clone()
        };

        let style = if focused {
            Style::default().fg(t.accent)
        } else if form.issue_type_id.is_empty() {
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
            .title(Span::styled(" Issue Type * ", title_style))
            .borders(Borders::ALL)
            .border_style(border_style);

        let paragraph = Paragraph::new(display_value).style(style).block(block);

        frame.render_widget(paragraph, area);
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
    fn render_assignee_field(&self, app: &App, frame: &mut Frame, area: Rect, focused: bool) {
        let t = theme();
        let form = app.create_issue_form();

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
    fn render_priority_field(&self, app: &App, frame: &mut Frame, area: Rect, focused: bool) {
        let t = theme();
        let form = app.create_issue_form();

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
    fn render_errors(&self, app: &App, frame: &mut Frame, area: Rect) {
        let t = theme();
        let errors = app.create_issue_errors();

        if !errors.is_empty() {
            let error_text = errors.join(", ");
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
