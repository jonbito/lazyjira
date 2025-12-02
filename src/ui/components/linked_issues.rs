//! Linked issues section component.
//!
//! Displays linked issues, subtasks, and parent issue for an issue detail view.
//! Supports keyboard navigation and expand/collapse functionality.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::api::types::{IssueLink, ParentIssue, Status, Subtask};

/// Action that can be triggered from the linked issues section.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkedIssuesAction {
    /// Navigate to the selected issue.
    Navigate(String),
    /// Delete a link (link ID, display description for confirmation).
    Delete(String, String),
}

/// Direction of a link relationship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinkDirection {
    /// Other issue affects this one (inward link).
    Inward,
    /// This issue affects other (outward link).
    Outward,
}

/// A display-ready representation of a linked issue.
#[derive(Debug, Clone)]
pub struct DisplayLink {
    /// The link ID (for deletion).
    pub link_id: String,
    /// The issue key.
    pub key: String,
    /// The issue summary.
    pub summary: String,
    /// The issue status.
    pub status: Status,
    /// The link description (e.g., "blocks", "is blocked by").
    pub link_description: String,
    /// The direction of the link.
    pub direction: LinkDirection,
}

/// The linked issues section component.
///
/// Displays parent issue, linked issues, and subtasks with navigation support.
#[derive(Debug)]
pub struct LinkedIssuesSection {
    /// Processed display links.
    links: Vec<DisplayLink>,
    /// Subtasks of the issue.
    subtasks: Vec<Subtask>,
    /// Parent issue (for subtasks).
    parent: Option<ParentIssue>,
    /// Currently selected item index.
    selected: usize,
    /// Whether the section is expanded.
    expanded: bool,
    /// Whether this section has focus.
    focused: bool,
}

impl LinkedIssuesSection {
    /// Create a new linked issues section.
    pub fn new(links: &[IssueLink], subtasks: &[Subtask], parent: Option<ParentIssue>) -> Self {
        let display_links: Vec<DisplayLink> = links
            .iter()
            .filter_map(|link| {
                if let Some(inward) = &link.inward_issue {
                    Some(DisplayLink {
                        link_id: link.id.clone(),
                        key: inward.key.clone(),
                        summary: inward.fields.summary.clone(),
                        status: inward.fields.status.clone(),
                        link_description: link.link_type.inward.clone(),
                        direction: LinkDirection::Inward,
                    })
                } else {
                    link.outward_issue.as_ref().map(|outward| DisplayLink {
                        link_id: link.id.clone(),
                        key: outward.key.clone(),
                        summary: outward.fields.summary.clone(),
                        status: outward.fields.status.clone(),
                        link_description: link.link_type.outward.clone(),
                        direction: LinkDirection::Outward,
                    })
                }
            })
            .collect();

        Self {
            links: display_links,
            subtasks: subtasks.to_vec(),
            parent,
            selected: 0,
            expanded: true,
            focused: false,
        }
    }

    /// Create an empty linked issues section.
    pub fn empty() -> Self {
        Self {
            links: Vec::new(),
            subtasks: Vec::new(),
            parent: None,
            selected: 0,
            expanded: true,
            focused: false,
        }
    }

    /// Get the total number of items (parent + links + subtasks).
    pub fn total_items(&self) -> usize {
        let parent_count = if self.parent.is_some() { 1 } else { 0 };
        parent_count + self.links.len() + self.subtasks.len()
    }

    /// Check if the section is empty.
    pub fn is_empty(&self) -> bool {
        self.total_items() == 0
    }

    /// Check if the section is expanded.
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Toggle expand/collapse.
    pub fn toggle_expand(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Set focused state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Check if the section is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Get the currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Move selection down.
    pub fn select_next(&mut self) {
        if self.selected < self.total_items().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Move selection up.
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Get the key of the currently selected issue.
    pub fn selected_key(&self) -> Option<String> {
        let mut index = self.selected;

        // Parent first
        if let Some(parent) = &self.parent {
            if index == 0 {
                return Some(parent.key.clone());
            }
            index -= 1;
        }

        // Then links
        if index < self.links.len() {
            return Some(self.links[index].key.clone());
        }
        index -= self.links.len();

        // Then subtasks
        if index < self.subtasks.len() {
            return Some(self.subtasks[index].key.clone());
        }

        None
    }

    /// Get the currently selected link (if a link is selected, not parent or subtask).
    ///
    /// Returns the link's ID and a description for confirmation dialog.
    pub fn selected_link(&self) -> Option<(String, String)> {
        let mut index = self.selected;

        // Skip parent
        if self.parent.is_some() {
            if index == 0 {
                return None; // Parent is selected, not a link
            }
            index -= 1;
        }

        // Check if we're in the links range
        if index < self.links.len() {
            let link = &self.links[index];
            let description = format!("{} ({})", link.key, link.link_description);
            return Some((link.link_id.clone(), description));
        }

        // Subtasks are not deletable links
        None
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the parent view.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<LinkedIssuesAction> {
        if !self.focused || self.is_empty() {
            return None;
        }

        match (key.code, key.modifiers) {
            // Navigate down
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                if self.expanded {
                    self.select_next();
                }
                None
            }
            // Navigate up
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                if self.expanded {
                    self.select_previous();
                }
                None
            }
            // Select/navigate to linked issue
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if self.expanded {
                    self.selected_key().map(LinkedIssuesAction::Navigate)
                } else {
                    None
                }
            }
            // Toggle expand/collapse
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.toggle_expand();
                None
            }
            // Delete selected link
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                if self.expanded {
                    if let Some((link_id, description)) = self.selected_link() {
                        Some(LinkedIssuesAction::Delete(link_id, description))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Render the linked issues section.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if self.is_empty() {
            return;
        }

        let title = if self.expanded {
            format!(" Related Issues \u{25BC} ({}) ", self.total_items())
        } else {
            format!(" Related Issues \u{25B6} ({}) ", self.total_items())
        };

        let border_color = if self.focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        if !self.expanded {
            frame.render_widget(block, area);
            return;
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();
        let mut item_index = 0;

        // Parent issue
        if let Some(parent) = &self.parent {
            let is_selected = self.focused && item_index == self.selected;
            lines.push(self.render_link_line(
                "\u{2191} Parent",
                &parent.key,
                &parent.fields.summary,
                &parent.fields.status,
                is_selected,
            ));
            item_index += 1;
        }

        // Links
        if !self.links.is_empty() {
            lines.push(Line::from(Span::styled(
                "Linked Issues:",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Gray),
            )));

            for link in &self.links {
                let is_selected = self.focused && item_index == self.selected;
                let prefix = match link.direction {
                    LinkDirection::Inward => "\u{2190}",
                    LinkDirection::Outward => "\u{2192}",
                };
                lines.push(self.render_link_line(
                    &format!("{} {}", prefix, link.link_description),
                    &link.key,
                    &link.summary,
                    &link.status,
                    is_selected,
                ));
                item_index += 1;
            }
        }

        // Subtasks
        if !self.subtasks.is_empty() {
            if !self.links.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                "Subtasks:",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Gray),
            )));

            for subtask in &self.subtasks {
                let is_selected = self.focused && item_index == self.selected;
                lines.push(self.render_subtask_line(subtask, is_selected));
                item_index += 1;
            }
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    /// Render a link line.
    fn render_link_line(
        &self,
        prefix: &str,
        key: &str,
        summary: &str,
        status: &Status,
        selected: bool,
    ) -> Line<'static> {
        let style = if selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let status_style = status_category_style(status);
        let truncated_summary = truncate(summary, 40);

        Line::from(vec![
            Span::styled(format!("  {} ", prefix), Style::default().fg(Color::Gray)),
            Span::styled(key.to_string(), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::raw(truncated_summary),
            Span::raw(" ["),
            Span::styled(status.name.clone(), status_style),
            Span::raw("]"),
        ])
        .style(style)
    }

    /// Render a subtask line.
    fn render_subtask_line(&self, subtask: &Subtask, selected: bool) -> Line<'static> {
        let style = if selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let checkbox = if subtask.fields.status.is_done() {
            "[x]"
        } else {
            "[ ]"
        };

        let status_style = status_category_style(&subtask.fields.status);
        let truncated_summary = truncate(&subtask.fields.summary, 40);

        Line::from(vec![
            Span::styled(format!("  {} ", checkbox), style),
            Span::styled(subtask.key.clone(), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::raw(truncated_summary),
            Span::raw(" ["),
            Span::styled(subtask.fields.status.name.clone(), status_style),
            Span::raw("]"),
        ])
        .style(style)
    }

    /// Calculate the height needed to render this section.
    pub fn height(&self) -> u16 {
        if self.is_empty() {
            return 0;
        }

        if !self.expanded {
            return 3; // Just the block with title
        }

        // 2 for borders + items + header lines
        let mut height = 2u16;

        if self.parent.is_some() {
            height += 1;
        }

        if !self.links.is_empty() {
            height += 1; // "Linked Issues:" header
            height += self.links.len() as u16;
        }

        if !self.subtasks.is_empty() {
            if !self.links.is_empty() {
                height += 1; // Empty line separator
            }
            height += 1; // "Subtasks:" header
            height += self.subtasks.len() as u16;
        }

        height.min(15) // Cap at reasonable height
    }
}

impl Default for LinkedIssuesSection {
    fn default() -> Self {
        Self::empty()
    }
}

/// Get the style for a status based on its category.
fn status_category_style(status: &Status) -> Style {
    if let Some(category) = &status.status_category {
        match category.key.as_str() {
            "done" => Style::default().fg(Color::Green),
            "indeterminate" => Style::default().fg(Color::Yellow),
            "new" => Style::default().fg(Color::Gray),
            _ => Style::default().fg(Color::White),
        }
    } else {
        Style::default().fg(Color::White)
    }
}

/// Truncate a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{
        IssueLinkType, LinkedIssue, LinkedIssueFields, ParentIssueFields, StatusCategory,
        SubtaskFields,
    };

    fn create_test_status(name: &str, done: bool) -> Status {
        Status {
            id: "1".to_string(),
            name: name.to_string(),
            status_category: Some(StatusCategory {
                id: if done { 3 } else { 2 },
                key: if done { "done" } else { "indeterminate" }.to_string(),
                name: if done { "Done" } else { "In Progress" }.to_string(),
                color_name: None,
            }),
        }
    }

    fn create_test_link(id: &str, key: &str, summary: &str, link_type: &str) -> IssueLink {
        IssueLink {
            id: format!("link-{}", id),
            link_type: IssueLinkType {
                id: "1".to_string(),
                name: link_type.to_string(),
                inward: format!("is {} by", link_type.to_lowercase()),
                outward: link_type.to_lowercase(),
            },
            inward_issue: None,
            outward_issue: Some(LinkedIssue {
                id: id.to_string(),
                key: key.to_string(),
                fields: LinkedIssueFields {
                    summary: summary.to_string(),
                    status: create_test_status("Open", false),
                    priority: None,
                    issue_type: None,
                },
            }),
        }
    }

    fn create_test_subtask(id: &str, key: &str, summary: &str, done: bool) -> Subtask {
        Subtask {
            id: id.to_string(),
            key: key.to_string(),
            fields: SubtaskFields {
                summary: summary.to_string(),
                status: create_test_status(if done { "Done" } else { "To Do" }, done),
                issue_type: None,
            },
        }
    }

    fn create_test_parent(key: &str, summary: &str) -> ParentIssue {
        ParentIssue {
            id: "1".to_string(),
            key: key.to_string(),
            fields: ParentIssueFields {
                summary: summary.to_string(),
                status: create_test_status("In Progress", false),
                issue_type: None,
            },
        }
    }

    #[test]
    fn test_empty_section() {
        let section = LinkedIssuesSection::empty();
        assert!(section.is_empty());
        assert_eq!(section.total_items(), 0);
        assert!(section.selected_key().is_none());
    }

    #[test]
    fn test_section_with_parent() {
        let parent = create_test_parent("PROJ-100", "Parent issue");
        let section = LinkedIssuesSection::new(&[], &[], Some(parent));

        assert!(!section.is_empty());
        assert_eq!(section.total_items(), 1);
        assert_eq!(section.selected_key(), Some("PROJ-100".to_string()));
    }

    #[test]
    fn test_section_with_links() {
        let links = vec![
            create_test_link("1", "PROJ-200", "Blocked issue", "Blocks"),
            create_test_link("2", "PROJ-201", "Related issue", "Relates"),
        ];
        let section = LinkedIssuesSection::new(&links, &[], None);

        assert!(!section.is_empty());
        assert_eq!(section.total_items(), 2);
        assert_eq!(section.links.len(), 2);
    }

    #[test]
    fn test_section_with_subtasks() {
        let subtasks = vec![
            create_test_subtask("1", "PROJ-101", "Subtask 1", false),
            create_test_subtask("2", "PROJ-102", "Subtask 2", true),
        ];
        let section = LinkedIssuesSection::new(&[], &subtasks, None);

        assert!(!section.is_empty());
        assert_eq!(section.total_items(), 2);
        assert_eq!(section.subtasks.len(), 2);
    }

    #[test]
    fn test_section_with_all() {
        let parent = create_test_parent("PROJ-100", "Parent");
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let subtasks = vec![create_test_subtask("1", "PROJ-101", "Subtask", false)];

        let section = LinkedIssuesSection::new(&links, &subtasks, Some(parent));

        assert_eq!(section.total_items(), 3);
    }

    #[test]
    fn test_navigation() {
        let parent = create_test_parent("PROJ-100", "Parent");
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let subtasks = vec![create_test_subtask("1", "PROJ-101", "Subtask", false)];

        let mut section = LinkedIssuesSection::new(&links, &subtasks, Some(parent));
        section.set_focused(true);

        // Initially at parent
        assert_eq!(section.selected(), 0);
        assert_eq!(section.selected_key(), Some("PROJ-100".to_string()));

        // Move to link
        section.select_next();
        assert_eq!(section.selected(), 1);
        assert_eq!(section.selected_key(), Some("PROJ-200".to_string()));

        // Move to subtask
        section.select_next();
        assert_eq!(section.selected(), 2);
        assert_eq!(section.selected_key(), Some("PROJ-101".to_string()));

        // Can't go past end
        section.select_next();
        assert_eq!(section.selected(), 2);

        // Move back up
        section.select_previous();
        assert_eq!(section.selected(), 1);
    }

    #[test]
    fn test_expand_collapse() {
        let mut section = LinkedIssuesSection::new(&[], &[], None);

        assert!(section.is_expanded());
        section.toggle_expand();
        assert!(!section.is_expanded());
        section.toggle_expand();
        assert!(section.is_expanded());
    }

    #[test]
    fn test_handle_input_navigation() {
        let links = vec![
            create_test_link("1", "PROJ-200", "Link 1", "Blocks"),
            create_test_link("2", "PROJ-201", "Link 2", "Relates"),
        ];
        let mut section = LinkedIssuesSection::new(&links, &[], None);
        section.set_focused(true);

        // Navigate down with j
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = section.handle_input(key);
        assert!(action.is_none());
        assert_eq!(section.selected(), 1);

        // Navigate up with k
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let action = section.handle_input(key);
        assert!(action.is_none());
        assert_eq!(section.selected(), 0);
    }

    #[test]
    fn test_handle_input_enter() {
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let mut section = LinkedIssuesSection::new(&links, &[], None);
        section.set_focused(true);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = section.handle_input(key);

        assert_eq!(
            action,
            Some(LinkedIssuesAction::Navigate("PROJ-200".to_string()))
        );
    }

    #[test]
    fn test_handle_input_tab_toggles_expand() {
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let mut section = LinkedIssuesSection::new(&links, &[], None);
        section.set_focused(true);

        assert!(section.is_expanded());

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let action = section.handle_input(key);
        assert!(action.is_none());
        assert!(!section.is_expanded());
    }

    #[test]
    fn test_input_ignored_when_not_focused() {
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let mut section = LinkedIssuesSection::new(&links, &[], None);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = section.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_input_ignored_when_collapsed() {
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let mut section = LinkedIssuesSection::new(&links, &[], None);
        section.set_focused(true);
        section.toggle_expand(); // Collapse

        // Navigation should be ignored
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = section.handle_input(key);
        assert!(action.is_none());

        // Enter should be ignored
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = section.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is...");
        assert_eq!(truncate("exactly10!", 10), "exactly10!");
    }

    #[test]
    fn test_status_is_done() {
        let done_status = create_test_status("Done", true);
        let in_progress_status = create_test_status("In Progress", false);

        assert!(done_status.is_done());
        assert!(!in_progress_status.is_done());
    }

    #[test]
    fn test_height_calculation() {
        // Empty section
        let empty = LinkedIssuesSection::empty();
        assert_eq!(empty.height(), 0);

        // Collapsed section
        let mut section = LinkedIssuesSection::new(
            &[create_test_link("1", "PROJ-1", "Link", "Blocks")],
            &[],
            None,
        );
        section.toggle_expand();
        assert_eq!(section.height(), 3);

        // With parent only
        let section =
            LinkedIssuesSection::new(&[], &[], Some(create_test_parent("PROJ-1", "Parent")));
        assert!(section.height() >= 3);
    }

    #[test]
    fn test_default_impl() {
        let section = LinkedIssuesSection::default();
        assert!(section.is_empty());
    }

    #[test]
    fn test_selected_link_on_link() {
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let section = LinkedIssuesSection::new(&links, &[], None);

        // First item is the link
        let result = section.selected_link();
        assert!(result.is_some());
        let (link_id, description) = result.unwrap();
        assert_eq!(link_id, "link-1");
        assert!(description.contains("PROJ-200"));
        assert!(description.contains("blocks"));
    }

    #[test]
    fn test_selected_link_on_parent() {
        let parent = create_test_parent("PROJ-100", "Parent");
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let section = LinkedIssuesSection::new(&links, &[], Some(parent));

        // First item is parent, not a link
        let result = section.selected_link();
        assert!(result.is_none());
    }

    #[test]
    fn test_selected_link_on_subtask() {
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let subtasks = vec![create_test_subtask("1", "PROJ-101", "Subtask", false)];
        let mut section = LinkedIssuesSection::new(&links, &subtasks, None);

        // Move to subtask (past the link)
        section.select_next();

        // Subtask is not a deletable link
        let result = section.selected_link();
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_key_on_link() {
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let mut section = LinkedIssuesSection::new(&links, &[], None);
        section.set_focused(true);

        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        let action = section.handle_input(key);

        match action {
            Some(LinkedIssuesAction::Delete(link_id, _)) => {
                assert_eq!(link_id, "link-1");
            }
            _ => panic!("Expected Delete action"),
        }
    }

    #[test]
    fn test_delete_key_on_parent_does_nothing() {
        let parent = create_test_parent("PROJ-100", "Parent");
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let mut section = LinkedIssuesSection::new(&links, &[], Some(parent));
        section.set_focused(true);

        // Focused on parent
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        let action = section.handle_input(key);

        assert!(action.is_none());
    }

    #[test]
    fn test_delete_key_on_subtask_does_nothing() {
        let subtasks = vec![create_test_subtask("1", "PROJ-101", "Subtask", false)];
        let mut section = LinkedIssuesSection::new(&[], &subtasks, None);
        section.set_focused(true);

        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        let action = section.handle_input(key);

        assert!(action.is_none());
    }

    #[test]
    fn test_delete_ignored_when_collapsed() {
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let mut section = LinkedIssuesSection::new(&links, &[], None);
        section.set_focused(true);
        section.toggle_expand(); // Collapse

        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        let action = section.handle_input(key);

        assert!(action.is_none());
    }
}
