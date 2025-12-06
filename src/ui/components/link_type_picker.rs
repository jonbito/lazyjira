//! Link manager component for viewing and managing issue links.
//!
//! Displays existing issue links and allows the user to:
//! - Navigate and view existing links
//! - Delete existing links
//! - Create new links (via 'c' or 'n' key)

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::api::types::{IssueLink, IssueLinkType, ParentIssue, Status, Subtask};

/// Action resulting from link manager input.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkManagerAction {
    /// Navigate to an issue.
    Navigate(String),
    /// Delete a link (link_id, description for confirmation).
    Delete(String, String),
    /// Start creating a new link - show link type selection.
    CreateNew,
    /// Select a link type for creation (type, is_outward).
    SelectLinkType(IssueLinkType, bool),
    /// Cancel and close the manager.
    Cancel,
}

/// Mode of the link manager.
#[derive(Debug, Clone, Copy, PartialEq)]
enum LinkManagerMode {
    /// Viewing existing links.
    ViewLinks,
    /// Selecting a link type for new link creation.
    SelectLinkType,
}

/// Direction of a link relationship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinkDirection {
    /// Other issue affects this one (inward link).
    Inward,
    /// This issue affects other (outward link).
    Outward,
}

/// A display-ready representation of an item in the link manager.
#[derive(Debug, Clone)]
enum LinkItem {
    /// Parent issue (not deletable).
    Parent {
        key: String,
        summary: String,
        status: Status,
    },
    /// A linked issue.
    Link {
        link_id: String,
        key: String,
        summary: String,
        status: Status,
        link_description: String,
        direction: LinkDirection,
    },
    /// A subtask (not deletable as a link).
    Subtask {
        key: String,
        summary: String,
        status: Status,
        is_done: bool,
    },
    /// Section header (not selectable).
    Header(String),
}

impl LinkItem {
    fn is_selectable(&self) -> bool {
        !matches!(self, LinkItem::Header(_))
    }

    fn key(&self) -> Option<&str> {
        match self {
            LinkItem::Parent { key, .. } => Some(key),
            LinkItem::Link { key, .. } => Some(key),
            LinkItem::Subtask { key, .. } => Some(key),
            LinkItem::Header(_) => None,
        }
    }

    fn link_info(&self) -> Option<(String, String)> {
        match self {
            LinkItem::Link {
                link_id,
                key,
                link_description,
                ..
            } => Some((link_id.clone(), format!("{} ({})", key, link_description))),
            _ => None,
        }
    }
}

/// Display item for link type selection.
#[derive(Debug, Clone)]
struct LinkTypeItem {
    /// The underlying link type.
    link_type: IssueLinkType,
    /// The display text for this direction.
    display: String,
    /// True if selecting this makes the current issue the outward issue.
    is_outward: bool,
}

/// Link manager component.
///
/// Shows existing links and allows viewing, deleting, and creating new links.
#[derive(Debug)]
pub struct LinkManager {
    /// Current mode (viewing links or selecting link type).
    mode: LinkManagerMode,
    /// Items to display in view mode.
    items: Vec<LinkItem>,
    /// Link type items for selection mode.
    link_type_items: Vec<LinkTypeItem>,
    /// Currently selected index in view mode.
    selected: usize,
    /// Currently selected index in link type selection mode.
    link_type_selected: usize,
    /// Whether the manager is visible.
    visible: bool,
    /// Whether link types are loading.
    loading: bool,
}

impl LinkManager {
    /// Create a new link manager.
    pub fn new() -> Self {
        Self {
            mode: LinkManagerMode::ViewLinks,
            items: Vec::new(),
            link_type_items: Vec::new(),
            selected: 0,
            link_type_selected: 0,
            visible: false,
            loading: false,
        }
    }

    /// Check if the manager is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if link types are loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Show the manager with loading state (for link type fetching).
    pub fn show_loading(&mut self) {
        self.link_type_items.clear();
        self.link_type_selected = 0;
        self.loading = true;
        self.mode = LinkManagerMode::SelectLinkType;
        self.visible = true;
    }

    /// Show the manager with the given issue data.
    pub fn show(
        &mut self,
        links: &[IssueLink],
        subtasks: &[Subtask],
        parent: Option<&ParentIssue>,
    ) {
        self.items.clear();
        self.selected = 0;
        self.mode = LinkManagerMode::ViewLinks;
        self.loading = false;

        // Add parent if exists
        if let Some(p) = parent {
            self.items.push(LinkItem::Parent {
                key: p.key.clone(),
                summary: p.fields.summary.clone(),
                status: p.fields.status.clone(),
            });
        }

        // Add linked issues
        let display_links: Vec<_> = links
            .iter()
            .filter_map(|link| {
                if let Some(inward) = &link.inward_issue {
                    Some(LinkItem::Link {
                        link_id: link.id.clone(),
                        key: inward.key.clone(),
                        summary: inward.fields.summary.clone(),
                        status: inward.fields.status.clone(),
                        link_description: link.link_type.inward.clone(),
                        direction: LinkDirection::Inward,
                    })
                } else {
                    link.outward_issue.as_ref().map(|outward| LinkItem::Link {
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

        if !display_links.is_empty() {
            self.items
                .push(LinkItem::Header("Linked Issues".to_string()));
            self.items.extend(display_links);
        }

        // Add subtasks
        if !subtasks.is_empty() {
            self.items.push(LinkItem::Header("Subtasks".to_string()));
            for st in subtasks {
                self.items.push(LinkItem::Subtask {
                    key: st.key.clone(),
                    summary: st.fields.summary.clone(),
                    status: st.fields.status.clone(),
                    is_done: st.fields.status.is_done(),
                });
            }
        }

        // Skip to first selectable item
        self.selected = self
            .items
            .iter()
            .position(|item| item.is_selectable())
            .unwrap_or(0);

        self.visible = true;
    }

    /// Set link types for selection mode.
    pub fn set_link_types(&mut self, link_types: Vec<IssueLinkType>) {
        self.link_type_items.clear();

        for lt in link_types {
            // Outward direction: "this issue [outward] target"
            self.link_type_items.push(LinkTypeItem {
                display: format!("This issue {} ...", lt.outward),
                link_type: lt.clone(),
                is_outward: true,
            });

            // Inward direction: "this issue [inward] target"
            self.link_type_items.push(LinkTypeItem {
                display: format!("This issue {} ...", lt.inward),
                link_type: lt,
                is_outward: false,
            });
        }

        self.link_type_selected = 0;
        self.loading = false;
    }

    /// Switch to link type selection mode.
    pub fn start_link_type_selection(&mut self) {
        self.mode = LinkManagerMode::SelectLinkType;
        self.link_type_selected = 0;
    }

    /// Hide the manager.
    pub fn hide(&mut self) {
        self.visible = false;
        self.loading = false;
        self.mode = LinkManagerMode::ViewLinks;
    }

    /// Get the total number of selectable items.
    pub fn selectable_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| item.is_selectable())
            .count()
    }

    /// Check if manager is empty (no links/subtasks/parent).
    pub fn is_empty(&self) -> bool {
        self.selectable_count() == 0
    }

    /// Handle keyboard input.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<LinkManagerAction> {
        if !self.visible {
            return None;
        }

        // Allow Esc while loading
        if self.loading {
            if key.code == KeyCode::Esc {
                self.hide();
                return Some(LinkManagerAction::Cancel);
            }
            return None;
        }

        match self.mode {
            LinkManagerMode::ViewLinks => self.handle_view_mode_input(key),
            LinkManagerMode::SelectLinkType => self.handle_link_type_selection_input(key),
        }
    }

    /// Handle input in view mode.
    fn handle_view_mode_input(&mut self, key: KeyEvent) -> Option<LinkManagerAction> {
        match (key.code, key.modifiers) {
            // Navigation down with j or arrow
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                self.select_next();
                None
            }
            // Navigation up with k or arrow
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                self.select_previous();
                None
            }
            // Navigate to issue
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(item) = self.items.get(self.selected) {
                    if let Some(key) = item.key() {
                        let result = LinkManagerAction::Navigate(key.to_string());
                        self.hide();
                        return Some(result);
                    }
                }
                None
            }
            // Create new link
            (KeyCode::Char('c'), KeyModifiers::NONE)
            | (KeyCode::Char('n'), KeyModifiers::NONE) => Some(LinkManagerAction::CreateNew),
            // Delete selected link
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                if let Some(item) = self.items.get(self.selected) {
                    if let Some((link_id, description)) = item.link_info() {
                        self.hide();
                        return Some(LinkManagerAction::Delete(link_id, description));
                    }
                }
                None
            }
            // Cancel with q or Esc
            (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.hide();
                Some(LinkManagerAction::Cancel)
            }
            _ => None,
        }
    }

    /// Handle input in link type selection mode.
    fn handle_link_type_selection_input(&mut self, key: KeyEvent) -> Option<LinkManagerAction> {
        match (key.code, key.modifiers) {
            // Navigation down
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                if !self.link_type_items.is_empty()
                    && self.link_type_selected < self.link_type_items.len() - 1
                {
                    self.link_type_selected += 1;
                }
                None
            }
            // Navigation up
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                if self.link_type_selected > 0 {
                    self.link_type_selected -= 1;
                }
                None
            }
            // Select link type
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(item) = self.link_type_items.get(self.link_type_selected) {
                    let result =
                        LinkManagerAction::SelectLinkType(item.link_type.clone(), item.is_outward);
                    self.hide();
                    return Some(result);
                }
                None
            }
            // Cancel - go back to view mode or close
            (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                if !self.items.is_empty() {
                    // Go back to view mode
                    self.mode = LinkManagerMode::ViewLinks;
                    None
                } else {
                    // Close if no items to view
                    self.hide();
                    Some(LinkManagerAction::Cancel)
                }
            }
            _ => None,
        }
    }

    /// Move selection to next selectable item.
    fn select_next(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }

        let mut next = self.selected + 1;
        while next < len {
            if self.items[next].is_selectable() {
                self.selected = next;
                return;
            }
            next += 1;
        }
    }

    /// Move selection to previous selectable item.
    fn select_previous(&mut self) {
        if self.selected == 0 {
            return;
        }

        let mut prev = self.selected - 1;
        loop {
            if self.items[prev].is_selectable() {
                self.selected = prev;
                return;
            }
            if prev == 0 {
                break;
            }
            prev -= 1;
        }
    }

    /// Render the link manager.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        match self.mode {
            LinkManagerMode::ViewLinks => self.render_view_mode(frame, area),
            LinkManagerMode::SelectLinkType => self.render_link_type_selection(frame, area),
        }
    }

    /// Render view mode (showing existing links).
    fn render_view_mode(&self, frame: &mut Frame, area: Rect) {
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 20.min(area.height.saturating_sub(4));
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        frame.render_widget(Clear, dialog_area);

        let title = if self.is_empty() {
            " Issue Links (empty) "
        } else {
            " Issue Links "
        };

        let block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Links list
                Constraint::Length(2), // Help text
            ])
            .split(inner);

        if self.is_empty() {
            let empty_text = Paragraph::new("No linked issues, subtasks, or parent.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_text, chunks[0]);
        } else {
            // Build list items
            let list_items: Vec<ListItem> = self
                .items
                .iter()
                .enumerate()
                .map(|(idx, item)| self.render_item(item, idx == self.selected))
                .collect();

            let list = List::new(list_items);

            frame.render_widget(list, chunks[0]);
        }

        // Help text
        let help_text = Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(": nav  "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": go to  "),
            Span::styled("c/n", Style::default().fg(Color::Cyan)),
            Span::raw(": new  "),
            Span::styled("d", Style::default().fg(Color::Red)),
            Span::raw(": delete  "),
            Span::styled("q", Style::default().fg(Color::Gray)),
            Span::raw(": close"),
        ]);
        let help_paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
        frame.render_widget(help_paragraph, chunks[1]);
    }

    /// Render a single item in the list.
    fn render_item(&self, item: &LinkItem, selected: bool) -> ListItem<'static> {
        // When selected, use brighter colors for better readability against dark background
        let (line_style, label_color, key_color, text_color) = if selected {
            (
                Style::default().bg(Color::DarkGray),
                Color::White,
                Color::Cyan,
                Color::White,
            )
        } else {
            (Style::default(), Color::Gray, Color::Cyan, Color::Reset)
        };

        match item {
            LinkItem::Header(text) => ListItem::new(Line::from(Span::styled(
                text.clone(),
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            ))),
            LinkItem::Parent {
                key,
                summary,
                status,
            } => {
                let status_style = if selected {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    status_category_style(status)
                };
                let truncated = truncate(summary, 35);

                ListItem::new(
                    Line::from(vec![
                        Span::styled("  ↑ Parent ", Style::default().fg(label_color)),
                        Span::styled(key.clone(), Style::default().fg(key_color)),
                        Span::styled(" ", Style::default().fg(text_color)),
                        Span::styled(truncated, Style::default().fg(text_color)),
                        Span::styled(" [", Style::default().fg(text_color)),
                        Span::styled(status.name.clone(), status_style),
                        Span::styled("]", Style::default().fg(text_color)),
                    ])
                    .style(line_style),
                )
            }
            LinkItem::Link {
                key,
                summary,
                status,
                link_description,
                direction,
                ..
            } => {
                let status_style = if selected {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    status_category_style(status)
                };
                let truncated = truncate(summary, 30);
                let arrow = match direction {
                    LinkDirection::Inward => "←",
                    LinkDirection::Outward => "→",
                };

                ListItem::new(
                    Line::from(vec![
                        Span::styled(
                            format!("  {} {} ", arrow, link_description),
                            Style::default().fg(label_color),
                        ),
                        Span::styled(key.clone(), Style::default().fg(key_color)),
                        Span::styled(" ", Style::default().fg(text_color)),
                        Span::styled(truncated, Style::default().fg(text_color)),
                        Span::styled(" [", Style::default().fg(text_color)),
                        Span::styled(status.name.clone(), status_style),
                        Span::styled("]", Style::default().fg(text_color)),
                    ])
                    .style(line_style),
                )
            }
            LinkItem::Subtask {
                key,
                summary,
                status,
                is_done,
            } => {
                let status_style = if selected {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    status_category_style(status)
                };
                let truncated = truncate(summary, 35);
                let checkbox = if *is_done { "[x]" } else { "[ ]" };

                ListItem::new(
                    Line::from(vec![
                        Span::styled(format!("  {} ", checkbox), Style::default().fg(label_color)),
                        Span::styled(key.clone(), Style::default().fg(key_color)),
                        Span::styled(" ", Style::default().fg(text_color)),
                        Span::styled(truncated, Style::default().fg(text_color)),
                        Span::styled(" [", Style::default().fg(text_color)),
                        Span::styled(status.name.clone(), status_style),
                        Span::styled("]", Style::default().fg(text_color)),
                    ])
                    .style(line_style),
                )
            }
        }
    }

    /// Render link type selection mode.
    fn render_link_type_selection(&self, frame: &mut Frame, area: Rect) {
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 18.min(area.height.saturating_sub(4));
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(" Select Link Type ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Description
                Constraint::Min(3),    // Link types list
                Constraint::Length(2), // Help text
            ])
            .split(inner);

        // Description
        let desc_text = Line::from(vec![Span::styled(
            "Choose how this issue relates to the target:",
            Style::default().fg(Color::DarkGray),
        )]);
        let desc_paragraph = Paragraph::new(desc_text);
        frame.render_widget(desc_paragraph, chunks[0]);

        // Link types list or loading
        if self.loading {
            let loading_text = Paragraph::new("Loading link types...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading_text, chunks[1]);
        } else if self.link_type_items.is_empty() {
            let empty_text = Paragraph::new("No link types available")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_text, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .link_type_items
                .iter()
                .map(|item| {
                    ListItem::new(item.display.clone()).style(Style::default().fg(Color::White))
                })
                .collect();

            let list = List::new(items)
                .highlight_style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            let mut state = ListState::default();
            state.select(Some(self.link_type_selected));

            frame.render_stateful_widget(list, chunks[1], &mut state);
        }

        // Help text
        let help_text = Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(": navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": select  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": back"),
        ]);
        let help_paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
        frame.render_widget(help_paragraph, chunks[2]);
    }
}

impl Default for LinkManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a centered rectangle.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
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

    fn create_test_link_type(id: &str, name: &str, inward: &str, outward: &str) -> IssueLinkType {
        IssueLinkType {
            id: id.to_string(),
            name: name.to_string(),
            inward: inward.to_string(),
            outward: outward.to_string(),
        }
    }

    fn create_test_link(id: &str, key: &str, summary: &str, link_type_name: &str) -> IssueLink {
        IssueLink {
            id: format!("link-{}", id),
            link_type: IssueLinkType {
                id: "1".to_string(),
                name: link_type_name.to_string(),
                inward: format!("is {} by", link_type_name.to_lowercase()),
                outward: link_type_name.to_lowercase(),
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
    fn test_new_manager() {
        let manager = LinkManager::new();
        assert!(!manager.is_visible());
        assert!(!manager.is_loading());
        assert!(manager.is_empty());
    }

    #[test]
    fn test_show_with_links() {
        let mut manager = LinkManager::new();
        let links = vec![
            create_test_link("1", "PROJ-200", "Blocked issue", "Blocks"),
            create_test_link("2", "PROJ-201", "Related issue", "Relates"),
        ];

        manager.show(&links, &[], None);

        assert!(manager.is_visible());
        assert!(!manager.is_loading());
        // Header + 2 links
        assert_eq!(manager.selectable_count(), 2);
    }

    #[test]
    fn test_show_with_parent_links_subtasks() {
        let mut manager = LinkManager::new();
        let parent = create_test_parent("PROJ-100", "Parent issue");
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let subtasks = vec![create_test_subtask("1", "PROJ-101", "Subtask", false)];

        manager.show(&links, &subtasks, Some(&parent));

        assert!(manager.is_visible());
        // Parent + 1 link + 1 subtask (headers are not selectable)
        assert_eq!(manager.selectable_count(), 3);
    }

    #[test]
    fn test_navigation_skips_headers() {
        let mut manager = LinkManager::new();
        let parent = create_test_parent("PROJ-100", "Parent");
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];
        let subtasks = vec![create_test_subtask("1", "PROJ-101", "Subtask", false)];

        manager.show(&links, &subtasks, Some(&parent));

        // Should start at parent (first selectable)
        assert_eq!(manager.selected, 0);

        // Move down - should skip header and go to link
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        manager.handle_input(key);
        // selected should now be at link (index 2, skipping header at 1)
        assert_eq!(manager.selected, 2);

        // Move down again - should skip header and go to subtask
        manager.handle_input(key);
        assert_eq!(manager.selected, 4);
    }

    #[test]
    fn test_navigate_to_issue() {
        let mut manager = LinkManager::new();
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];

        manager.show(&links, &[], None);

        // Skip to the link (first item is header)
        manager.selected = 1;

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = manager.handle_input(key);

        match action {
            Some(LinkManagerAction::Navigate(key)) => {
                assert_eq!(key, "PROJ-200");
            }
            _ => panic!("Expected Navigate action"),
        }
        assert!(!manager.is_visible());
    }

    #[test]
    fn test_create_new_link_with_c() {
        let mut manager = LinkManager::new();
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];

        manager.show(&links, &[], None);

        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        let action = manager.handle_input(key);

        assert_eq!(action, Some(LinkManagerAction::CreateNew));
    }

    #[test]
    fn test_create_new_link_with_n() {
        let mut manager = LinkManager::new();
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];

        manager.show(&links, &[], None);

        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        let action = manager.handle_input(key);

        assert_eq!(action, Some(LinkManagerAction::CreateNew));
    }

    #[test]
    fn test_delete_link() {
        let mut manager = LinkManager::new();
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];

        manager.show(&links, &[], None);

        // Move to link (first selectable after header)
        manager.selected = 1;

        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        let action = manager.handle_input(key);

        match action {
            Some(LinkManagerAction::Delete(link_id, description)) => {
                assert_eq!(link_id, "link-1");
                assert!(description.contains("PROJ-200"));
            }
            _ => panic!("Expected Delete action"),
        }
        // Manager should hide when delete is triggered
        assert!(!manager.is_visible());
    }

    #[test]
    fn test_delete_parent_does_nothing() {
        let mut manager = LinkManager::new();
        let parent = create_test_parent("PROJ-100", "Parent");

        manager.show(&[], &[], Some(&parent));

        // Selected on parent
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        let action = manager.handle_input(key);

        assert!(action.is_none());
    }

    #[test]
    fn test_delete_subtask_does_nothing() {
        let mut manager = LinkManager::new();
        let subtasks = vec![create_test_subtask("1", "PROJ-101", "Subtask", false)];

        manager.show(&[], &subtasks, None);

        // Move to subtask
        manager.selected = 1;

        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        let action = manager.handle_input(key);

        assert!(action.is_none());
    }

    #[test]
    fn test_link_type_selection() {
        let mut manager = LinkManager::new();
        let link_types = vec![create_test_link_type(
            "1",
            "Blocks",
            "is blocked by",
            "blocks",
        )];

        manager.show_loading();
        manager.set_link_types(link_types);

        assert!(manager.is_visible());
        assert!(!manager.is_loading());
        assert_eq!(manager.mode, LinkManagerMode::SelectLinkType);

        // Select first link type (outward)
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = manager.handle_input(key);

        match action {
            Some(LinkManagerAction::SelectLinkType(lt, is_outward)) => {
                assert_eq!(lt.id, "1");
                assert!(is_outward);
            }
            _ => panic!("Expected SelectLinkType action"),
        }
    }

    #[test]
    fn test_cancel_with_esc() {
        let mut manager = LinkManager::new();
        manager.show(&[], &[], None);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = manager.handle_input(key);

        assert_eq!(action, Some(LinkManagerAction::Cancel));
        assert!(!manager.is_visible());
    }

    #[test]
    fn test_cancel_with_q() {
        let mut manager = LinkManager::new();
        manager.show(&[], &[], None);

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = manager.handle_input(key);

        assert_eq!(action, Some(LinkManagerAction::Cancel));
        assert!(!manager.is_visible());
    }

    #[test]
    fn test_esc_while_loading() {
        let mut manager = LinkManager::new();
        manager.show_loading();

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = manager.handle_input(key);

        assert_eq!(action, Some(LinkManagerAction::Cancel));
        assert!(!manager.is_visible());
    }

    #[test]
    fn test_input_ignored_when_not_visible() {
        let mut manager = LinkManager::new();

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = manager.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_default_impl() {
        let manager = LinkManager::default();
        assert!(!manager.is_visible());
    }

    #[test]
    fn test_hide() {
        let mut manager = LinkManager::new();
        manager.show(&[], &[], None);
        assert!(manager.is_visible());

        manager.hide();
        assert!(!manager.is_visible());
    }

    #[test]
    fn test_navigation_vim_keys() {
        let mut manager = LinkManager::new();
        let links = vec![
            create_test_link("1", "PROJ-200", "Link 1", "Blocks"),
            create_test_link("2", "PROJ-201", "Link 2", "Relates"),
        ];
        manager.show(&links, &[], None);

        // Start after header
        manager.selected = 1;

        // Move down with j
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        manager.handle_input(key);
        assert_eq!(manager.selected, 2);

        // Move up with k
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        manager.handle_input(key);
        assert_eq!(manager.selected, 1);
    }

    #[test]
    fn test_esc_in_link_type_selection_returns_to_view() {
        let mut manager = LinkManager::new();
        let links = vec![create_test_link("1", "PROJ-200", "Linked", "Blocks")];

        manager.show(&links, &[], None);
        manager.set_link_types(vec![create_test_link_type(
            "1",
            "Blocks",
            "is blocked by",
            "blocks",
        )]);
        manager.start_link_type_selection();

        // Esc should return to view mode
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = manager.handle_input(key);

        assert!(action.is_none());
        assert_eq!(manager.mode, LinkManagerMode::ViewLinks);
        assert!(manager.is_visible());
    }

    #[test]
    fn test_esc_in_link_type_selection_closes_if_empty() {
        let mut manager = LinkManager::new();
        manager.show_loading();
        manager.set_link_types(vec![create_test_link_type(
            "1",
            "Blocks",
            "is blocked by",
            "blocks",
        )]);

        // No items in view mode, Esc should close
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = manager.handle_input(key);

        assert_eq!(action, Some(LinkManagerAction::Cancel));
        assert!(!manager.is_visible());
    }
}
