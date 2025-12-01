//! Tag editor component for labels and components.
//!
//! A reusable chip-based editor that displays current tags as chips and allows
//! adding/removing tags from a list of available options.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use super::InputMode;

/// Action resulting from tag editor input.
#[derive(Debug, Clone, PartialEq)]
pub enum TagAction {
    /// Add a tag (tag name).
    Add(String),
    /// Remove a tag (tag name).
    Remove(String),
    /// Cancel the editor.
    Cancel,
}

/// Configuration for the tag editor appearance.
#[derive(Debug, Clone)]
pub struct TagEditorConfig {
    /// Title for the editor dialog.
    pub title: String,
    /// Color for tag chips.
    pub tag_color: Color,
    /// Label text for "current items".
    pub current_label: String,
    /// Label text for "available items".
    pub available_label: String,
}

impl TagEditorConfig {
    /// Create config for editing labels.
    pub fn labels() -> Self {
        Self {
            title: " Edit Labels ".to_string(),
            tag_color: Color::Blue,
            current_label: "Current Labels".to_string(),
            available_label: "Available Labels".to_string(),
        }
    }

    /// Create config for editing components.
    pub fn components() -> Self {
        Self {
            title: " Edit Components ".to_string(),
            tag_color: Color::Magenta,
            current_label: "Current Components".to_string(),
            available_label: "Available Components".to_string(),
        }
    }
}

/// Tag editor component.
///
/// Shows current tags as chips and allows adding/removing from available options.
/// Supports search filtering and keyboard navigation.
///
/// Supports two modes (vim-style):
/// - Normal mode: j/k navigate, Tab switches sections, / starts search, Enter/Space selects, q/Esc cancels
/// - Search mode: type to filter, Enter/Space selects, Esc returns to Normal mode
#[derive(Debug)]
pub struct TagEditor {
    /// Configuration for the editor.
    config: TagEditorConfig,
    /// Currently assigned tags.
    current_tags: Vec<String>,
    /// All available tags (options).
    available_tags: Vec<String>,
    /// Which section is focused: true = current tags, false = available tags.
    focus_on_current: bool,
    /// Currently selected index in the focused section.
    selected: usize,
    /// Whether the editor is visible.
    visible: bool,
    /// Whether tags are loading.
    loading: bool,
    /// Search/filter query.
    search_query: String,
    /// Filtered available tags indices.
    filtered_indices: Vec<usize>,
    /// Current input mode (Normal for navigation, Insert for typing).
    input_mode: InputMode,
}

impl TagEditor {
    /// Create a new tag editor with the given configuration.
    pub fn new(config: TagEditorConfig) -> Self {
        Self {
            config,
            current_tags: Vec::new(),
            available_tags: Vec::new(),
            focus_on_current: false,
            selected: 0,
            visible: false,
            loading: false,
            search_query: String::new(),
            filtered_indices: Vec::new(),
            input_mode: InputMode::Normal,
        }
    }

    /// Create a new tag editor for labels.
    pub fn for_labels() -> Self {
        Self::new(TagEditorConfig::labels())
    }

    /// Create a new tag editor for components.
    pub fn for_components() -> Self {
        Self::new(TagEditorConfig::components())
    }

    /// Check if the editor is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if tags are loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Show the editor with loading state.
    pub fn show_loading(&mut self, current_tags: Vec<String>) {
        self.current_tags = current_tags;
        self.available_tags.clear();
        self.focus_on_current = false;
        self.selected = 0;
        self.search_query.clear();
        self.filtered_indices.clear();
        self.loading = true;
        self.visible = true;
    }

    /// Show the editor with the given available tags.
    pub fn show(&mut self, current_tags: Vec<String>, available_tags: Vec<String>) {
        self.current_tags = current_tags;
        self.available_tags = available_tags;
        self.focus_on_current = false;
        self.selected = 0;
        self.search_query.clear();
        self.update_filtered_indices();
        self.loading = false;
        self.visible = true;
    }

    /// Hide the editor.
    pub fn hide(&mut self) {
        self.visible = false;
        self.loading = false;
        self.search_query.clear();
        self.input_mode = InputMode::Normal;
    }

    /// Update filtered indices based on search query.
    fn update_filtered_indices(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.available_tags.len()).collect();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_indices = self
                .available_tags
                .iter()
                .enumerate()
                .filter(|(_, tag)| tag.to_lowercase().contains(&query_lower))
                .map(|(i, _)| i)
                .collect();
        }
        // Reset selection when filter changes
        self.selected = 0;
    }

    /// Get the count of items in the currently focused section.
    fn focused_count(&self) -> usize {
        if self.focus_on_current {
            self.current_tags.len()
        } else {
            self.filtered_indices.len()
        }
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the parent view.
    ///
    /// Two modes (vim-style):
    /// - Normal mode: j/k navigate, Tab switches sections, / starts search, Enter/Space selects, q/Esc cancel
    /// - Search mode: type to filter, Enter/Space selects, Esc returns to Normal mode
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<TagAction> {
        if !self.visible {
            return None;
        }

        // Don't handle input while loading (except Esc)
        if self.loading {
            if key.code == KeyCode::Esc {
                self.hide();
                return Some(TagAction::Cancel);
            }
            return None;
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_input(key),
            InputMode::Insert => self.handle_insert_input(key),
        }
    }

    /// Handle input in Normal mode (navigation).
    fn handle_normal_input(&mut self, key: KeyEvent) -> Option<TagAction> {
        match (key.code, key.modifiers) {
            // Navigation down with j or arrow
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                if self.focused_count() > 0
                    && self.selected < self.focused_count().saturating_sub(1)
                {
                    self.selected += 1;
                }
                None
            }
            // Navigation up with k or arrow
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            // Switch focus between sections
            (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::BackTab, KeyModifiers::SHIFT) => {
                self.focus_on_current = !self.focus_on_current;
                self.selected = 0;
                // Reset to Normal mode when switching sections
                self.input_mode = InputMode::Normal;
                None
            }
            // '/' - enter search mode (vim-style, only on available section)
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                if !self.focus_on_current {
                    self.input_mode = InputMode::Insert;
                }
                None
            }
            // Enter or Space - add/remove current item
            (KeyCode::Enter, KeyModifiers::NONE) | (KeyCode::Char(' '), KeyModifiers::NONE) => {
                if self.focus_on_current {
                    self.remove_current_tag()
                } else {
                    self.add_current_tag()
                }
            }
            // Cancel with q or Esc
            (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.hide();
                Some(TagAction::Cancel)
            }
            _ => None,
        }
    }

    /// Handle input in Insert mode (typing to search).
    fn handle_insert_input(&mut self, key: KeyEvent) -> Option<TagAction> {
        match (key.code, key.modifiers) {
            // Enter or Esc - complete search and return to Normal mode
            (KeyCode::Enter, KeyModifiers::NONE) | (KeyCode::Esc, KeyModifiers::NONE) => {
                self.input_mode = InputMode::Normal;
                None
            }
            // Arrow keys still work for navigation in search mode
            (KeyCode::Down, _) => {
                if self.focused_count() > 0
                    && self.selected < self.focused_count().saturating_sub(1)
                {
                    self.selected += 1;
                }
                None
            }
            (KeyCode::Up, _) => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            // Tab switches sections (and returns to Normal mode)
            (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::BackTab, KeyModifiers::SHIFT) => {
                self.focus_on_current = !self.focus_on_current;
                self.selected = 0;
                self.input_mode = InputMode::Normal;
                None
            }
            // Backspace - delete from search
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                if !self.search_query.is_empty() {
                    self.search_query.pop();
                    self.update_filtered_indices();
                }
                None
            }
            // Character input - add to search (includes j/k/q)
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT)
                if c.is_alphanumeric() || c == '-' || c == '_' || c.is_whitespace() =>
            {
                self.search_query.push(c);
                self.update_filtered_indices();
                None
            }
            _ => None,
        }
    }

    /// Remove the currently selected tag from current tags.
    fn remove_current_tag(&mut self) -> Option<TagAction> {
        if let Some(tag) = self.current_tags.get(self.selected).cloned() {
            self.current_tags.remove(self.selected);
            // Adjust selection if needed
            if self.selected >= self.current_tags.len() && self.selected > 0 {
                self.selected -= 1;
            }
            Some(TagAction::Remove(tag))
        } else {
            None
        }
    }

    /// Add the currently selected available tag.
    fn add_current_tag(&mut self) -> Option<TagAction> {
        if let Some(&idx) = self.filtered_indices.get(self.selected) {
            if let Some(tag) = self.available_tags.get(idx).cloned() {
                // Only add if not already in current
                if !self.current_tags.contains(&tag) {
                    self.current_tags.push(tag.clone());
                    return Some(TagAction::Add(tag));
                }
            }
        }
        None
    }

    /// Render the tag editor.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate dialog size and position (centered)
        let dialog_width = 72.min(area.width.saturating_sub(4));
        let dialog_height = 22.min(area.height.saturating_sub(4));

        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Clear the background
        frame.render_widget(Clear, dialog_area);

        // Create the dialog block
        let block = Block::default()
            .title(self.config.title.as_str())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.config.tag_color));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Split inner area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Current tags (chips)
                Constraint::Length(2), // Search bar
                Constraint::Min(6),    // Available tags list
                Constraint::Length(2), // Help text
            ])
            .split(inner);

        // Render current tags section
        self.render_current_tags(frame, chunks[0]);

        // Render search bar
        self.render_search_bar(frame, chunks[1]);

        // Render available tags or loading
        if self.loading {
            let loading_text = Paragraph::new("Loading available options...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading_text, chunks[2]);
        } else {
            self.render_available_tags(frame, chunks[2]);
        }

        // Render help text
        self.render_help(frame, chunks[3]);
    }

    /// Render the current tags section with chips.
    fn render_current_tags(&self, frame: &mut Frame, area: Rect) {
        let section_style = if self.focus_on_current {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let title = if self.focus_on_current {
            format!(" {} (focused) ", self.config.current_label)
        } else {
            format!(" {} ", self.config.current_label)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(section_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.current_tags.is_empty() {
            let empty_text =
                Paragraph::new("No tags assigned").style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty_text, inner);
        } else if self.focus_on_current {
            // When focused, show as a list for selection
            let items: Vec<ListItem> = self
                .current_tags
                .iter()
                .map(|tag| {
                    ListItem::new(Line::from(Span::styled(
                        format!("  {} ", tag),
                        Style::default().fg(Color::White).bg(self.config.tag_color),
                    )))
                })
                .collect();

            let list = List::new(items)
                .highlight_style(
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            let mut state = ListState::default();
            state.select(Some(self.selected));

            frame.render_stateful_widget(list, inner, &mut state);
        } else {
            // When not focused, show as inline chips
            let mut spans: Vec<Span> = Vec::new();
            for (i, tag) in self.current_tags.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw(" "));
                }
                spans.push(Span::styled(
                    format!(" {} ", tag),
                    Style::default().fg(Color::White).bg(self.config.tag_color),
                ));
            }
            let line = Line::from(spans);
            let paragraph = Paragraph::new(line).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, inner);
        }
    }

    /// Render the search bar.
    fn render_search_bar(&self, frame: &mut Frame, area: Rect) {
        let search_line = if self.focus_on_current {
            // When focused on current tags, show dimmed search hint
            Line::from(Span::styled(
                "Press / to search available options...",
                Style::default().fg(Color::DarkGray),
            ))
        } else {
            match self.input_mode {
                InputMode::Insert => {
                    // Show "/" prompt when in search mode
                    Line::from(vec![
                        Span::styled(
                            "/",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(&self.search_query, Style::default().fg(Color::White)),
                        Span::styled("▏", Style::default().fg(Color::Yellow)), // Cursor
                    ])
                }
                InputMode::Normal => {
                    if self.search_query.is_empty() {
                        Line::from(Span::styled(
                            "Press / to search...",
                            Style::default().fg(Color::DarkGray),
                        ))
                    } else {
                        // Show current filter
                        Line::from(vec![
                            Span::styled("/", Style::default().fg(Color::DarkGray)),
                            Span::styled(&self.search_query, Style::default().fg(Color::Cyan)),
                        ])
                    }
                }
            }
        };
        frame.render_widget(Paragraph::new(search_line), area);
    }

    /// Render the available tags list.
    fn render_available_tags(&self, frame: &mut Frame, area: Rect) {
        let section_style = if !self.focus_on_current {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let title = if !self.focus_on_current {
            format!(" {} (focused) ", self.config.available_label)
        } else {
            format!(" {} ", self.config.available_label)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(section_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.filtered_indices.is_empty() {
            let empty_text = if self.search_query.is_empty() {
                "No options available"
            } else {
                "No matching options"
            };
            let paragraph = Paragraph::new(empty_text).style(Style::default().fg(Color::DarkGray));
            frame.render_widget(paragraph, inner);
        } else {
            let items: Vec<ListItem> = self
                .filtered_indices
                .iter()
                .map(|&idx| {
                    let tag = &self.available_tags[idx];
                    let is_assigned = self.current_tags.contains(tag);
                    let style = if is_assigned {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default()
                    };
                    let prefix = if is_assigned { "[+] " } else { "    " };
                    ListItem::new(format!("{}{}", prefix, tag)).style(style)
                })
                .collect();

            let list = List::new(items);

            if self.focus_on_current {
                frame.render_widget(list, inner);
            } else {
                let list = list
                    .highlight_style(
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol("> ");

                let mut state = ListState::default();
                state.select(Some(self.selected));

                frame.render_stateful_widget(list, inner, &mut state);
            }
        }
    }

    /// Render the help text.
    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help_text = match self.input_mode {
            InputMode::Normal => {
                if self.focus_on_current {
                    Line::from(vec![
                        Span::styled("j/k", Style::default().fg(Color::Yellow)),
                        Span::raw(": navigate  "),
                        Span::styled("Tab", Style::default().fg(Color::Cyan)),
                        Span::raw(": switch  "),
                        Span::styled("Enter", Style::default().fg(Color::Green)),
                        Span::raw(": remove  "),
                        Span::styled("q", Style::default().fg(Color::Red)),
                        Span::raw(": close"),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled("j/k", Style::default().fg(Color::Yellow)),
                        Span::raw(": navigate  "),
                        Span::styled("Tab", Style::default().fg(Color::Cyan)),
                        Span::raw(": switch  "),
                        Span::styled("/", Style::default().fg(Color::Cyan)),
                        Span::raw(": search  "),
                        Span::styled("Enter", Style::default().fg(Color::Green)),
                        Span::raw(": add  "),
                        Span::styled("q", Style::default().fg(Color::Red)),
                        Span::raw(": close"),
                    ])
                }
            }
            InputMode::Insert => Line::from(vec![
                Span::styled(
                    "-- SEARCH --",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  type to filter  "),
                Span::styled("Enter/Esc", Style::default().fg(Color::Yellow)),
                Span::raw(": done"),
            ]),
        };
        frame.render_widget(Paragraph::new(help_text).alignment(Alignment::Center), area);
    }
}

/// Create a centered rectangle.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_editor() {
        let editor = TagEditor::for_labels();
        assert!(!editor.is_visible());
        assert!(!editor.is_loading());
    }

    #[test]
    fn test_show_loading() {
        let mut editor = TagEditor::for_labels();
        editor.show_loading(vec!["bug".to_string()]);

        assert!(editor.is_visible());
        assert!(editor.is_loading());
        assert_eq!(editor.current_tags, vec!["bug"]);
    }

    #[test]
    fn test_show() {
        let mut editor = TagEditor::for_labels();
        let current = vec!["bug".to_string()];
        let available = vec!["bug".to_string(), "feature".to_string(), "docs".to_string()];

        editor.show(current, available);

        assert!(editor.is_visible());
        assert!(!editor.is_loading());
        assert_eq!(editor.current_tags.len(), 1);
        assert_eq!(editor.available_tags.len(), 3);
    }

    #[test]
    fn test_hide() {
        let mut editor = TagEditor::for_labels();
        editor.show_loading(vec![]);
        assert!(editor.is_visible());

        editor.hide();
        assert!(!editor.is_visible());
        assert!(!editor.is_loading());
    }

    #[test]
    fn test_navigation_down() {
        let mut editor = TagEditor::for_labels();
        let current = vec![];
        let available = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        editor.show(current, available);

        assert_eq!(editor.selected, 0);

        // Navigate down with arrow key
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let action = editor.handle_input(key);
        assert!(action.is_none());
        assert_eq!(editor.selected, 1);

        editor.handle_input(key);
        assert_eq!(editor.selected, 2);

        // Can't go past the end
        editor.handle_input(key);
        assert_eq!(editor.selected, 2);
    }

    #[test]
    fn test_navigation_up() {
        let mut editor = TagEditor::for_labels();
        let current = vec![];
        let available = vec!["a".to_string(), "b".to_string()];
        editor.show(current, available);
        editor.selected = 1;

        // Navigate up with arrow key
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let action = editor.handle_input(key);
        assert!(action.is_none());
        assert_eq!(editor.selected, 0);

        // Can't go past the beginning
        editor.handle_input(key);
        assert_eq!(editor.selected, 0);
    }

    #[test]
    fn test_switch_focus() {
        let mut editor = TagEditor::for_labels();
        let current = vec!["bug".to_string()];
        let available = vec!["bug".to_string(), "feature".to_string()];
        editor.show(current, available);

        assert!(!editor.focus_on_current);

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        editor.handle_input(key);
        assert!(editor.focus_on_current);
        assert_eq!(editor.selected, 0);

        editor.handle_input(key);
        assert!(!editor.focus_on_current);
    }

    #[test]
    fn test_add_tag_with_enter() {
        let mut editor = TagEditor::for_labels();
        let current = vec![];
        let available = vec!["bug".to_string(), "feature".to_string()];
        editor.show(current, available);

        // Select "bug" with Enter
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = editor.handle_input(key);

        assert_eq!(action, Some(TagAction::Add("bug".to_string())));
        assert!(editor.current_tags.contains(&"bug".to_string()));
    }

    #[test]
    fn test_add_tag_with_space() {
        let mut editor = TagEditor::for_labels();
        let current = vec![];
        let available = vec!["bug".to_string(), "feature".to_string()];
        editor.show(current, available);

        // Select "bug" with Space
        let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        let action = editor.handle_input(key);

        assert_eq!(action, Some(TagAction::Add("bug".to_string())));
        assert!(editor.current_tags.contains(&"bug".to_string()));
    }

    #[test]
    fn test_slash_enters_search_mode() {
        let mut editor = TagEditor::for_labels();
        let current = vec![];
        let available = vec!["bug".to_string(), "feature".to_string()];
        editor.show(current, available);

        // '/' in Normal mode on available section enters search mode
        let key = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        let action = editor.handle_input(key);

        assert!(action.is_none());
        assert_eq!(editor.input_mode, InputMode::Insert);
    }

    #[test]
    fn test_add_duplicate_tag_no_action() {
        let mut editor = TagEditor::for_labels();
        let current = vec!["bug".to_string()];
        let available = vec!["bug".to_string(), "feature".to_string()];
        editor.show(current, available);

        // Try to add "bug" again with Space - should not produce an action
        let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        let action = editor.handle_input(key);

        assert!(action.is_none());
    }

    #[test]
    fn test_remove_tag() {
        let mut editor = TagEditor::for_labels();
        let current = vec!["bug".to_string(), "feature".to_string()];
        let available = vec!["bug".to_string(), "feature".to_string()];
        editor.show(current, available);

        // Switch to current tags
        let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        editor.handle_input(tab);
        assert!(editor.focus_on_current);

        // Remove first tag
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = editor.handle_input(enter);

        assert_eq!(action, Some(TagAction::Remove("bug".to_string())));
        assert!(!editor.current_tags.contains(&"bug".to_string()));
        assert_eq!(editor.current_tags.len(), 1);
    }

    #[test]
    fn test_cancel_with_esc() {
        let mut editor = TagEditor::for_labels();
        editor.show(vec![], vec!["a".to_string()]);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = editor.handle_input(key);

        assert_eq!(action, Some(TagAction::Cancel));
        assert!(!editor.is_visible());
    }

    #[test]
    fn test_q_cancels_in_normal_mode() {
        let mut editor = TagEditor::for_labels();
        editor.show(vec![], vec!["a".to_string()]);

        // 'q' in Normal mode should cancel
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = editor.handle_input(key);

        assert_eq!(action, Some(TagAction::Cancel));
        assert!(!editor.is_visible());
    }

    #[test]
    fn test_q_adds_to_search_in_insert_mode() {
        let mut editor = TagEditor::for_labels();
        editor.show(vec![], vec!["a".to_string()]);

        // Enter search mode with '/'
        let slash = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        editor.handle_input(slash);
        assert_eq!(editor.input_mode, InputMode::Insert);

        // 'q' in search mode should add to search query
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = editor.handle_input(key);

        assert!(action.is_none()); // No action, character added to search
        assert!(editor.is_visible()); // Still visible
        assert_eq!(editor.search_query, "q");
    }

    #[test]
    fn test_esc_while_loading() {
        let mut editor = TagEditor::for_labels();
        editor.show_loading(vec![]);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = editor.handle_input(key);

        assert_eq!(action, Some(TagAction::Cancel));
        assert!(!editor.is_visible());
    }

    #[test]
    fn test_input_ignored_while_loading() {
        let mut editor = TagEditor::for_labels();
        editor.show_loading(vec![]);

        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = editor.handle_input(key);
        assert!(action.is_none());

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = editor.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_input_ignored_when_not_visible() {
        let mut editor = TagEditor::for_labels();

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = editor.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_search_filter() {
        let mut editor = TagEditor::for_labels();
        let available = vec![
            "backend".to_string(),
            "frontend".to_string(),
            "bug".to_string(),
            "feature".to_string(),
        ];
        editor.show(vec![], available);

        assert_eq!(editor.filtered_indices.len(), 4);

        // Enter search mode with '/'
        let slash = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        editor.handle_input(slash);

        // Type 'b' to filter
        let key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        editor.handle_input(key);

        // Should match "backend" and "bug"
        assert_eq!(editor.filtered_indices.len(), 2);
        assert_eq!(editor.selected, 0);

        // Backspace
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        editor.handle_input(key);
        assert_eq!(editor.filtered_indices.len(), 4);
    }

    #[test]
    fn test_search_not_active_when_focused_on_current() {
        let mut editor = TagEditor::for_labels();
        let current = vec!["bug".to_string()];
        let available = vec!["bug".to_string(), "feature".to_string()];
        editor.show(current, available);

        // Switch to current tags (in Normal mode, search is not available here)
        let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        editor.handle_input(tab);

        // Try to type - should not affect search (x is not handled in current section)
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        editor.handle_input(key);

        assert!(editor.search_query.is_empty());
    }

    #[test]
    fn test_space_to_add_remove() {
        let mut editor = TagEditor::for_labels();
        let available = vec!["bug".to_string()];
        editor.show(vec![], available);

        let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        let action = editor.handle_input(key);

        assert_eq!(action, Some(TagAction::Add("bug".to_string())));
    }

    #[test]
    fn test_config_labels() {
        let config = TagEditorConfig::labels();
        assert_eq!(config.tag_color, Color::Blue);
        assert!(config.title.contains("Labels"));
    }

    #[test]
    fn test_config_components() {
        let config = TagEditorConfig::components();
        assert_eq!(config.tag_color, Color::Magenta);
        assert!(config.title.contains("Components"));
    }

    #[test]
    fn test_remove_adjusts_selection() {
        let mut editor = TagEditor::for_labels();
        let current = vec!["a".to_string(), "b".to_string()];
        editor.show(current, vec![]);

        // Switch to current and select last item
        let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        editor.handle_input(tab);
        editor.selected = 1;

        // Remove "b"
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = editor.handle_input(enter);

        assert_eq!(action, Some(TagAction::Remove("b".to_string())));
        // Selection should adjust to remain valid
        assert_eq!(editor.selected, 0);
    }
}
