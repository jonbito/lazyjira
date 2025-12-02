//! Multi-select widget for filter options.
//!
//! Provides a scrollable list with checkboxes for selecting multiple items.

use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

/// A selectable item in the multi-select list.
#[derive(Debug, Clone)]
pub struct SelectItem {
    /// The unique identifier for this item.
    pub id: String,
    /// The display label for this item.
    pub label: String,
}

impl SelectItem {
    /// Create a new select item.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// A multi-select widget that allows selecting multiple items from a list.
pub struct MultiSelect {
    /// The list of items to display.
    items: Vec<SelectItem>,
    /// Set of selected item IDs.
    selected: HashSet<String>,
    /// Currently focused item index.
    cursor: usize,
    /// Scroll offset for long lists.
    scroll: usize,
    /// Widget title.
    title: String,
    /// List state for ratatui.
    list_state: ListState,
}

impl MultiSelect {
    /// Create a new multi-select widget.
    pub fn new(title: impl Into<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            items: Vec::new(),
            selected: HashSet::new(),
            cursor: 0,
            scroll: 0,
            title: title.into(),
            list_state,
        }
    }

    /// Set the items to display.
    pub fn set_items(&mut self, items: Vec<SelectItem>) {
        self.items = items;
        self.cursor = 0;
        self.scroll = 0;
        self.list_state.select(Some(0));
    }

    /// Set the selected item IDs.
    pub fn set_selected(&mut self, selected: HashSet<String>) {
        self.selected = selected;
    }

    /// Get the selected item IDs.
    pub fn selected(&self) -> &HashSet<String> {
        &self.selected
    }

    /// Get the selected item IDs as a vector.
    pub fn selected_ids(&self) -> Vec<String> {
        self.selected.iter().cloned().collect()
    }

    /// Check if an item is selected.
    pub fn is_selected(&self, id: &str) -> bool {
        self.selected.contains(id)
    }

    /// Toggle selection of the currently focused item.
    pub fn toggle_current(&mut self) {
        if let Some(item) = self.items.get(self.cursor) {
            let id = item.id.clone();
            if self.selected.contains(&id) {
                self.selected.remove(&id);
            } else {
                self.selected.insert(id);
            }
        }
    }

    /// Select all items.
    pub fn select_all(&mut self) {
        for item in &self.items {
            self.selected.insert(item.id.clone());
        }
    }

    /// Clear all selections.
    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    /// Move cursor up.
    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.list_state.select(Some(self.cursor));
            self.adjust_scroll();
        }
    }

    /// Move cursor down.
    pub fn move_down(&mut self) {
        if !self.items.is_empty() && self.cursor < self.items.len() - 1 {
            self.cursor += 1;
            self.list_state.select(Some(self.cursor));
            self.adjust_scroll();
        }
    }

    /// Move cursor to the first item.
    pub fn move_to_start(&mut self) {
        self.cursor = 0;
        self.scroll = 0;
        self.list_state.select(Some(0));
    }

    /// Move cursor to the last item.
    pub fn move_to_end(&mut self) {
        if !self.items.is_empty() {
            self.cursor = self.items.len() - 1;
            self.list_state.select(Some(self.cursor));
            self.adjust_scroll();
        }
    }

    /// Adjust scroll offset to keep cursor visible.
    fn adjust_scroll(&mut self) {
        // This is handled by ratatui's ListState
    }

    /// Get the number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of selected items.
    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    /// Handle keyboard input.
    ///
    /// Returns true if the input was handled.
    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        match (key.code, key.modifiers) {
            // Navigation with j/k or arrow keys
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                self.move_down();
                true
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                self.move_up();
                true
            }
            (KeyCode::Char(' '), KeyModifiers::NONE) => {
                self.toggle_current();
                true
            }
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                self.select_all();
                true
            }
            (KeyCode::Home, _) => {
                self.move_to_start();
                true
            }
            (KeyCode::End, _) => {
                self.move_to_end();
                true
            }
            _ => false,
        }
    }

    /// Render the multi-select widget.
    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Create title with selection count
        let title = if self.selected.is_empty() {
            self.title.clone()
        } else {
            format!("{} ({})", self.title, self.selected.len())
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        if self.items.is_empty() {
            // Show empty message
            let empty_item = ListItem::new(Line::from(Span::styled(
                "No options available",
                Style::default().fg(Color::DarkGray),
            )));
            let list = List::new(vec![empty_item]).block(block);
            frame.render_widget(list, area);
            return;
        }

        // Create list items with checkboxes
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| {
                let checkbox = if self.selected.contains(&item.id) {
                    "[x]"
                } else {
                    "[ ]"
                };
                let style = if self.selected.contains(&item.id) {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(checkbox, style),
                    Span::raw(" "),
                    Span::raw(&item.label),
                ]))
            })
            .collect();

        let list = if focused {
            // Only show highlight when focused
            let highlight_style = Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD);

            List::new(items)
                .block(block)
                .highlight_style(highlight_style)
                .highlight_symbol("> ")
        } else {
            // No highlight when not focused
            List::new(items).block(block)
        };

        if focused {
            frame.render_stateful_widget(list, area, &mut self.list_state);
        } else {
            frame.render_widget(list, area);
        }
    }
}

impl Default for MultiSelect {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_items() -> Vec<SelectItem> {
        vec![
            SelectItem::new("1", "Item One"),
            SelectItem::new("2", "Item Two"),
            SelectItem::new("3", "Item Three"),
        ]
    }

    #[test]
    fn test_new() {
        let ms = MultiSelect::new("Test");
        assert!(ms.items.is_empty());
        assert!(ms.selected.is_empty());
        assert_eq!(ms.cursor, 0);
    }

    #[test]
    fn test_set_items() {
        let mut ms = MultiSelect::new("Test");
        ms.set_items(create_test_items());
        assert_eq!(ms.len(), 3);
        assert!(!ms.is_empty());
    }

    #[test]
    fn test_toggle_current() {
        let mut ms = MultiSelect::new("Test");
        ms.set_items(create_test_items());

        assert!(!ms.is_selected("1"));
        ms.toggle_current();
        assert!(ms.is_selected("1"));
        ms.toggle_current();
        assert!(!ms.is_selected("1"));
    }

    #[test]
    fn test_navigation() {
        let mut ms = MultiSelect::new("Test");
        ms.set_items(create_test_items());

        assert_eq!(ms.cursor, 0);
        ms.move_down();
        assert_eq!(ms.cursor, 1);
        ms.move_down();
        assert_eq!(ms.cursor, 2);
        // Should not go past end
        ms.move_down();
        assert_eq!(ms.cursor, 2);

        ms.move_up();
        assert_eq!(ms.cursor, 1);
        ms.move_to_start();
        assert_eq!(ms.cursor, 0);
        ms.move_to_end();
        assert_eq!(ms.cursor, 2);
    }

    #[test]
    fn test_select_all() {
        let mut ms = MultiSelect::new("Test");
        ms.set_items(create_test_items());

        ms.select_all();
        assert_eq!(ms.selected_count(), 3);
        assert!(ms.is_selected("1"));
        assert!(ms.is_selected("2"));
        assert!(ms.is_selected("3"));
    }

    #[test]
    fn test_clear_selection() {
        let mut ms = MultiSelect::new("Test");
        ms.set_items(create_test_items());
        ms.select_all();
        assert_eq!(ms.selected_count(), 3);

        ms.clear_selection();
        assert_eq!(ms.selected_count(), 0);
    }

    #[test]
    fn test_selected_ids() {
        let mut ms = MultiSelect::new("Test");
        ms.set_items(create_test_items());
        ms.toggle_current();
        ms.move_down();
        ms.toggle_current();

        let ids = ms.selected_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"1".to_string()));
        assert!(ids.contains(&"2".to_string()));
    }

    #[test]
    fn test_handle_input() {
        let mut ms = MultiSelect::new("Test");
        ms.set_items(create_test_items());

        // Down arrow moves down
        let handled = ms.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert!(handled);
        assert_eq!(ms.cursor, 1);

        // Up arrow moves up
        let handled = ms.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert!(handled);
        assert_eq!(ms.cursor, 0);

        // space toggles
        let handled = ms.handle_input(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        assert!(handled);
        assert!(ms.is_selected("1"));

        // j/k are now handled for navigation
        let handled = ms.handle_input(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        assert!(handled);
        assert_eq!(ms.cursor, 1);

        let handled = ms.handle_input(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
        assert!(handled);
        assert_eq!(ms.cursor, 0);
    }

    #[test]
    fn test_empty_list_navigation() {
        let mut ms = MultiSelect::new("Test");
        // Should not panic on empty list
        ms.move_down();
        ms.move_up();
        ms.move_to_start();
        ms.move_to_end();
        ms.toggle_current();
    }
}
