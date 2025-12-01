//! History view for displaying issue changelog.
//!
//! This view provides a scrollable panel that displays the change history
//! of a JIRA issue, showing field changes grouped by date and user.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::api::types::{ChangeHistory, ChangeItem, ChangeType, Changelog};
use crate::ui::theme::theme;

/// Actions that can be returned from the history view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryAction {
    /// Close the history panel.
    Close,
    /// Load more history entries.
    LoadMore(String),
}

/// Filter for which change types to display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistoryFilter {
    /// Show all changes.
    #[default]
    All,
    /// Show only status/workflow changes.
    Status,
    /// Show only assignee changes.
    Assignee,
    /// Show only content changes (summary, description).
    Content,
    /// Show only field changes (priority, labels, etc.).
    Fields,
}

impl HistoryFilter {
    /// Get the display name for this filter.
    pub fn display(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Status => "Status",
            Self::Assignee => "Assignee",
            Self::Content => "Content",
            Self::Fields => "Fields",
        }
    }

    /// Cycle to the next filter.
    pub fn next(&self) -> Self {
        match self {
            Self::All => Self::Status,
            Self::Status => Self::Assignee,
            Self::Assignee => Self::Content,
            Self::Content => Self::Fields,
            Self::Fields => Self::All,
        }
    }

    /// Cycle to the previous filter.
    pub fn prev(&self) -> Self {
        match self {
            Self::All => Self::Fields,
            Self::Status => Self::All,
            Self::Assignee => Self::Status,
            Self::Content => Self::Assignee,
            Self::Fields => Self::Content,
        }
    }

    /// Check if a change item matches this filter.
    pub fn matches(&self, item: &ChangeItem) -> bool {
        match self {
            Self::All => true,
            Self::Status => matches!(item.change_type(), ChangeType::Status | ChangeType::Resolution),
            Self::Assignee => matches!(item.change_type(), ChangeType::Assignee),
            Self::Content => matches!(item.change_type(), ChangeType::Content | ChangeType::Comment),
            Self::Fields => matches!(
                item.change_type(),
                ChangeType::Priority
                    | ChangeType::Tags
                    | ChangeType::Sprint
                    | ChangeType::Version
                    | ChangeType::Link
                    | ChangeType::Attachment
                    | ChangeType::Other
            ),
        }
    }
}

/// The history view panel.
pub struct HistoryView {
    /// Whether the panel is visible.
    visible: bool,
    /// Whether the history is currently loading.
    loading: bool,
    /// The issue key for the current history.
    issue_key: Option<String>,
    /// The list of history entries.
    histories: Vec<ChangeHistory>,
    /// Total number of history entries available.
    total: u32,
    /// Whether all entries have been loaded.
    all_loaded: bool,
    /// Current scroll position.
    scroll: usize,
    /// Visible height (updated on render).
    visible_height: usize,
    /// Total lines in content (calculated during render).
    total_lines: usize,
    /// Current filter.
    filter: HistoryFilter,
    /// Cached filtered entries (recalculated when filter changes).
    filtered_count: usize,
}

impl HistoryView {
    /// Create a new history view.
    pub fn new() -> Self {
        Self {
            visible: false,
            loading: false,
            issue_key: None,
            histories: Vec::new(),
            total: 0,
            all_loaded: false,
            scroll: 0,
            visible_height: 0,
            total_lines: 0,
            filter: HistoryFilter::All,
            filtered_count: 0,
        }
    }

    /// Check if the panel is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if the history is loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Show the history panel for an issue.
    pub fn show(&mut self, issue_key: &str) {
        self.visible = true;
        self.loading = true;
        self.issue_key = Some(issue_key.to_string());
        self.histories.clear();
        self.total = 0;
        self.all_loaded = false;
        self.scroll = 0;
        self.filter = HistoryFilter::All;
        self.filtered_count = 0;
    }

    /// Hide the history panel.
    pub fn hide(&mut self) {
        self.visible = false;
        self.loading = false;
        self.issue_key = None;
        self.histories.clear();
        self.total = 0;
        self.all_loaded = false;
        self.scroll = 0;
    }

    /// Set the loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Set the changelog data.
    pub fn set_changelog(&mut self, changelog: Changelog) {
        self.loading = false;
        self.total = changelog.total;
        self.all_loaded = !changelog.has_more();
        // Append new histories (in reverse order so newest is first)
        self.histories.extend(changelog.histories);
        self.update_filtered_count();
    }

    /// Append more changelog data.
    pub fn append_changelog(&mut self, changelog: Changelog) {
        self.loading = false;
        self.all_loaded = !changelog.has_more();
        self.histories.extend(changelog.histories);
        self.update_filtered_count();
    }

    /// Get the issue key.
    pub fn issue_key(&self) -> Option<&str> {
        self.issue_key.as_deref()
    }

    /// Get the current starting position for loading more.
    pub fn next_start(&self) -> u32 {
        self.histories.len() as u32
    }

    /// Check if more history can be loaded.
    pub fn can_load_more(&self) -> bool {
        !self.loading && !self.all_loaded
    }

    /// Get the current filter.
    pub fn filter(&self) -> HistoryFilter {
        self.filter
    }

    /// Update the filtered count.
    fn update_filtered_count(&mut self) {
        self.filtered_count = self
            .histories
            .iter()
            .flat_map(|h| h.items.iter())
            .filter(|item| self.filter.matches(item))
            .count();
    }

    /// Get the maximum scroll position.
    fn max_scroll(&self) -> usize {
        self.total_lines.saturating_sub(self.visible_height)
    }

    /// Handle keyboard input for the history view.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<HistoryAction> {
        if !self.visible {
            return None;
        }

        match (key.code, key.modifiers) {
            // Close panel
            (KeyCode::Esc, KeyModifiers::NONE)
            | (KeyCode::Char('q'), KeyModifiers::NONE)
            | (KeyCode::Char('h'), KeyModifiers::NONE) => {
                self.hide();
                Some(HistoryAction::Close)
            }

            // Scroll down
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
                self.scroll = (self.scroll + 1).min(self.max_scroll());
                // Trigger load more when near bottom
                if self.scroll >= self.max_scroll().saturating_sub(5) && self.can_load_more() {
                    if let Some(key) = &self.issue_key {
                        self.loading = true;
                        return Some(HistoryAction::LoadMore(key.clone()));
                    }
                }
                None
            }

            // Scroll up
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
                self.scroll = self.scroll.saturating_sub(1);
                None
            }

            // Page down
            (KeyCode::Char('d'), KeyModifiers::CONTROL)
            | (KeyCode::PageDown, KeyModifiers::NONE) => {
                let page_size = self.visible_height.saturating_sub(2);
                self.scroll = (self.scroll + page_size).min(self.max_scroll());
                // Trigger load more when near bottom
                if self.scroll >= self.max_scroll().saturating_sub(5) && self.can_load_more() {
                    if let Some(key) = &self.issue_key {
                        self.loading = true;
                        return Some(HistoryAction::LoadMore(key.clone()));
                    }
                }
                None
            }

            // Page up
            (KeyCode::Char('u'), KeyModifiers::CONTROL) | (KeyCode::PageUp, KeyModifiers::NONE) => {
                let page_size = self.visible_height.saturating_sub(2);
                self.scroll = self.scroll.saturating_sub(page_size);
                None
            }

            // Go to top
            (KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.scroll = 0;
                None
            }

            // Go to bottom
            (KeyCode::Char('G'), KeyModifiers::SHIFT) | (KeyCode::Char('G'), KeyModifiers::NONE) => {
                self.scroll = self.max_scroll();
                // Trigger load more when at bottom
                if self.can_load_more() {
                    if let Some(key) = &self.issue_key {
                        self.loading = true;
                        return Some(HistoryAction::LoadMore(key.clone()));
                    }
                }
                None
            }

            // Next filter
            (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::Char('f'), KeyModifiers::NONE) => {
                self.filter = self.filter.next();
                self.update_filtered_count();
                self.scroll = 0;
                None
            }

            // Previous filter
            (KeyCode::BackTab, KeyModifiers::SHIFT)
            | (KeyCode::Char('F'), KeyModifiers::SHIFT) => {
                self.filter = self.filter.prev();
                self.update_filtered_count();
                self.scroll = 0;
                None
            }

            _ => None,
        }
    }

    /// Render the history view.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let t = theme();

        // Create centered overlay (80% width, 90% height)
        let overlay_width = (area.width as f32 * 0.8) as u16;
        let overlay_height = (area.height as f32 * 0.9) as u16;
        let overlay_x = area.x + (area.width - overlay_width) / 2;
        let overlay_y = area.y + (area.height - overlay_height) / 2;
        let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

        // Clear the area
        frame.render_widget(Clear, overlay_area);

        // Create layout with header, content, and footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(5),    // Content
                Constraint::Length(3), // Footer (needs 3 for borders)
            ])
            .split(overlay_area);

        // Render header
        self.render_header(frame, chunks[0]);

        // Render content
        self.render_content(frame, chunks[1]);

        // Render footer
        self.render_footer(frame, chunks[2]);
    }

    /// Render the header section.
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let t = theme();

        let issue_key = self.issue_key.as_deref().unwrap_or("Unknown");
        let count_text = if self.all_loaded {
            format!("{} changes", self.total)
        } else {
            format!("{} of {} changes", self.histories.len(), self.total)
        };

        let title = format!(" History - {} ({}) ", issue_key, count_text);

        let filter_text = format!("Filter: {}", self.filter.display());
        let loading_text = if self.loading { " [Loading...]" } else { "" };

        let header = Paragraph::new(Line::from(vec![
            Span::styled(filter_text, Style::default().fg(t.accent)),
            Span::styled(loading_text, Style::default().fg(t.warning)),
        ]))
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .title(title)
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(t.accent)),
        );

        frame.render_widget(header, area);
    }

    /// Render the main content area.
    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        let t = theme();

        let block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(t.border));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Update visible height
        self.visible_height = inner.height as usize;

        // Build content lines
        let lines = self.build_content_lines();
        self.total_lines = lines.len();

        // Ensure scroll is within bounds
        if self.scroll > self.max_scroll() {
            self.scroll = self.max_scroll();
        }

        // Create paragraph with scroll
        let paragraph = Paragraph::new(lines)
            .scroll((self.scroll as u16, 0))
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, inner);

        // Render scrollbar if needed
        if self.total_lines > self.visible_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"));

            let mut scrollbar_state =
                ScrollbarState::new(self.max_scroll()).position(self.scroll);

            let scrollbar_area = Rect::new(
                area.x + area.width - 1,
                area.y,
                1,
                area.height,
            );

            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }
    }

    /// Build the content lines for the history panel.
    fn build_content_lines(&self) -> Vec<Line<'static>> {
        let t = theme();
        let mut lines: Vec<Line<'static>> = Vec::new();

        if self.loading && self.histories.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Loading history...",
                Style::default().fg(t.muted),
            )]));
            return lines;
        }

        if self.histories.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "No history available",
                Style::default().fg(t.muted),
            )]));
            return lines;
        }

        // Group entries by date
        let mut current_date = String::new();

        for history in &self.histories {
            // Filter items
            let filtered_items: Vec<&ChangeItem> = history
                .items
                .iter()
                .filter(|item| self.filter.matches(item))
                .collect();

            if filtered_items.is_empty() {
                continue;
            }

            // Extract date from timestamp (YYYY-MM-DD)
            let date = if history.created.len() >= 10 {
                &history.created[..10]
            } else {
                &history.created
            };

            // Add date header if changed
            if date != current_date {
                if !current_date.is_empty() {
                    lines.push(Line::from("")); // Separator
                }
                current_date = date.to_string();
                lines.push(Line::from(vec![Span::styled(
                    format!("─── {} ───", date),
                    Style::default()
                        .fg(t.warning)
                        .add_modifier(Modifier::BOLD),
                )]));
                lines.push(Line::from(""));
            }

            // Time and author
            let time = if history.created.len() >= 16 {
                &history.created[11..16]
            } else {
                ""
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} ", time),
                    Style::default().fg(t.dim),
                ),
                Span::styled(
                    history.author.display_name.clone(),
                    Style::default()
                        .fg(t.accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));

            // Each change item
            for item in filtered_items {
                let change_type = item.change_type();
                let icon = change_type.icon();

                // Style based on change type
                let type_style = match change_type {
                    ChangeType::Status => Style::default().fg(t.success),
                    ChangeType::Assignee => Style::default().fg(t.info),
                    ChangeType::Priority => Style::default().fg(t.warning),
                    ChangeType::Content => Style::default().fg(t.accent),
                    ChangeType::Resolution => Style::default().fg(t.success),
                    _ => Style::default().fg(t.muted),
                };

                // Format the change description
                let from = item.display_from();
                let to = item.display_to();

                // For description changes, truncate the values
                let (from_display, to_display) = if matches!(change_type, ChangeType::Content) {
                    (truncate_str(from, 30), truncate_str(to, 30))
                } else {
                    (from.to_string(), to.to_string())
                };

                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("{} ", icon), type_style),
                    Span::styled(
                        format!("{}: ", item.field),
                        Style::default().fg(t.dim),
                    ),
                    Span::styled(from_display, Style::default().fg(t.error)),
                    Span::styled(" → ", Style::default().fg(t.dim)),
                    Span::styled(to_display, Style::default().fg(t.success)),
                ]));
            }
        }

        // Add "Load more" indicator if available
        if !self.all_loaded {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                if self.loading {
                    "Loading more..."
                } else {
                    "Scroll down to load more"
                },
                Style::default().fg(t.dim),
            )]));
        }

        lines
    }

    /// Render the footer section.
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let t = theme();

        let hints = Line::from(vec![
            Span::styled("j/k", Style::default().fg(t.success)),
            Span::raw(": scroll  "),
            Span::styled("f/Tab", Style::default().fg(t.success)),
            Span::raw(": filter  "),
            Span::styled("g/G", Style::default().fg(t.success)),
            Span::raw(": top/bottom  "),
            Span::styled("q/Esc", Style::default().fg(t.success)),
            Span::raw(": close"),
        ]);

        let footer = Paragraph::new(hints).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(t.border)),
        );

        frame.render_widget(footer, area);
    }
}

impl Default for HistoryView {
    fn default() -> Self {
        Self::new()
    }
}

/// Truncate a string to the given length, adding ellipsis if needed.
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::User;
    use crate::ui::theme::{init_theme, Theme};
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_test_theme() {
        INIT.call_once(|| {
            init_theme(Theme::dark());
        });
    }

    fn create_test_history(id: &str, author_name: &str, date: &str, items: Vec<ChangeItem>) -> ChangeHistory {
        ChangeHistory {
            id: id.to_string(),
            author: User {
                account_id: "test".to_string(),
                display_name: author_name.to_string(),
                email_address: None,
                active: true,
                avatar_urls: None,
            },
            created: date.to_string(),
            items,
        }
    }

    fn create_test_item(field: &str, from: &str, to: &str) -> ChangeItem {
        ChangeItem {
            field: field.to_string(),
            field_type: Some("jira".to_string()),
            from_value: Some(from.to_string()),
            from_string: Some(from.to_string()),
            to_value: Some(to.to_string()),
            to_string: Some(to.to_string()),
        }
    }

    #[test]
    fn test_history_view_new() {
        let view = HistoryView::new();
        assert!(!view.is_visible());
        assert!(!view.is_loading());
        assert!(view.histories.is_empty());
    }

    #[test]
    fn test_history_view_show() {
        let mut view = HistoryView::new();
        view.show("TEST-123");

        assert!(view.is_visible());
        assert!(view.is_loading());
        assert_eq!(view.issue_key(), Some("TEST-123"));
    }

    #[test]
    fn test_history_view_hide() {
        let mut view = HistoryView::new();
        view.show("TEST-123");
        view.hide();

        assert!(!view.is_visible());
        assert!(!view.is_loading());
        assert!(view.issue_key().is_none());
    }

    #[test]
    fn test_history_view_set_changelog() {
        let mut view = HistoryView::new();
        view.show("TEST-123");

        let changelog = Changelog {
            histories: vec![create_test_history(
                "1",
                "John",
                "2024-01-15T10:00:00.000+0000",
                vec![create_test_item("status", "Open", "In Progress")],
            )],
            start_at: 0,
            max_results: 50,
            total: 1,
            is_last: true,
        };

        view.set_changelog(changelog);

        assert!(!view.is_loading());
        assert_eq!(view.histories.len(), 1);
        assert!(view.all_loaded);
    }

    #[test]
    fn test_history_filter_cycle() {
        assert_eq!(HistoryFilter::All.next(), HistoryFilter::Status);
        assert_eq!(HistoryFilter::Status.next(), HistoryFilter::Assignee);
        assert_eq!(HistoryFilter::Fields.next(), HistoryFilter::All);

        assert_eq!(HistoryFilter::All.prev(), HistoryFilter::Fields);
        assert_eq!(HistoryFilter::Status.prev(), HistoryFilter::All);
    }

    #[test]
    fn test_history_filter_matches() {
        let status_item = create_test_item("status", "Open", "Done");
        let assignee_item = create_test_item("assignee", "None", "John");
        let summary_item = create_test_item("summary", "Old", "New");
        let priority_item = create_test_item("priority", "Low", "High");

        assert!(HistoryFilter::All.matches(&status_item));
        assert!(HistoryFilter::All.matches(&assignee_item));

        assert!(HistoryFilter::Status.matches(&status_item));
        assert!(!HistoryFilter::Status.matches(&assignee_item));

        assert!(HistoryFilter::Assignee.matches(&assignee_item));
        assert!(!HistoryFilter::Assignee.matches(&status_item));

        assert!(HistoryFilter::Content.matches(&summary_item));
        assert!(!HistoryFilter::Content.matches(&status_item));

        assert!(HistoryFilter::Fields.matches(&priority_item));
        assert!(!HistoryFilter::Fields.matches(&status_item));
    }

    #[test]
    fn test_handle_input_close() {
        let mut view = HistoryView::new();
        view.show("TEST-123");

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert_eq!(action, Some(HistoryAction::Close));
        assert!(!view.is_visible());
    }

    #[test]
    fn test_handle_input_scroll() {
        let mut view = HistoryView::new();
        view.show("TEST-123");
        view.total_lines = 100;
        view.visible_height = 20;

        // Scroll down
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let _ = view.handle_input(key);
        assert_eq!(view.scroll, 1);

        // Scroll up
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let _ = view.handle_input(key);
        assert_eq!(view.scroll, 0);
    }

    #[test]
    fn test_handle_input_filter() {
        let mut view = HistoryView::new();
        view.show("TEST-123");

        assert_eq!(view.filter, HistoryFilter::All);

        let key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
        let _ = view.handle_input(key);
        assert_eq!(view.filter, HistoryFilter::Status);

        let key = KeyEvent::new(KeyCode::Char('F'), KeyModifiers::SHIFT);
        let _ = view.handle_input(key);
        assert_eq!(view.filter, HistoryFilter::All);
    }

    #[test]
    fn test_handle_input_go_to_top() {
        let mut view = HistoryView::new();
        view.show("TEST-123");
        view.scroll = 50;

        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        let _ = view.handle_input(key);
        assert_eq!(view.scroll, 0);
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("short", 10), "short");
        assert_eq!(truncate_str("this is a longer string", 10), "this is...");
        assert_eq!(truncate_str("abc", 3), "abc");
    }

    #[test]
    fn test_build_content_lines_empty() {
        init_test_theme();
        let mut view = HistoryView::new();
        view.show("TEST-123");
        view.loading = false;

        let lines = view.build_content_lines();
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_build_content_lines_with_history() {
        init_test_theme();
        let mut view = HistoryView::new();
        view.show("TEST-123");

        let changelog = Changelog {
            histories: vec![
                create_test_history(
                    "1",
                    "John Doe",
                    "2024-01-15T10:00:00.000+0000",
                    vec![create_test_item("status", "Open", "In Progress")],
                ),
                create_test_history(
                    "2",
                    "Jane Smith",
                    "2024-01-15T14:30:00.000+0000",
                    vec![create_test_item("assignee", "None", "John Doe")],
                ),
            ],
            start_at: 0,
            max_results: 50,
            total: 2,
            is_last: true,
        };

        view.set_changelog(changelog);
        let lines = view.build_content_lines();

        // Should have date header, entries, etc.
        assert!(lines.len() > 2);
    }

    #[test]
    fn test_can_load_more() {
        let mut view = HistoryView::new();
        view.show("TEST-123");
        view.loading = false;
        view.all_loaded = false;

        assert!(view.can_load_more());

        view.loading = true;
        assert!(!view.can_load_more());

        view.loading = false;
        view.all_loaded = true;
        assert!(!view.can_load_more());
    }

    #[test]
    fn test_next_start() {
        let mut view = HistoryView::new();
        view.show("TEST-123");

        assert_eq!(view.next_start(), 0);

        let changelog = Changelog {
            histories: vec![
                create_test_history("1", "John", "2024-01-15T10:00:00.000+0000", vec![]),
                create_test_history("2", "Jane", "2024-01-15T14:00:00.000+0000", vec![]),
            ],
            start_at: 0,
            max_results: 50,
            total: 10,
            is_last: false,
        };

        view.set_changelog(changelog);
        assert_eq!(view.next_start(), 2);
    }
}
