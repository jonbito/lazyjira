//! Issue detail view.
//!
//! Displays a single JIRA issue with all its fields in a readable format.
//! Supports scrolling for long descriptions and keyboard navigation.
//! Supports edit mode for summary and description fields.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::api::types::{AtlassianDoc, FieldUpdates, Issue, IssueUpdateRequest, Transition};
use crate::ui::components::{TextEditor, TextInput, TransitionAction, TransitionPicker};
use crate::ui::theme::{issue_type_prefix, priority_style, status_style};

/// Action that can be triggered from the detail view.
#[derive(Debug, Clone, PartialEq)]
pub enum DetailAction {
    /// Go back to the issue list.
    GoBack,
    /// Enter edit mode for the issue.
    EditIssue,
    /// Add a comment (future feature).
    AddComment,
    /// Save the current edit (issue key, update request).
    SaveEdit(String, IssueUpdateRequest),
    /// Show confirmation dialog before discarding changes.
    ConfirmDiscard,
    /// Open the status transition picker.
    OpenTransitionPicker,
    /// Request transitions from the API (issue key, current status).
    FetchTransitions(String, String),
    /// Execute a status transition (issue key, transition ID, optional fields).
    ExecuteTransition(String, String, Option<FieldUpdates>),
    /// Show a message that transition requires fields (not yet supported).
    TransitionRequiresFields(String),
}

/// Which field is currently being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
    /// Editing the summary field.
    Summary,
    /// Editing the description field.
    Description,
}

/// Edit mode state.
#[derive(Debug, Clone)]
pub struct EditState {
    /// Which field is currently being edited.
    pub field: EditField,
    /// The summary input field.
    pub summary_input: TextInput,
    /// The description editor.
    pub description_editor: TextEditor,
    /// Original summary for change detection.
    pub original_summary: String,
    /// Original description for change detection.
    pub original_description: String,
}

/// The issue detail view.
pub struct DetailView {
    /// The issue being displayed.
    issue: Option<Issue>,
    /// Current scroll position.
    scroll: u16,
    /// Maximum scroll position (calculated during render).
    max_scroll: u16,
    /// Content height (calculated during render).
    content_height: u16,
    /// Visible area height (calculated during render).
    visible_height: u16,
    /// Edit mode state (None if not in edit mode).
    edit_state: Option<EditState>,
    /// Whether we are currently saving.
    is_saving: bool,
    /// Transition picker for status changes.
    transition_picker: TransitionPicker,
}

impl DetailView {
    /// Create a new detail view.
    pub fn new() -> Self {
        Self {
            issue: None,
            scroll: 0,
            max_scroll: 0,
            content_height: 0,
            visible_height: 0,
            edit_state: None,
            is_saving: false,
            transition_picker: TransitionPicker::new(),
        }
    }

    /// Set the issue to display.
    pub fn set_issue(&mut self, issue: Issue) {
        self.issue = Some(issue);
        self.scroll = 0;
        self.max_scroll = 0;
        self.edit_state = None;
        self.is_saving = false;
        self.transition_picker.hide();
    }

    /// Clear the current issue.
    pub fn clear(&mut self) {
        self.issue = None;
        self.scroll = 0;
        self.max_scroll = 0;
        self.edit_state = None;
        self.is_saving = false;
        self.transition_picker.hide();
    }

    /// Get a reference to the current issue.
    pub fn issue(&self) -> Option<&Issue> {
        self.issue.as_ref()
    }

    /// Get the current scroll position.
    pub fn scroll(&self) -> u16 {
        self.scroll
    }

    /// Get the maximum scroll position.
    pub fn max_scroll(&self) -> u16 {
        self.max_scroll
    }

    /// Set the maximum scroll position (for testing).
    #[cfg(test)]
    pub fn set_max_scroll(&mut self, max_scroll: u16) {
        self.max_scroll = max_scroll;
    }

    /// Check if the view is in edit mode.
    pub fn is_editing(&self) -> bool {
        self.edit_state.is_some()
    }

    /// Check if there are unsaved changes.
    pub fn has_unsaved_changes(&self) -> bool {
        if let Some(edit_state) = &self.edit_state {
            edit_state.summary_input.value() != edit_state.original_summary
                || edit_state.description_editor.has_changes()
        } else {
            false
        }
    }

    /// Check if we are currently saving.
    pub fn is_saving(&self) -> bool {
        self.is_saving
    }

    /// Set the saving state.
    pub fn set_saving(&mut self, saving: bool) {
        self.is_saving = saving;
    }

    // ========================================================================
    // Transition picker methods
    // ========================================================================

    /// Check if the transition picker is visible.
    pub fn is_transition_picker_visible(&self) -> bool {
        self.transition_picker.is_visible()
    }

    /// Check if transitions are loading.
    pub fn is_transitions_loading(&self) -> bool {
        self.transition_picker.is_loading()
    }

    /// Show the transition picker in loading state.
    ///
    /// This should be called when the user presses 's' to open the picker,
    /// and then the API call to get transitions should be made.
    pub fn show_transition_picker_loading(&mut self) {
        if let Some(issue) = &self.issue {
            let current_status = issue.fields.status.name.clone();
            self.transition_picker.show_loading(&current_status);
        }
    }

    /// Set the available transitions in the picker.
    ///
    /// Call this after receiving the transitions from the API.
    pub fn set_transitions(&mut self, transitions: Vec<Transition>) {
        if let Some(issue) = &self.issue {
            let current_status = issue.fields.status.name.clone();
            self.transition_picker.show(transitions, &current_status);
        }
    }

    /// Hide the transition picker.
    pub fn hide_transition_picker(&mut self) {
        self.transition_picker.hide();
    }

    /// Enter edit mode for the current issue.
    pub fn enter_edit_mode(&mut self) {
        if let Some(issue) = &self.issue {
            let summary = issue.fields.summary.clone();
            let description = issue.description_text();

            let mut summary_input = TextInput::with_value(&summary);
            summary_input.set_placeholder("Enter summary...");

            let description_editor = TextEditor::new(&description);

            self.edit_state = Some(EditState {
                field: EditField::Summary,
                summary_input,
                description_editor,
                original_summary: summary,
                original_description: description,
            });
        }
    }

    /// Exit edit mode without saving.
    pub fn exit_edit_mode(&mut self) {
        self.edit_state = None;
        self.is_saving = false;
    }

    /// Get the current edit field.
    pub fn current_edit_field(&self) -> Option<EditField> {
        self.edit_state.as_ref().map(|s| s.field)
    }

    /// Switch focus to the next field in edit mode.
    fn switch_edit_field(&mut self) {
        if let Some(edit_state) = &mut self.edit_state {
            edit_state.field = match edit_state.field {
                EditField::Summary => EditField::Description,
                EditField::Description => EditField::Summary,
            };
        }
    }

    /// Create an update request from the current edit state.
    fn create_update_request(&self) -> Option<IssueUpdateRequest> {
        let edit_state = self.edit_state.as_ref()?;

        let mut fields = FieldUpdates::default();
        let mut has_changes = false;

        // Check if summary changed
        if edit_state.summary_input.value() != edit_state.original_summary {
            fields.summary = Some(edit_state.summary_input.value().to_string());
            has_changes = true;
        }

        // Check if description changed
        if edit_state.description_editor.has_changes() {
            let new_description = edit_state.description_editor.content();
            fields.description = Some(AtlassianDoc::from_plain_text(&new_description));
            has_changes = true;
        }

        if has_changes {
            Some(IssueUpdateRequest {
                fields: Some(fields),
                update: None,
            })
        } else {
            None
        }
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the application.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<DetailAction> {
        // Handle transition picker first (blocks other input when visible)
        if self.transition_picker.is_visible() {
            return self.handle_transition_picker_input(key);
        }

        // If in edit mode, handle edit-specific input
        if self.edit_state.is_some() {
            return self.handle_edit_input(key);
        }

        // Normal (view) mode input handling
        match (key.code, key.modifiers) {
            // Navigation - go back
            (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Esc, KeyModifiers::NONE) => {
                Some(DetailAction::GoBack)
            }
            // Scroll down
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                self.scroll_down();
                None
            }
            // Scroll up
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                self.scroll_up();
                None
            }
            // Page down
            (KeyCode::PageDown, _) | (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                self.page_down();
                None
            }
            // Page up
            (KeyCode::PageUp, _) | (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                self.page_up();
                None
            }
            // Go to top
            (KeyCode::Home, _) | (KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.scroll = 0;
                None
            }
            // Go to bottom
            (KeyCode::End, _) | (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                self.scroll = self.max_scroll;
                None
            }
            // Edit issue
            (KeyCode::Char('e'), KeyModifiers::NONE) => Some(DetailAction::EditIssue),
            // Add comment
            (KeyCode::Char('c'), KeyModifiers::NONE) => Some(DetailAction::AddComment),
            // Change status (open transition picker)
            (KeyCode::Char('s'), KeyModifiers::NONE) => {
                if let Some(issue) = &self.issue {
                    let issue_key = issue.key.clone();
                    let current_status = issue.fields.status.name.clone();
                    self.show_transition_picker_loading();
                    Some(DetailAction::FetchTransitions(issue_key, current_status))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Handle keyboard input for the transition picker.
    fn handle_transition_picker_input(&mut self, key: KeyEvent) -> Option<DetailAction> {
        if let Some(action) = self.transition_picker.handle_input(key) {
            match action {
                TransitionAction::Execute(transition_id, fields) => {
                    if let Some(issue) = &self.issue {
                        let issue_key = issue.key.clone();
                        Some(DetailAction::ExecuteTransition(issue_key, transition_id, fields))
                    } else {
                        None
                    }
                }
                TransitionAction::RequiresFields(transition_id) => {
                    Some(DetailAction::TransitionRequiresFields(transition_id))
                }
                TransitionAction::Cancel => None,
            }
        } else {
            None
        }
    }

    /// Handle keyboard input in edit mode.
    fn handle_edit_input(&mut self, key: KeyEvent) -> Option<DetailAction> {
        match (key.code, key.modifiers) {
            // Escape - cancel edit (may show confirmation if changes exist)
            (KeyCode::Esc, KeyModifiers::NONE) => {
                if self.has_unsaved_changes() {
                    Some(DetailAction::ConfirmDiscard)
                } else {
                    self.exit_edit_mode();
                    None
                }
            }
            // Ctrl+S - save changes
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                if let Some(issue) = &self.issue {
                    if let Some(update_request) = self.create_update_request() {
                        Some(DetailAction::SaveEdit(issue.key.clone(), update_request))
                    } else {
                        // No changes, just exit edit mode
                        self.exit_edit_mode();
                        None
                    }
                } else {
                    None
                }
            }
            // Tab - switch between fields
            (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::BackTab, KeyModifiers::SHIFT) => {
                self.switch_edit_field();
                None
            }
            // All other input goes to the focused field
            _ => {
                if let Some(edit_state) = &mut self.edit_state {
                    match edit_state.field {
                        EditField::Summary => {
                            edit_state.summary_input.handle_input(key);
                        }
                        EditField::Description => {
                            edit_state.description_editor.handle_input(key);
                        }
                    }
                }
                None
            }
        }
    }

    /// Scroll down by one line.
    fn scroll_down(&mut self) {
        if self.scroll < self.max_scroll {
            self.scroll += 1;
        }
    }

    /// Scroll up by one line.
    fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    /// Scroll down by half a page.
    fn page_down(&mut self) {
        let page_size = self.visible_height / 2;
        self.scroll = (self.scroll + page_size).min(self.max_scroll);
    }

    /// Scroll up by half a page.
    fn page_up(&mut self) {
        let page_size = self.visible_height / 2;
        self.scroll = self.scroll.saturating_sub(page_size);
    }

    /// Render the detail view.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Render edit mode if active
        if self.edit_state.is_some() {
            self.render_edit_mode(frame, area);
            return;
        }

        let Some(issue) = &self.issue else {
            self.render_no_issue(frame, area);
            return;
        };

        // Clone the issue data we need for rendering to avoid borrow issues
        let issue_key = issue.key.clone();
        let issue_type_name = issue.fields.issuetype.name.clone();
        let summary = issue.fields.summary.clone();
        let status = issue.fields.status.clone();
        let priority = issue.fields.priority.clone();
        let assignee_name = issue.assignee_name().to_string();
        let reporter_name = issue
            .reporter()
            .unwrap_or("Unknown")
            .to_string();
        let created = issue.fields.created.clone();
        let updated = issue.fields.updated.clone();
        let labels = issue.fields.labels.clone();
        let components: Vec<String> = issue
            .fields
            .components
            .iter()
            .map(|c| c.name.clone())
            .collect();
        let description = issue.description_text();
        let project_key = issue.project_key().map(|s| s.to_string());

        // Calculate layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header (type + key)
                Constraint::Length(2), // Summary
                Constraint::Length(7), // Metadata
                Constraint::Min(5),    // Description (scrollable)
            ])
            .split(area);

        // Render header
        self.render_header(frame, chunks[0], &issue_type_name, &issue_key);

        // Render summary
        self.render_summary(frame, chunks[1], &summary);

        // Render metadata
        self.render_metadata(
            frame,
            chunks[2],
            &status,
            priority.as_ref(),
            &assignee_name,
            &reporter_name,
            created.as_deref(),
            updated.as_deref(),
            &labels,
            &components,
            project_key.as_deref(),
        );

        // Render description (scrollable)
        self.render_description(frame, chunks[3], &description);

        // Render transition picker (overlay)
        self.transition_picker.render(frame, area);
    }

    /// Render the edit mode interface.
    fn render_edit_mode(&mut self, frame: &mut Frame, area: Rect) {
        // Get issue info for header
        let (issue_key, issue_type_name) = if let Some(issue) = &self.issue {
            (issue.key.clone(), issue.fields.issuetype.name.clone())
        } else {
            return;
        };

        // Extract edit_state to avoid borrowing issues
        let edit_state = match self.edit_state.take() {
            Some(state) => state,
            None => return,
        };

        let current_field = edit_state.field;
        let has_changes = edit_state.summary_input.value() != edit_state.original_summary
            || edit_state.description_editor.content() != edit_state.original_description;

        // Calculate layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header with edit indicator
                Constraint::Length(3), // Summary input
                Constraint::Min(5),    // Description editor
                Constraint::Length(2), // Edit mode hints
            ])
            .split(area);

        // Render header with edit indicator
        let type_prefix = issue_type_prefix(&issue_type_name);
        let edit_indicator = if has_changes { " [*]" } else { "" };
        let saving_indicator = if self.is_saving { " [Saving...]" } else { "" };
        let header_text = format!(
            "{} {} - {} (EDITING){}{}",
            type_prefix, issue_type_name, issue_key, edit_indicator, saving_indicator
        );

        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                header_text,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        frame.render_widget(header, chunks[0]);

        // Put edit_state back to render the fields
        let mut edit_state = edit_state;

        // Render summary input
        let summary_title = if current_field == EditField::Summary {
            " Summary (editing) "
        } else {
            " Summary "
        };
        edit_state.summary_input.render_with_label(
            frame,
            chunks[1],
            summary_title,
            current_field == EditField::Summary,
        );

        // Render description editor
        let description_title = if current_field == EditField::Description {
            " Description (editing) "
        } else {
            " Description "
        };
        edit_state.description_editor.render(
            frame,
            chunks[2],
            current_field == EditField::Description,
            Some(description_title),
        );

        // Put edit_state back
        self.edit_state = Some(edit_state);

        // Render edit mode hints
        let hints = Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": switch field  "),
            Span::styled("Ctrl+S", Style::default().fg(Color::Green)),
            Span::raw(": save  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": cancel"),
        ]);
        let hints_paragraph = Paragraph::new(hints);
        frame.render_widget(hints_paragraph, chunks[3]);
    }

    /// Render when no issue is set.
    fn render_no_issue(&self, frame: &mut Frame, area: Rect) {
        let message = Paragraph::new("No issue selected")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Issue Detail"));
        frame.render_widget(message, area);
    }

    /// Render the header section with issue type and key.
    fn render_header(&self, frame: &mut Frame, area: Rect, issue_type: &str, key: &str) {
        let type_prefix = issue_type_prefix(issue_type);
        let header_text = format!("{} {} - {}", type_prefix, issue_type, key);

        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                header_text,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        frame.render_widget(header, area);
    }

    /// Render the summary section.
    fn render_summary(&self, frame: &mut Frame, area: Rect, summary: &str) {
        let summary_paragraph = Paragraph::new(Line::from(Span::styled(
            summary,
            Style::default().add_modifier(Modifier::BOLD),
        )))
        .wrap(Wrap { trim: true });

        frame.render_widget(summary_paragraph, area);
    }

    /// Render the metadata section.
    #[allow(clippy::too_many_arguments)]
    fn render_metadata(
        &self,
        frame: &mut Frame,
        area: Rect,
        status: &crate::api::types::Status,
        priority: Option<&crate::api::types::Priority>,
        assignee: &str,
        reporter: &str,
        created: Option<&str>,
        updated: Option<&str>,
        labels: &[String],
        components: &[String],
        project: Option<&str>,
    ) {
        let status_sty = status_style(status);
        let priority_sty = priority_style(priority);
        let priority_name = priority.map(|p| p.name.as_str()).unwrap_or("None");

        let mut lines = vec![
            // Status and Priority
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&status.name, status_sty),
                Span::raw("    "),
                Span::styled("Priority: ", Style::default().fg(Color::DarkGray)),
                Span::styled(priority_name, priority_sty),
            ]),
            // Assignee and Reporter
            Line::from(vec![
                Span::styled("Assignee: ", Style::default().fg(Color::DarkGray)),
                Span::raw(assignee),
                Span::raw("    "),
                Span::styled("Reporter: ", Style::default().fg(Color::DarkGray)),
                Span::raw(reporter),
            ]),
        ];

        // Project (if available)
        if let Some(proj) = project {
            lines.push(Line::from(vec![
                Span::styled("Project: ", Style::default().fg(Color::DarkGray)),
                Span::raw(proj),
            ]));
        }

        // Dates
        let created_str = created
            .map(|d| format_date(d))
            .unwrap_or_else(|| "Unknown".to_string());
        let updated_str = updated
            .map(|d| format_date(d))
            .unwrap_or_else(|| "Unknown".to_string());

        lines.push(Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::DarkGray)),
            Span::raw(&created_str),
            Span::raw("    "),
            Span::styled("Updated: ", Style::default().fg(Color::DarkGray)),
            Span::raw(&updated_str),
        ]));

        // Labels
        if !labels.is_empty() {
            let mut label_spans = vec![Span::styled(
                "Labels: ",
                Style::default().fg(Color::DarkGray),
            )];
            for (i, label) in labels.iter().enumerate() {
                if i > 0 {
                    label_spans.push(Span::raw(" "));
                }
                label_spans.push(Span::styled(
                    format!(" {} ", label),
                    Style::default().bg(Color::Blue).fg(Color::White),
                ));
            }
            lines.push(Line::from(label_spans));
        }

        // Components
        if !components.is_empty() {
            let mut comp_spans = vec![Span::styled(
                "Components: ",
                Style::default().fg(Color::DarkGray),
            )];
            for (i, component) in components.iter().enumerate() {
                if i > 0 {
                    comp_spans.push(Span::raw(" "));
                }
                comp_spans.push(Span::styled(
                    format!(" {} ", component),
                    Style::default().bg(Color::Magenta).fg(Color::White),
                ));
            }
            lines.push(Line::from(comp_spans));
        }

        let metadata = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        frame.render_widget(metadata, area);
    }

    /// Render the description section with scrolling.
    fn render_description(&mut self, frame: &mut Frame, area: Rect, description: &str) {
        let description_text = if description.is_empty() {
            "No description provided.".to_string()
        } else {
            description.to_string()
        };

        // Calculate content height for scrolling
        // Estimate: count lines + wrapped lines based on area width
        let inner_width = area.width.saturating_sub(2) as usize; // Account for borders
        let line_count = estimate_line_count(&description_text, inner_width);

        self.content_height = line_count as u16;
        self.visible_height = area.height.saturating_sub(2); // Account for borders

        // Calculate max scroll
        self.max_scroll = self.content_height.saturating_sub(self.visible_height);

        // Ensure scroll is within bounds
        if self.scroll > self.max_scroll {
            self.scroll = self.max_scroll;
        }

        let description_paragraph = Paragraph::new(description_text)
            .block(
                Block::default()
                    .title("Description")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0));

        frame.render_widget(description_paragraph, area);
    }

    /// Render the status bar for the detail view.
    pub fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let issue_key = self
            .issue
            .as_ref()
            .map(|i| i.key.as_str())
            .unwrap_or("No issue");

        // Edit mode status bar
        if self.edit_state.is_some() {
            let unsaved_indicator = if self.has_unsaved_changes() {
                " [*] "
            } else {
                " "
            };

            let saving_indicator = if self.is_saving {
                "Saving..."
            } else {
                ""
            };

            let status_line = Line::from(vec![
                Span::styled(
                    format!(" {} ", issue_key),
                    Style::default().fg(Color::Black).bg(Color::Yellow),
                ),
                Span::styled(
                    " EDITING ",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::styled(unsaved_indicator, Style::default().fg(Color::Red)),
                Span::styled(saving_indicator, Style::default().fg(Color::Cyan)),
            ]);

            let paragraph = Paragraph::new(status_line);
            frame.render_widget(paragraph, area);
            return;
        }

        // Normal mode status bar
        let scroll_info = if self.max_scroll > 0 {
            format!(
                " [scroll: {}/{}]",
                self.scroll + 1,
                self.max_scroll + 1
            )
        } else {
            String::new()
        };

        let status_line = Line::from(vec![
            Span::styled(
                format!(" {} ", issue_key),
                Style::default().fg(Color::Black).bg(Color::Cyan),
            ),
            Span::styled(scroll_info, Style::default().fg(Color::DarkGray)),
            Span::raw(" | "),
            Span::styled(
                "j/k:scroll  q:back  e:edit  s:status  c:comment",
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        let paragraph = Paragraph::new(status_line);
        frame.render_widget(paragraph, area);
    }
}

impl Default for DetailView {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a JIRA date string for display.
///
/// JIRA dates are typically in ISO 8601 format: "2024-01-15T10:00:00.000+0000"
/// This function extracts just the date portion.
fn format_date(date_str: &str) -> String {
    // Extract YYYY-MM-DD from the beginning
    if date_str.len() >= 10 {
        date_str[..10].to_string()
    } else {
        date_str.to_string()
    }
}

/// Estimate the number of lines needed to display text with word wrapping.
fn estimate_line_count(text: &str, width: usize) -> usize {
    if width == 0 {
        return text.lines().count().max(1);
    }

    let mut count = 0;
    for line in text.lines() {
        if line.is_empty() {
            count += 1;
        } else {
            // Rough estimate: divide line length by width, round up
            count += (line.len() + width - 1) / width;
        }
    }
    count.max(1)
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

    fn create_full_test_issue() -> Issue {
        use crate::api::types::{Component, Priority, Project, User};

        Issue {
            id: "1".to_string(),
            key: "TEST-123".to_string(),
            self_url: "https://example.com".to_string(),
            fields: IssueFields {
                summary: "Fix login timeout issue on slow connections".to_string(),
                description: Some(serde_json::json!({
                    "type": "doc",
                    "version": 1,
                    "content": [
                        {
                            "type": "paragraph",
                            "content": [
                                {"type": "text", "text": "When users are on slow connections, the login request times out."}
                            ]
                        }
                    ]
                })),
                status: Status {
                    id: "2".to_string(),
                    name: "In Progress".to_string(),
                    status_category: Some(crate::api::types::StatusCategory {
                        id: 4,
                        key: "indeterminate".to_string(),
                        name: "In Progress".to_string(),
                        color_name: Some("yellow".to_string()),
                    }),
                },
                issuetype: IssueType {
                    id: "1".to_string(),
                    name: "Bug".to_string(),
                    subtask: false,
                    description: None,
                    icon_url: None,
                },
                priority: Some(Priority {
                    id: "2".to_string(),
                    name: "High".to_string(),
                    icon_url: None,
                }),
                assignee: Some(User {
                    account_id: "abc123".to_string(),
                    display_name: "John Doe".to_string(),
                    email_address: None,
                    active: true,
                    avatar_urls: None,
                }),
                reporter: Some(User {
                    account_id: "def456".to_string(),
                    display_name: "Jane Smith".to_string(),
                    email_address: None,
                    active: true,
                    avatar_urls: None,
                }),
                project: Some(Project {
                    id: "10000".to_string(),
                    key: "TEST".to_string(),
                    name: "Test Project".to_string(),
                    avatar_urls: None,
                }),
                labels: vec!["backend".to_string(), "urgent".to_string()],
                components: vec![Component {
                    id: "10001".to_string(),
                    name: "Authentication".to_string(),
                    description: None,
                }],
                created: Some("2024-01-15T10:00:00.000+0000".to_string()),
                updated: Some("2024-01-16T14:30:00.000+0000".to_string()),
                duedate: None,
                story_points: None,
            },
        }
    }

    #[test]
    fn test_new_detail_view() {
        let view = DetailView::new();
        assert!(view.issue.is_none());
        assert_eq!(view.scroll, 0);
        assert_eq!(view.max_scroll, 0);
    }

    #[test]
    fn test_set_issue() {
        let mut view = DetailView::new();
        let issue = create_test_issue("TEST-1", "Test issue");
        view.set_issue(issue);

        assert!(view.issue.is_some());
        assert_eq!(view.issue().unwrap().key, "TEST-1");
        assert_eq!(view.scroll, 0);
    }

    #[test]
    fn test_clear_issue() {
        let mut view = DetailView::new();
        view.set_issue(create_test_issue("TEST-1", "Test"));
        view.scroll = 5;

        view.clear();

        assert!(view.issue.is_none());
        assert_eq!(view.scroll, 0);
    }

    #[test]
    fn test_scroll_down() {
        let mut view = DetailView::new();
        view.max_scroll = 10;
        view.scroll = 0;

        view.scroll_down();
        assert_eq!(view.scroll, 1);

        view.scroll_down();
        assert_eq!(view.scroll, 2);
    }

    #[test]
    fn test_scroll_down_at_max() {
        let mut view = DetailView::new();
        view.max_scroll = 5;
        view.scroll = 5;

        view.scroll_down();
        assert_eq!(view.scroll, 5); // Should not exceed max
    }

    #[test]
    fn test_scroll_up() {
        let mut view = DetailView::new();
        view.scroll = 5;

        view.scroll_up();
        assert_eq!(view.scroll, 4);

        view.scroll_up();
        assert_eq!(view.scroll, 3);
    }

    #[test]
    fn test_scroll_up_at_zero() {
        let mut view = DetailView::new();
        view.scroll = 0;

        view.scroll_up();
        assert_eq!(view.scroll, 0); // Should not go below 0
    }

    #[test]
    fn test_page_down() {
        let mut view = DetailView::new();
        view.visible_height = 20;
        view.max_scroll = 100;
        view.scroll = 0;

        view.page_down();
        assert_eq!(view.scroll, 10); // Half page = 10
    }

    #[test]
    fn test_page_up() {
        let mut view = DetailView::new();
        view.visible_height = 20;
        view.scroll = 50;

        view.page_up();
        assert_eq!(view.scroll, 40); // Half page = 10
    }

    #[test]
    fn test_handle_input_go_back_q() {
        let mut view = DetailView::new();
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert_eq!(action, Some(DetailAction::GoBack));
    }

    #[test]
    fn test_handle_input_go_back_esc() {
        let mut view = DetailView::new();
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert_eq!(action, Some(DetailAction::GoBack));
    }

    #[test]
    fn test_handle_input_scroll_j() {
        let mut view = DetailView::new();
        view.max_scroll = 10;

        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert!(action.is_none());
        assert_eq!(view.scroll, 1);
    }

    #[test]
    fn test_handle_input_scroll_k() {
        let mut view = DetailView::new();
        view.scroll = 5;

        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert!(action.is_none());
        assert_eq!(view.scroll, 4);
    }

    #[test]
    fn test_handle_input_edit() {
        let mut view = DetailView::new();
        let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert_eq!(action, Some(DetailAction::EditIssue));
    }

    #[test]
    fn test_handle_input_comment() {
        let mut view = DetailView::new();
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert_eq!(action, Some(DetailAction::AddComment));
    }

    #[test]
    fn test_handle_input_home() {
        let mut view = DetailView::new();
        view.scroll = 50;

        let key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert!(action.is_none());
        assert_eq!(view.scroll, 0);
    }

    #[test]
    fn test_handle_input_end() {
        let mut view = DetailView::new();
        view.max_scroll = 100;
        view.scroll = 0;

        let key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        let action = view.handle_input(key);

        assert!(action.is_none());
        assert_eq!(view.scroll, 100);
    }

    #[test]
    fn test_format_date() {
        assert_eq!(
            format_date("2024-01-15T10:00:00.000+0000"),
            "2024-01-15"
        );
        assert_eq!(format_date("2024-01-15"), "2024-01-15");
        assert_eq!(format_date("short"), "short");
    }

    #[test]
    fn test_estimate_line_count() {
        // Single line, fits in width
        assert_eq!(estimate_line_count("hello", 80), 1);

        // Multiple lines
        assert_eq!(estimate_line_count("line1\nline2\nline3", 80), 3);

        // Empty line
        assert_eq!(estimate_line_count("line1\n\nline3", 80), 3);

        // Long line that needs wrapping
        let long_line = "a".repeat(100);
        assert_eq!(estimate_line_count(&long_line, 50), 2);

        // Zero width
        assert_eq!(estimate_line_count("hello\nworld", 0), 2);
    }

    #[test]
    fn test_full_issue_display() {
        let mut view = DetailView::new();
        let issue = create_full_test_issue();
        view.set_issue(issue);

        let issue = view.issue().unwrap();
        assert_eq!(issue.key, "TEST-123");
        assert_eq!(issue.assignee_name(), "John Doe");
        assert_eq!(issue.reporter(), Some("Jane Smith"));
        assert_eq!(issue.fields.labels.len(), 2);
        assert_eq!(issue.fields.components.len(), 1);
    }

    #[test]
    fn test_default_impl() {
        let view = DetailView::default();
        assert!(view.issue.is_none());
    }

    #[test]
    fn test_issue_with_missing_fields() {
        let mut view = DetailView::new();
        let issue = create_test_issue("TEST-1", "Minimal issue");
        view.set_issue(issue);

        let issue = view.issue().unwrap();
        assert_eq!(issue.assignee_name(), "Unassigned");
        assert_eq!(issue.priority_name(), "None");
        assert_eq!(issue.reporter(), None);
        assert!(issue.fields.labels.is_empty());
        assert!(issue.fields.components.is_empty());
    }

    // ========================================================================
    // Transition picker tests
    // ========================================================================

    fn create_test_transition(id: &str, name: &str, target_name: &str) -> Transition {
        use crate::api::types::{StatusCategory, TransitionTarget};
        use std::collections::HashMap;

        Transition {
            id: id.to_string(),
            name: name.to_string(),
            to: TransitionTarget {
                id: "1".to_string(),
                name: target_name.to_string(),
                status_category: Some(StatusCategory {
                    id: 2,
                    key: "indeterminate".to_string(),
                    name: "In Progress".to_string(),
                    color_name: Some("yellow".to_string()),
                }),
            },
            fields: HashMap::new(),
        }
    }

    #[test]
    fn test_transition_picker_visibility() {
        let mut view = DetailView::new();
        let issue = create_test_issue("TEST-1", "Test issue");
        view.set_issue(issue);

        assert!(!view.is_transition_picker_visible());
        assert!(!view.is_transitions_loading());

        view.show_transition_picker_loading();

        assert!(view.is_transition_picker_visible());
        assert!(view.is_transitions_loading());

        view.hide_transition_picker();

        assert!(!view.is_transition_picker_visible());
    }

    #[test]
    fn test_set_transitions() {
        let mut view = DetailView::new();
        let issue = create_test_issue("TEST-1", "Test issue");
        view.set_issue(issue);

        view.show_transition_picker_loading();
        assert!(view.is_transitions_loading());

        let transitions = vec![
            create_test_transition("11", "Start Progress", "In Progress"),
            create_test_transition("21", "Done", "Done"),
        ];

        view.set_transitions(transitions);

        assert!(view.is_transition_picker_visible());
        assert!(!view.is_transitions_loading());
    }

    #[test]
    fn test_s_key_opens_transition_picker() {
        let mut view = DetailView::new();
        let issue = create_test_issue("TEST-1", "Test issue");
        view.set_issue(issue);

        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert!(matches!(action, Some(DetailAction::FetchTransitions(_, _))));
        assert!(view.is_transition_picker_visible());
        assert!(view.is_transitions_loading());
    }

    #[test]
    fn test_s_key_without_issue_does_nothing() {
        let mut view = DetailView::new();

        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert!(action.is_none());
        assert!(!view.is_transition_picker_visible());
    }

    #[test]
    fn test_transition_picker_blocks_other_input() {
        let mut view = DetailView::new();
        let issue = create_test_issue("TEST-1", "Test issue");
        view.set_issue(issue);

        view.show_transition_picker_loading();

        // 'q' should not go back while transition picker is visible
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = view.handle_input(key);

        // Cancel action is returned when pressing q/Esc in transition picker
        assert!(action.is_none() || matches!(action, Some(DetailAction::GoBack)) == false);
    }

    #[test]
    fn test_transition_picker_cancel() {
        let mut view = DetailView::new();
        let issue = create_test_issue("TEST-1", "Test issue");
        view.set_issue(issue);

        let transitions = vec![create_test_transition("11", "Start", "In Progress")];
        view.set_transitions(transitions);

        assert!(view.is_transition_picker_visible());

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert!(action.is_none()); // Cancel returns None
        assert!(!view.is_transition_picker_visible());
    }

    #[test]
    fn test_transition_execute() {
        let mut view = DetailView::new();
        let issue = create_test_issue("TEST-1", "Test issue");
        view.set_issue(issue);

        let transitions = vec![create_test_transition("11", "Start Progress", "In Progress")];
        view.set_transitions(transitions);

        // Press Enter to select the transition
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = view.handle_input(key);

        match action {
            Some(DetailAction::ExecuteTransition(issue_key, transition_id, _)) => {
                assert_eq!(issue_key, "TEST-1");
                assert_eq!(transition_id, "11");
            }
            _ => panic!("Expected ExecuteTransition action"),
        }
        assert!(!view.is_transition_picker_visible());
    }

    #[test]
    fn test_set_issue_hides_transition_picker() {
        let mut view = DetailView::new();
        let issue = create_test_issue("TEST-1", "Test issue");
        view.set_issue(issue);

        view.show_transition_picker_loading();
        assert!(view.is_transition_picker_visible());

        // Setting a new issue should hide the transition picker
        let new_issue = create_test_issue("TEST-2", "Another issue");
        view.set_issue(new_issue);

        assert!(!view.is_transition_picker_visible());
    }

    #[test]
    fn test_clear_hides_transition_picker() {
        let mut view = DetailView::new();
        let issue = create_test_issue("TEST-1", "Test issue");
        view.set_issue(issue);

        view.show_transition_picker_loading();
        assert!(view.is_transition_picker_visible());

        view.clear();

        assert!(!view.is_transition_picker_visible());
    }
}
