//! Comments component for viewing and adding comments.
//!
//! Displays comments for an issue and provides an input form
//! for adding new comments.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

use crate::api::types::Comment;
use super::TextEditor;

/// Action resulting from comments panel input.
#[derive(Debug, Clone, PartialEq)]
pub enum CommentAction {
    /// Submit a new comment with the given text.
    Submit(String),
    /// Cancel the comment input / close panel.
    Cancel,
    /// Request to load comments for an issue.
    LoadComments(String),
}

/// The current mode of the comments panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommentPanelMode {
    /// Viewing existing comments.
    #[default]
    Viewing,
    /// Composing a new comment.
    Composing,
}

/// Comments panel component.
///
/// Shows existing comments and allows adding new ones.
#[derive(Debug)]
pub struct CommentsPanel {
    /// The list of comments.
    comments: Vec<Comment>,
    /// Total number of comments (may be more than loaded).
    total_comments: u32,
    /// Whether the panel is visible.
    visible: bool,
    /// Whether comments are loading.
    loading: bool,
    /// Current scroll offset for viewing comments.
    scroll_offset: usize,
    /// Maximum scroll offset based on content.
    max_scroll: usize,
    /// Current mode (viewing or composing).
    mode: CommentPanelMode,
    /// Text editor for composing new comment.
    editor: TextEditor,
    /// Whether the comment is being submitted.
    submitting: bool,
    /// The issue key for which comments are displayed.
    issue_key: String,
}

impl CommentsPanel {
    /// Create a new comments panel.
    pub fn new() -> Self {
        Self {
            comments: Vec::new(),
            total_comments: 0,
            visible: false,
            loading: false,
            scroll_offset: 0,
            max_scroll: 0,
            mode: CommentPanelMode::Viewing,
            editor: TextEditor::empty(),
            submitting: false,
            issue_key: String::new(),
        }
    }

    /// Check if the panel is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if comments are loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Check if a comment is being submitted.
    pub fn is_submitting(&self) -> bool {
        self.submitting
    }

    /// Get the current mode.
    pub fn mode(&self) -> CommentPanelMode {
        self.mode
    }

    /// Get the number of loaded comments.
    pub fn comment_count(&self) -> usize {
        self.comments.len()
    }

    /// Get the total number of comments.
    pub fn total_count(&self) -> u32 {
        self.total_comments
    }

    /// Show the panel for an issue.
    pub fn show(&mut self, issue_key: &str) {
        self.issue_key = issue_key.to_string();
        self.comments.clear();
        self.total_comments = 0;
        self.scroll_offset = 0;
        self.max_scroll = 0;
        self.mode = CommentPanelMode::Viewing;
        self.editor = TextEditor::empty();
        self.loading = true;
        self.visible = true;
        self.submitting = false;
    }

    /// Hide the panel.
    pub fn hide(&mut self) {
        self.visible = false;
        self.loading = false;
        self.submitting = false;
        self.mode = CommentPanelMode::Viewing;
    }

    /// Set the comments to display.
    pub fn set_comments(&mut self, comments: Vec<Comment>, total: u32) {
        self.comments = comments;
        self.total_comments = total;
        self.loading = false;
        self.scroll_offset = 0;
    }

    /// Add a newly created comment to the list.
    pub fn add_comment(&mut self, comment: Comment) {
        // Insert at the beginning (newest first)
        self.comments.insert(0, comment);
        self.total_comments += 1;
        self.submitting = false;
        self.mode = CommentPanelMode::Viewing;
        self.editor = TextEditor::empty();
        self.scroll_offset = 0;
    }

    /// Set loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Set submitting state.
    pub fn set_submitting(&mut self, submitting: bool) {
        self.submitting = submitting;
    }

    /// Start composing a new comment.
    pub fn start_composing(&mut self) {
        self.mode = CommentPanelMode::Composing;
        self.editor = TextEditor::empty();
    }

    /// Cancel composing and return to viewing.
    pub fn cancel_composing(&mut self) {
        self.mode = CommentPanelMode::Viewing;
        self.editor = TextEditor::empty();
    }

    /// Get the issue key.
    pub fn issue_key(&self) -> &str {
        &self.issue_key
    }

    /// Set the maximum scroll based on content height.
    pub fn set_max_scroll(&mut self, max: usize) {
        self.max_scroll = max;
    }

    /// Get the current scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the parent view.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<CommentAction> {
        if !self.visible {
            return None;
        }

        // Handle submitting state - only allow Esc
        if self.submitting {
            if key.code == KeyCode::Esc {
                self.submitting = false;
            }
            return None;
        }

        match self.mode {
            CommentPanelMode::Viewing => self.handle_viewing_input(key),
            CommentPanelMode::Composing => self.handle_composing_input(key),
        }
    }

    /// Handle input in viewing mode.
    fn handle_viewing_input(&mut self, key: KeyEvent) -> Option<CommentAction> {
        match (key.code, key.modifiers) {
            // Scroll down
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                if self.scroll_offset < self.max_scroll {
                    self.scroll_offset += 1;
                }
                None
            }
            // Scroll up
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
                None
            }
            // Page down
            (KeyCode::Char('d'), KeyModifiers::CONTROL) | (KeyCode::PageDown, _) => {
                self.scroll_offset = (self.scroll_offset + 10).min(self.max_scroll);
                None
            }
            // Page up
            (KeyCode::Char('u'), KeyModifiers::CONTROL) | (KeyCode::PageUp, _) => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                None
            }
            // Go to top
            (KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.scroll_offset = 0;
                None
            }
            // Go to bottom
            (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                self.scroll_offset = self.max_scroll;
                None
            }
            // Add comment
            (KeyCode::Char('a'), KeyModifiers::NONE) | (KeyCode::Char('c'), KeyModifiers::NONE) => {
                self.start_composing();
                None
            }
            // Close panel
            (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.hide();
                Some(CommentAction::Cancel)
            }
            _ => None,
        }
    }

    /// Handle input in composing mode.
    fn handle_composing_input(&mut self, key: KeyEvent) -> Option<CommentAction> {
        match (key.code, key.modifiers) {
            // Submit comment with Ctrl+S (consistent with edit mode, works on macOS)
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                let content = self.editor.content().trim().to_string();
                if !content.is_empty() {
                    self.submitting = true;
                    return Some(CommentAction::Submit(content));
                }
                None
            }
            // Cancel composing
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.cancel_composing();
                None
            }
            // Forward other input to editor
            _ => {
                self.editor.handle_input(key);
                None
            }
        }
    }

    /// Render the comments panel.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate dialog size (larger panel for comments)
        let dialog_width = (area.width * 8 / 10).max(60).min(area.width.saturating_sub(4));
        let dialog_height = (area.height * 8 / 10).max(20).min(area.height.saturating_sub(4));

        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Clear the background
        frame.render_widget(Clear, dialog_area);

        // Create the dialog block
        let title = format!(
            " Comments ({}) ",
            if self.loading {
                "loading...".to_string()
            } else {
                format!("{}", self.total_comments)
            }
        );
        let block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        if self.loading {
            let loading_text = Paragraph::new("Loading comments...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading_text, inner);
            return;
        }

        // Split inner area based on mode
        match self.mode {
            CommentPanelMode::Viewing => {
                self.render_viewing_mode(frame, inner);
            }
            CommentPanelMode::Composing => {
                self.render_composing_mode(frame, inner);
            }
        }
    }

    /// Render viewing mode.
    fn render_viewing_mode(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Comments area
                Constraint::Length(2), // Help text
            ])
            .split(area);

        if self.comments.is_empty() {
            let empty_text = Paragraph::new("No comments yet. Press 'a' or 'c' to add one.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_text, chunks[0]);
        } else {
            self.render_comments_list(frame, chunks[0]);
        }

        // Render help text
        let help_text = Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(": scroll  "),
            Span::styled("a/c", Style::default().fg(Color::Green)),
            Span::raw(": add comment  "),
            Span::styled("q/Esc", Style::default().fg(Color::Red)),
            Span::raw(": close"),
        ]);
        let help_paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
        frame.render_widget(help_paragraph, chunks[1]);
    }

    /// Render the list of comments.
    fn render_comments_list(&mut self, frame: &mut Frame, area: Rect) {
        // Build comment lines
        let mut lines: Vec<Line> = Vec::new();
        let content_width = area.width.saturating_sub(2) as usize;

        for (i, comment) in self.comments.iter().enumerate() {
            if i > 0 {
                // Separator between comments
                lines.push(Line::from(Span::styled(
                    "─".repeat(content_width.min(60)),
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::raw(""));
            }

            // Author and date
            let author = &comment.author.display_name;
            let date = format_date(&comment.created);
            lines.push(Line::from(vec![
                Span::styled(
                    author.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {}", date),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));

            // Comment body
            let body_text = comment.body.to_plain_text();
            for line in body_text.lines() {
                // Wrap long lines
                for wrapped_line in wrap_text(line, content_width) {
                    lines.push(Line::from(wrapped_line));
                }
            }
            lines.push(Line::raw(""));
        }

        // Calculate max scroll
        let visible_height = area.height.saturating_sub(0) as usize;
        self.max_scroll = lines.len().saturating_sub(visible_height);

        // Skip to scroll offset
        let visible_lines: Vec<Line> = lines
            .into_iter()
            .skip(self.scroll_offset)
            .take(visible_height)
            .collect();

        let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);

        // Render scrollbar if needed
        if self.max_scroll > 0 {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"));
            let mut scrollbar_state = ScrollbarState::new(self.max_scroll)
                .position(self.scroll_offset);
            frame.render_stateful_widget(
                scrollbar,
                area,
                &mut scrollbar_state,
            );
        }
    }

    /// Render composing mode.
    fn render_composing_mode(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Editor area
                Constraint::Length(2), // Help text
            ])
            .split(area);

        // Render editor
        if self.submitting {
            let submitting_text = Paragraph::new("Submitting comment...")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title(" New Comment ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)),
                );
            frame.render_widget(submitting_text, chunks[0]);
        } else {
            self.editor.render(frame, chunks[0], true, Some(" New Comment (Ctrl+S to submit) "));
        }

        // Render help text
        let help_text = Line::from(vec![
            Span::styled("Ctrl+S", Style::default().fg(Color::Green)),
            Span::raw(": submit  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": cancel"),
        ]);
        let help_paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
        frame.render_widget(help_paragraph, chunks[1]);
    }
}

impl Default for CommentsPanel {
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

/// Format a date string for display.
fn format_date(date_str: &str) -> String {
    // Parse ISO date and format as readable string
    // Expected format: 2024-01-15T10:30:00.000+0000
    if let Some(date_part) = date_str.get(0..10) {
        if let Some(time_part) = date_str.get(11..16) {
            return format!("{} {}", date_part, time_part);
        }
        return date_part.to_string();
    }
    date_str.to_string()
}

/// Wrap text to fit within a given width.
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if text.is_empty() || max_width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{AtlassianDoc, User};

    fn create_test_comment(id: &str, author: &str, body: &str) -> Comment {
        Comment {
            id: id.to_string(),
            body: AtlassianDoc::from_text(body),
            author: User {
                account_id: format!("user-{}", id),
                display_name: author.to_string(),
                email_address: None,
                avatar_urls: None,
                active: true,
            },
            created: "2024-01-15T10:30:00.000+0000".to_string(),
            updated: "2024-01-15T10:30:00.000+0000".to_string(),
            self_url: None,
        }
    }

    #[test]
    fn test_new_panel() {
        let panel = CommentsPanel::new();
        assert!(!panel.is_visible());
        assert!(!panel.is_loading());
        assert_eq!(panel.comment_count(), 0);
        assert_eq!(panel.mode(), CommentPanelMode::Viewing);
    }

    #[test]
    fn test_show() {
        let mut panel = CommentsPanel::new();
        panel.show("TEST-123");

        assert!(panel.is_visible());
        assert!(panel.is_loading());
        assert_eq!(panel.issue_key(), "TEST-123");
        assert_eq!(panel.mode(), CommentPanelMode::Viewing);
    }

    #[test]
    fn test_hide() {
        let mut panel = CommentsPanel::new();
        panel.show("TEST-123");
        assert!(panel.is_visible());

        panel.hide();
        assert!(!panel.is_visible());
        assert!(!panel.is_loading());
    }

    #[test]
    fn test_set_comments() {
        let mut panel = CommentsPanel::new();
        panel.show("TEST-123");

        let comments = vec![
            create_test_comment("1", "Alice", "First comment"),
            create_test_comment("2", "Bob", "Second comment"),
        ];
        panel.set_comments(comments, 2);

        assert!(!panel.is_loading());
        assert_eq!(panel.comment_count(), 2);
        assert_eq!(panel.total_count(), 2);
    }

    #[test]
    fn test_add_comment() {
        let mut panel = CommentsPanel::new();
        panel.show("TEST-123");
        panel.set_comments(vec![], 0);

        let comment = create_test_comment("1", "Alice", "New comment");
        panel.add_comment(comment);

        assert_eq!(panel.comment_count(), 1);
        assert_eq!(panel.total_count(), 1);
        assert!(!panel.is_submitting());
        assert_eq!(panel.mode(), CommentPanelMode::Viewing);
    }

    #[test]
    fn test_composing_mode() {
        let mut panel = CommentsPanel::new();
        panel.show("TEST-123");
        panel.set_comments(vec![], 0);

        panel.start_composing();
        assert_eq!(panel.mode(), CommentPanelMode::Composing);

        panel.cancel_composing();
        assert_eq!(panel.mode(), CommentPanelMode::Viewing);
    }

    #[test]
    fn test_scroll_in_viewing_mode() {
        let mut panel = CommentsPanel::new();
        panel.show("TEST-123");
        panel.set_comments(vec![], 0);
        panel.set_max_scroll(10);

        // Scroll down
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        panel.handle_input(key);
        assert_eq!(panel.scroll_offset(), 1);

        // Scroll up
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        panel.handle_input(key);
        assert_eq!(panel.scroll_offset(), 0);
    }

    #[test]
    fn test_start_composing_with_a() {
        let mut panel = CommentsPanel::new();
        panel.show("TEST-123");
        panel.set_comments(vec![], 0);

        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        panel.handle_input(key);
        assert_eq!(panel.mode(), CommentPanelMode::Composing);
    }

    #[test]
    fn test_start_composing_with_c() {
        let mut panel = CommentsPanel::new();
        panel.show("TEST-123");
        panel.set_comments(vec![], 0);

        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        panel.handle_input(key);
        assert_eq!(panel.mode(), CommentPanelMode::Composing);
    }

    #[test]
    fn test_cancel_with_esc() {
        let mut panel = CommentsPanel::new();
        panel.show("TEST-123");
        panel.set_comments(vec![], 0);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = panel.handle_input(key);

        assert_eq!(action, Some(CommentAction::Cancel));
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_cancel_with_q() {
        let mut panel = CommentsPanel::new();
        panel.show("TEST-123");
        panel.set_comments(vec![], 0);

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = panel.handle_input(key);

        assert_eq!(action, Some(CommentAction::Cancel));
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_cancel_composing_with_esc() {
        let mut panel = CommentsPanel::new();
        panel.show("TEST-123");
        panel.set_comments(vec![], 0);
        panel.start_composing();

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        panel.handle_input(key);

        assert_eq!(panel.mode(), CommentPanelMode::Viewing);
        assert!(panel.is_visible());
    }

    #[test]
    fn test_input_ignored_when_not_visible() {
        let mut panel = CommentsPanel::new();

        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = panel.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_format_date() {
        assert_eq!(
            format_date("2024-01-15T10:30:00.000+0000"),
            "2024-01-15 10:30"
        );
        assert_eq!(format_date("2024-01-15"), "2024-01-15");
        assert_eq!(format_date("invalid"), "invalid");
    }

    #[test]
    fn test_wrap_text() {
        assert_eq!(wrap_text("hello world", 20), vec!["hello world"]);
        assert_eq!(
            wrap_text("hello world test", 10),
            vec!["hello", "world test"]
        );
        assert_eq!(wrap_text("", 10), vec![""]);
    }

    #[test]
    fn test_default_impl() {
        let panel = CommentsPanel::default();
        assert!(!panel.is_visible());
    }
}
