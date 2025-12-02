//! Filter panel view for filtering issues.
//!
//! Provides a multi-column filter panel with sections for status, assignee,
//! project, labels, and sprint filters. Supports keyboard navigation and
//! generates JQL queries from selected filters.

use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::api::types::{FilterOptions, FilterState, SprintFilter};
use crate::ui::components::{MultiSelect, SelectItem};
use crate::ui::theme::theme;

/// Actions that can be returned from the filter panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterPanelAction {
    /// Apply the current filters and close the panel.
    Apply(FilterState),
    /// Cancel and close the panel without applying.
    Cancel,
}

/// The type of filter section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterSectionType {
    Status,
    Assignee,
    Project,
    Labels,
    Sprint,
    Epic,
}

/// The filter panel view.
pub struct FilterPanelView {
    /// Whether the panel is visible.
    visible: bool,
    /// The currently focused section index.
    focused_section: usize,
    /// Status multi-select.
    status_select: MultiSelect,
    /// Assignee multi-select.
    assignee_select: MultiSelect,
    /// Project multi-select (single select behavior).
    project_select: MultiSelect,
    /// Labels multi-select.
    labels_select: MultiSelect,
    /// Sprint multi-select.
    sprint_select: MultiSelect,
    /// Epic multi-select.
    epic_select: MultiSelect,
    /// Whether "Assigned to me" is selected.
    assigned_to_me: bool,
    /// Whether "Current sprint" is selected.
    current_sprint: bool,
    /// The filter options available.
    options: FilterOptions,
    /// Section types in order.
    sections: Vec<FilterSectionType>,
}

impl FilterPanelView {
    /// Create a new filter panel view.
    pub fn new() -> Self {
        Self {
            visible: false,
            focused_section: 0,
            status_select: MultiSelect::new("Status"),
            assignee_select: MultiSelect::new("Assignee"),
            project_select: MultiSelect::new("Project"),
            labels_select: MultiSelect::new("Labels"),
            sprint_select: MultiSelect::new("Sprint"),
            epic_select: MultiSelect::new("Epic"),
            assigned_to_me: false,
            current_sprint: false,
            options: FilterOptions::default(),
            sections: vec![
                FilterSectionType::Project,
                FilterSectionType::Epic,
                FilterSectionType::Status,
                FilterSectionType::Assignee,
                FilterSectionType::Labels,
                FilterSectionType::Sprint,
            ],
        }
    }

    /// Show the filter panel.
    pub fn show(&mut self) {
        self.visible = true;
        self.focused_section = 0;
    }

    /// Show the filter panel with existing filter state.
    pub fn show_with_state(&mut self, state: &FilterState) {
        self.visible = true;
        self.focused_section = 0;

        // Restore selections from filter state
        // Status labels need to be converted back to IDs
        let status_ids: HashSet<String> = state
            .statuses
            .iter()
            .filter_map(|label| {
                self.options
                    .statuses
                    .iter()
                    .find(|o| o.label == *label)
                    .map(|o| o.id.clone())
            })
            .collect();
        self.status_select.set_selected(status_ids);

        // Restore assignee selections - include __me__ if assigned_to_me is set
        let mut assignee_ids: HashSet<String> = state.assignees.iter().cloned().collect();
        if state.assignee_is_me {
            assignee_ids.insert("__me__".to_string());
        }
        self.assignee_select.set_selected(assignee_ids);
        self.assigned_to_me = state.assignee_is_me;

        if let Some(project) = &state.project {
            let mut selected = HashSet::new();
            selected.insert(project.clone());
            self.project_select.set_selected(selected);
        } else {
            self.project_select.set_selected(HashSet::new());
        }

        self.labels_select
            .set_selected(state.labels.iter().cloned().collect());

        match &state.sprint {
            Some(SprintFilter::Current) => {
                self.current_sprint = true;
                let mut selected = HashSet::new();
                selected.insert("__current__".to_string());
                self.sprint_select.set_selected(selected);
            }
            Some(SprintFilter::Specific(id)) => {
                self.current_sprint = false;
                let mut selected = HashSet::new();
                selected.insert(id.clone());
                self.sprint_select.set_selected(selected);
            }
            None => {
                self.current_sprint = false;
                self.sprint_select.set_selected(HashSet::new());
            }
        }

        // Restore epic selections
        self.epic_select
            .set_selected(state.epics.iter().cloned().collect());
    }

    /// Hide the filter panel.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if the filter panel is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set the available filter options.
    pub fn set_options(&mut self, options: FilterOptions) {
        // Convert to SelectItems
        let status_items: Vec<SelectItem> = options
            .statuses
            .iter()
            .map(|o| SelectItem::new(&o.id, &o.label))
            .collect();
        self.status_select.set_items(status_items);

        // Add "Assigned to me" as a special option in users
        let mut user_items: Vec<SelectItem> = vec![SelectItem::new("__me__", "Assigned to me")];
        user_items.extend(
            options
                .users
                .iter()
                .map(|o| SelectItem::new(&o.id, &o.label)),
        );
        self.assignee_select.set_items(user_items);

        let project_items: Vec<SelectItem> = options
            .projects
            .iter()
            .map(|o| SelectItem::new(&o.id, &o.label))
            .collect();
        self.project_select.set_items(project_items);

        let label_items: Vec<SelectItem> = options
            .labels
            .iter()
            .map(|o| SelectItem::new(&o.id, &o.label))
            .collect();
        self.labels_select.set_items(label_items);

        // Add "Current Sprint" as a special option
        let mut sprint_items: Vec<SelectItem> =
            vec![SelectItem::new("__current__", "Current Sprint")];
        sprint_items.extend(
            options
                .sprints
                .iter()
                .map(|o| SelectItem::new(&o.id, &o.label)),
        );
        self.sprint_select.set_items(sprint_items);

        let epic_items: Vec<SelectItem> = options
            .epics
            .iter()
            .map(|o| SelectItem::new(&o.id, &o.label))
            .collect();
        self.epic_select.set_items(epic_items);

        self.options = options;
    }

    /// Get the currently focused section type.
    fn focused_section_type(&self) -> FilterSectionType {
        self.sections[self.focused_section]
    }

    /// Get a mutable reference to the currently focused multi-select.
    fn focused_multiselect(&mut self) -> &mut MultiSelect {
        match self.focused_section_type() {
            FilterSectionType::Status => &mut self.status_select,
            FilterSectionType::Assignee => &mut self.assignee_select,
            FilterSectionType::Project => &mut self.project_select,
            FilterSectionType::Labels => &mut self.labels_select,
            FilterSectionType::Sprint => &mut self.sprint_select,
            FilterSectionType::Epic => &mut self.epic_select,
        }
    }

    /// Move to the next section.
    fn next_section(&mut self) {
        self.focused_section = (self.focused_section + 1) % self.sections.len();
    }

    /// Move to the previous section.
    fn prev_section(&mut self) {
        if self.focused_section == 0 {
            self.focused_section = self.sections.len() - 1;
        } else {
            self.focused_section -= 1;
        }
    }

    /// Build the filter state from current selections.
    fn build_filter_state(&self) -> FilterState {
        let mut state = FilterState::new();

        // Get selected statuses (using labels, not IDs for JQL)
        for id in self.status_select.selected() {
            // Find the label for this status ID
            if let Some(opt) = self.options.statuses.iter().find(|o| o.id == *id) {
                state.statuses.push(opt.label.clone());
            }
        }

        // Handle assignee
        if self.assignee_select.is_selected("__me__") {
            state.assignee_is_me = true;
        } else {
            for id in self.assignee_select.selected() {
                if id != "__me__" {
                    state.assignees.push(id.clone());
                }
            }
        }

        // Get selected project (single select - take first)
        if let Some(project_id) = self.project_select.selected().iter().next() {
            state.project = Some(project_id.clone());
        }

        // Get selected labels
        for label in self.labels_select.selected() {
            state.labels.push(label.clone());
        }

        // Handle sprint
        if self.sprint_select.is_selected("__current__") {
            state.sprint = Some(SprintFilter::Current);
        } else if let Some(sprint_id) = self
            .sprint_select
            .selected()
            .iter()
            .find(|id| *id != "__current__")
        {
            state.sprint = Some(SprintFilter::Specific(sprint_id.clone()));
        }

        // Get selected epics
        for epic in self.epic_select.selected() {
            state.epics.push(epic.clone());
        }

        state
    }

    /// Clear all filter selections.
    fn clear_all(&mut self) {
        self.status_select.clear_selection();
        self.assignee_select.clear_selection();
        self.project_select.clear_selection();
        self.labels_select.clear_selection();
        self.sprint_select.clear_selection();
        self.epic_select.clear_selection();
        self.assigned_to_me = false;
        self.current_sprint = false;
    }

    /// Handle keyboard input.
    ///
    /// Returns an action if one should be performed.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<FilterPanelAction> {
        match (key.code, key.modifiers) {
            // Apply filters
            (KeyCode::Enter, KeyModifiers::NONE) => {
                let state = self.build_filter_state();
                self.hide();
                return Some(FilterPanelAction::Apply(state));
            }
            // Cancel with Esc or q
            (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.hide();
                return Some(FilterPanelAction::Cancel);
            }
            // Clear all filters
            (KeyCode::Char('c'), KeyModifiers::NONE) => {
                self.clear_all();
            }
            // Switch sections with Tab, h/l, or arrow keys
            (KeyCode::Tab, KeyModifiers::NONE)
            | (KeyCode::Char('l'), KeyModifiers::NONE)
            | (KeyCode::Right, _) => {
                self.next_section();
            }
            (KeyCode::BackTab, KeyModifiers::SHIFT)
            | (KeyCode::Char('h'), KeyModifiers::NONE)
            | (KeyCode::Left, _) => {
                self.prev_section();
            }
            // Delegate to focused multi-select
            _ => {
                self.focused_multiselect().handle_input(key);
            }
        }
        None
    }

    /// Render the filter panel.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate panel size (80% of screen, centered)
        let panel_width = (area.width as f32 * 0.85) as u16;
        let panel_height = (area.height as f32 * 0.80) as u16;
        let panel_x = (area.width - panel_width) / 2;
        let panel_y = (area.height - panel_height) / 2;

        let panel_area = Rect::new(panel_x, panel_y, panel_width, panel_height);

        let t = theme();

        // Clear the background
        frame.render_widget(Clear, panel_area);

        // Create outer block
        let outer_block = Block::default()
            .title(" Filters ")
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.border_focused));

        let inner_area = outer_block.inner(panel_area);
        frame.render_widget(outer_block, panel_area);

        // Split into sections: content and footer
        let content_footer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(inner_area);

        // Split content into 6 columns for filter sections
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 6), // Project
                Constraint::Ratio(1, 6), // Epic
                Constraint::Ratio(1, 6), // Status
                Constraint::Ratio(1, 6), // Assignee
                Constraint::Ratio(1, 6), // Labels
                Constraint::Ratio(1, 6), // Sprint
            ])
            .split(content_footer[0]);

        // Render each section
        self.project_select
            .render(frame, columns[0], self.focused_section == 0);
        self.epic_select
            .render(frame, columns[1], self.focused_section == 1);
        self.status_select
            .render(frame, columns[2], self.focused_section == 2);
        self.assignee_select
            .render(frame, columns[3], self.focused_section == 3);
        self.labels_select
            .render(frame, columns[4], self.focused_section == 4);
        self.sprint_select
            .render(frame, columns[5], self.focused_section == 5);

        // Render footer with help
        let help_text = Line::from(vec![
            Span::styled("h/l", Style::default().fg(t.warning)),
            Span::raw(": switch section  "),
            Span::styled("j/k", Style::default().fg(t.warning)),
            Span::raw(": navigate  "),
            Span::styled("Space", Style::default().fg(t.warning)),
            Span::raw(": toggle  "),
            Span::styled("Enter", Style::default().fg(t.warning)),
            Span::raw(": apply  "),
            Span::styled("c", Style::default().fg(t.warning)),
            Span::raw(": clear  "),
            Span::styled("q/Esc", Style::default().fg(t.warning)),
            Span::raw(": cancel"),
        ]);

        let footer = Paragraph::new(help_text).style(Style::default().fg(t.muted));
        frame.render_widget(footer, content_footer[1]);
    }
}

impl Default for FilterPanelView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::FilterOption;

    fn create_test_options() -> FilterOptions {
        let mut options = FilterOptions::new();
        options.statuses = vec![
            FilterOption::new("1", "Open"),
            FilterOption::new("2", "In Progress"),
            FilterOption::new("3", "Done"),
        ];
        options.projects = vec![
            FilterOption::new("PROJ", "Project One"),
            FilterOption::new("TEST", "Test Project"),
        ];
        options.labels = vec![
            FilterOption::new("bug", "bug"),
            FilterOption::new("feature", "feature"),
        ];
        options
    }

    #[test]
    fn test_new() {
        let view = FilterPanelView::new();
        assert!(!view.is_visible());
        assert_eq!(view.focused_section, 0);
    }

    #[test]
    fn test_show_hide() {
        let mut view = FilterPanelView::new();
        assert!(!view.is_visible());

        view.show();
        assert!(view.is_visible());

        view.hide();
        assert!(!view.is_visible());
    }

    #[test]
    fn test_section_navigation() {
        let mut view = FilterPanelView::new();
        assert_eq!(view.focused_section, 0);

        view.next_section();
        assert_eq!(view.focused_section, 1);

        view.next_section();
        assert_eq!(view.focused_section, 2);

        view.prev_section();
        assert_eq!(view.focused_section, 1);

        // Wrap around (now 6 sections: Project, Epic, Status, Assignee, Labels, Sprint)
        view.focused_section = 5;
        view.next_section();
        assert_eq!(view.focused_section, 0);

        view.prev_section();
        assert_eq!(view.focused_section, 5);
    }

    #[test]
    fn test_set_options() {
        let mut view = FilterPanelView::new();
        view.set_options(create_test_options());

        // Status select should have items
        assert!(!view.status_select.is_empty());
        // Assignee should have "Assigned to me" plus any users
        assert!(!view.assignee_select.is_empty());
    }

    #[test]
    fn test_build_filter_state_empty() {
        let mut view = FilterPanelView::new();
        view.set_options(create_test_options());

        let state = view.build_filter_state();
        assert!(state.is_empty());
    }

    #[test]
    fn test_handle_input_cancel() {
        let mut view = FilterPanelView::new();
        view.show();

        let action = view.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(action, Some(FilterPanelAction::Cancel));
        assert!(!view.is_visible());
    }

    #[test]
    fn test_handle_input_apply() {
        let mut view = FilterPanelView::new();
        view.show();

        let action = view.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(action, Some(FilterPanelAction::Apply(_))));
        assert!(!view.is_visible());
    }

    #[test]
    fn test_handle_input_tab() {
        let mut view = FilterPanelView::new();
        view.show();

        assert_eq!(view.focused_section, 0);
        view.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(view.focused_section, 1);
    }

    #[test]
    fn test_clear_all() {
        let mut view = FilterPanelView::new();
        view.set_options(create_test_options());

        // Select some items
        view.status_select.toggle_current();

        view.clear_all();
        assert_eq!(view.status_select.selected_count(), 0);
    }

    #[test]
    fn test_show_with_state() {
        let mut view = FilterPanelView::new();
        view.set_options(create_test_options());

        let mut state = FilterState::new();
        state.toggle_status("Open");
        state.toggle_assigned_to_me();

        view.show_with_state(&state);
        assert!(view.is_visible());
        // The state should be restored - status "Open" corresponds to ID "1"
        assert!(view.status_select.is_selected("1"));
    }
}
