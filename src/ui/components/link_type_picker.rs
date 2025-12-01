//! Link type picker component for creating issue links.
//!
//! Displays available issue link types and allows the user to select
//! one when creating a new link between issues.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::api::types::IssueLinkType;

/// Action resulting from link type picker input.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkTypePickerAction {
    /// Select a link type (type name, inward description, outward description).
    /// The boolean indicates if this issue is the outward issue (true) or inward (false).
    Select(IssueLinkType, bool),
    /// Cancel the picker.
    Cancel,
}

/// Display item for link type selection.
/// Each link type can be used in two directions.
#[derive(Debug, Clone)]
struct LinkTypeItem {
    /// The underlying link type.
    link_type: IssueLinkType,
    /// The display text for this direction.
    display: String,
    /// True if selecting this makes the current issue the outward issue.
    is_outward: bool,
}

/// Link type picker component.
///
/// Shows available link types for creating issue links and allows
/// the user to select one using keyboard navigation.
#[derive(Debug)]
pub struct LinkTypePicker {
    /// Available link type items (each type appears twice - once for each direction).
    items: Vec<LinkTypeItem>,
    /// Currently selected index.
    selected: usize,
    /// Whether the picker is visible.
    visible: bool,
    /// Whether link types are loading.
    loading: bool,
}

impl LinkTypePicker {
    /// Create a new link type picker.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            visible: false,
            loading: false,
        }
    }

    /// Check if the picker is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if link types are loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Show the picker with loading state.
    pub fn show_loading(&mut self) {
        self.items.clear();
        self.selected = 0;
        self.loading = true;
        self.visible = true;
    }

    /// Show the picker with the given link types.
    ///
    /// Each link type is shown twice - once for each direction:
    /// - "blocks" (this issue blocks target)
    /// - "is blocked by" (this issue is blocked by target)
    pub fn show(&mut self, link_types: Vec<IssueLinkType>) {
        self.items.clear();

        for lt in link_types {
            // Outward direction: "this issue [outward] target"
            // e.g., "this issue blocks PROJ-123"
            self.items.push(LinkTypeItem {
                display: format!("This issue {} ...", lt.outward),
                link_type: lt.clone(),
                is_outward: true,
            });

            // Inward direction: "this issue [inward] target"
            // e.g., "this issue is blocked by PROJ-123"
            self.items.push(LinkTypeItem {
                display: format!("This issue {} ...", lt.inward),
                link_type: lt,
                is_outward: false,
            });
        }

        self.selected = 0;
        self.loading = false;
        self.visible = true;
    }

    /// Hide the picker.
    pub fn hide(&mut self) {
        self.visible = false;
        self.loading = false;
    }

    /// Get the number of available items.
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the parent view.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<LinkTypePickerAction> {
        if !self.visible {
            return None;
        }

        // Allow Esc while loading
        if self.loading {
            if key.code == KeyCode::Esc {
                self.hide();
                return Some(LinkTypePickerAction::Cancel);
            }
            return None;
        }

        match (key.code, key.modifiers) {
            // Navigation down with j or arrow
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                if !self.items.is_empty() && self.selected < self.items.len() - 1 {
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
            // Select link type
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(item) = self.items.get(self.selected) {
                    let result = LinkTypePickerAction::Select(
                        item.link_type.clone(),
                        item.is_outward,
                    );
                    self.hide();
                    Some(result)
                } else {
                    None
                }
            }
            // Cancel with q or Esc
            (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.hide();
                Some(LinkTypePickerAction::Cancel)
            }
            _ => None,
        }
    }

    /// Render the link type picker.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate dialog size and position (centered)
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 18.min(area.height.saturating_sub(4));

        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Clear the background
        frame.render_widget(Clear, dialog_area);

        // Create the dialog block
        let block = Block::default()
            .title(" Select Link Type ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Split inner area for description, list, and help
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Description
                Constraint::Min(3),    // Link types list
                Constraint::Length(2), // Help text
            ])
            .split(inner);

        // Render description
        let desc_text = Line::from(vec![
            Span::styled(
                "Choose how this issue relates to the target:",
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        let desc_paragraph = Paragraph::new(desc_text);
        frame.render_widget(desc_paragraph, chunks[0]);

        // Render loading or link types list
        if self.loading {
            let loading_text = Paragraph::new("Loading link types...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading_text, chunks[1]);
        } else if self.items.is_empty() {
            let empty_text = Paragraph::new("No link types available")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_text, chunks[1]);
        } else {
            // Build list items
            let items: Vec<ListItem> = self
                .items
                .iter()
                .map(|item| {
                    ListItem::new(item.display.clone())
                        .style(Style::default().fg(Color::White))
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
            state.select(Some(self.selected));

            frame.render_stateful_widget(list, chunks[1], &mut state);
        }

        // Render help text
        let help_text = Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(": navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": select  "),
            Span::styled("q/Esc", Style::default().fg(Color::Red)),
            Span::raw(": cancel"),
        ]);
        let help_paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
        frame.render_widget(help_paragraph, chunks[2]);
    }
}

impl Default for LinkTypePicker {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_link_type(id: &str, name: &str, inward: &str, outward: &str) -> IssueLinkType {
        IssueLinkType {
            id: id.to_string(),
            name: name.to_string(),
            inward: inward.to_string(),
            outward: outward.to_string(),
        }
    }

    #[test]
    fn test_new_picker() {
        let picker = LinkTypePicker::new();
        assert!(!picker.is_visible());
        assert!(!picker.is_loading());
        assert_eq!(picker.item_count(), 0);
    }

    #[test]
    fn test_show_loading() {
        let mut picker = LinkTypePicker::new();
        picker.show_loading();

        assert!(picker.is_visible());
        assert!(picker.is_loading());
    }

    #[test]
    fn test_show_link_types() {
        let mut picker = LinkTypePicker::new();
        let link_types = vec![
            create_test_link_type("1", "Blocks", "is blocked by", "blocks"),
            create_test_link_type("2", "Relates", "relates to", "relates to"),
        ];

        picker.show(link_types);

        assert!(picker.is_visible());
        assert!(!picker.is_loading());
        // Each link type should appear twice (once for each direction)
        assert_eq!(picker.item_count(), 4);
    }

    #[test]
    fn test_hide() {
        let mut picker = LinkTypePicker::new();
        picker.show_loading();
        assert!(picker.is_visible());

        picker.hide();
        assert!(!picker.is_visible());
        assert!(!picker.is_loading());
    }

    #[test]
    fn test_navigation_down() {
        let mut picker = LinkTypePicker::new();
        let link_types = vec![
            create_test_link_type("1", "Blocks", "is blocked by", "blocks"),
        ];
        picker.show(link_types);

        // Initial selection is 0
        assert_eq!(picker.selected, 0);

        // Navigate down with arrow key
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 1);

        // Can't go past the end
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 1);
    }

    #[test]
    fn test_navigation_up() {
        let mut picker = LinkTypePicker::new();
        let link_types = vec![
            create_test_link_type("1", "Blocks", "is blocked by", "blocks"),
        ];
        picker.show(link_types);
        picker.selected = 1;

        // Navigate up with arrow key
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 0);

        // Can't go past the beginning
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_select_outward_link_type() {
        let mut picker = LinkTypePicker::new();
        let link_types = vec![
            create_test_link_type("1", "Blocks", "is blocked by", "blocks"),
        ];
        picker.show(link_types);

        // First item is outward ("This issue blocks ...")
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        match action {
            Some(LinkTypePickerAction::Select(lt, is_outward)) => {
                assert_eq!(lt.id, "1");
                assert!(is_outward);
            }
            _ => panic!("Expected Select action"),
        }
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_select_inward_link_type() {
        let mut picker = LinkTypePicker::new();
        let link_types = vec![
            create_test_link_type("1", "Blocks", "is blocked by", "blocks"),
        ];
        picker.show(link_types);

        // Navigate to second item (inward direction)
        picker.selected = 1;

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        match action {
            Some(LinkTypePickerAction::Select(lt, is_outward)) => {
                assert_eq!(lt.id, "1");
                assert!(!is_outward);
            }
            _ => panic!("Expected Select action"),
        }
    }

    #[test]
    fn test_cancel_with_esc() {
        let mut picker = LinkTypePicker::new();
        picker.show(vec![create_test_link_type("1", "Blocks", "is blocked by", "blocks")]);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(LinkTypePickerAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_cancel_with_q() {
        let mut picker = LinkTypePicker::new();
        picker.show(vec![create_test_link_type("1", "Blocks", "is blocked by", "blocks")]);

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(LinkTypePickerAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_navigation_with_j_k() {
        let mut picker = LinkTypePicker::new();
        let link_types = vec![
            create_test_link_type("1", "Blocks", "is blocked by", "blocks"),
        ];
        picker.show(link_types);

        assert_eq!(picker.selected, 0);

        // Move down with j
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 1);

        // Move up with k
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_esc_while_loading() {
        let mut picker = LinkTypePicker::new();
        picker.show_loading();

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(LinkTypePickerAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_input_ignored_while_loading() {
        let mut picker = LinkTypePicker::new();
        picker.show_loading();

        // Navigation should be ignored while loading
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());

        // Enter should be ignored while loading
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_input_ignored_when_not_visible() {
        let mut picker = LinkTypePicker::new();

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_empty_link_types() {
        let mut picker = LinkTypePicker::new();
        picker.show(vec![]);

        assert!(picker.is_visible());
        assert_eq!(picker.item_count(), 0);

        // Enter should do nothing with empty list
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_default_impl() {
        let picker = LinkTypePicker::default();
        assert!(!picker.is_visible());
    }
}
