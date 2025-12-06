//! Dropdown component for selecting from a list of options.
//!
//! This module provides a reusable dropdown widget that:
//! - Displays the currently selected value when collapsed
//! - Expands to show all options when Enter is pressed
//! - Supports j/k and arrow key navigation
//! - Allows selection with Enter and cancellation with Esc

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::ui::theme::theme;

/// A single item in a dropdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropdownItem {
    /// Unique identifier for the item.
    pub id: String,
    /// Display label for the item.
    pub label: String,
}

impl DropdownItem {
    /// Create a new dropdown item.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// Action resulting from dropdown input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropdownAction {
    /// An item was selected (id, label).
    Select(String, String),
    /// The dropdown was closed without selection.
    Cancel,
}

/// Dropdown component for selecting from a list of options.
///
/// The dropdown can be in two states:
/// - Collapsed: Shows the selected value (or placeholder)
/// - Expanded: Shows a scrollable list of all options
#[derive(Debug)]
pub struct Dropdown {
    /// Available items.
    items: Vec<DropdownItem>,
    /// Index of the currently selected item (None if nothing selected).
    selected: Option<usize>,
    /// Index of the highlighted item in the expanded list.
    highlighted: usize,
    /// Whether the dropdown is expanded.
    expanded: bool,
    /// Placeholder text when no item is selected.
    placeholder: String,
    /// Label for the dropdown field.
    label: String,
    /// Whether this field is required (shows * in label).
    required: bool,
}

impl Dropdown {
    /// Create a new dropdown with a label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            highlighted: 0,
            expanded: false,
            placeholder: "Select...".to_string(),
            label: label.into(),
            required: false,
        }
    }

    /// Set whether this field is required.
    pub fn set_required(&mut self, required: bool) {
        self.required = required;
    }

    /// Set the placeholder text.
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Set the available items.
    pub fn set_items(&mut self, items: Vec<DropdownItem>) {
        self.items = items;
        // Reset highlighted to start if items change
        self.highlighted = 0;
        // Clear selection if it's no longer valid
        if let Some(idx) = self.selected {
            if idx >= self.items.len() {
                self.selected = None;
            }
        }
    }

    /// Get the available items.
    pub fn items(&self) -> &[DropdownItem] {
        &self.items
    }

    /// Get the number of items.
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if the dropdown has any items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the currently selected item.
    pub fn selected_item(&self) -> Option<&DropdownItem> {
        self.selected.and_then(|idx| self.items.get(idx))
    }

    /// Get the selected index.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Select an item by index.
    pub fn select_index(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = Some(index);
            self.highlighted = index;
        }
    }

    /// Select an item by ID.
    pub fn select_by_id(&mut self, id: &str) {
        if let Some(idx) = self.items.iter().position(|item| item.id == id) {
            self.selected = Some(idx);
            self.highlighted = idx;
        }
    }

    /// Clear the selection.
    pub fn clear_selection(&mut self) {
        self.selected = None;
        self.highlighted = 0;
    }

    /// Check if the dropdown is expanded.
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Expand the dropdown.
    pub fn expand(&mut self) {
        // For optional dropdowns, we can expand even with items to show "None" option
        if !self.items.is_empty() || !self.required {
            self.expanded = true;
            // Start at the selected item if there is one
            // For optional dropdowns, index 0 is "None", so actual items start at 1
            if let Some(idx) = self.selected {
                self.highlighted = if self.required { idx } else { idx + 1 };
            } else {
                // If nothing selected, start at "None" (index 0) for optional, or first item for required
                self.highlighted = 0;
            }
        }
    }

    /// Get the total number of items in the expanded list (including "None" for optional).
    fn expanded_item_count(&self) -> usize {
        if self.required {
            self.items.len()
        } else {
            self.items.len() + 1 // +1 for "None" option
        }
    }

    /// Collapse the dropdown.
    pub fn collapse(&mut self) {
        self.expanded = false;
    }

    /// Reset the dropdown to initial state.
    pub fn reset(&mut self) {
        self.selected = None;
        self.highlighted = 0;
        self.expanded = false;
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the parent.
    /// When collapsed and focused, Enter expands the dropdown.
    /// When expanded, j/k or arrows navigate, Enter selects, Esc cancels.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<DropdownAction> {
        if self.expanded {
            self.handle_expanded_input(key)
        } else {
            self.handle_collapsed_input(key)
        }
    }

    /// Handle input when the dropdown is collapsed.
    fn handle_collapsed_input(&mut self, key: KeyEvent) -> Option<DropdownAction> {
        match (key.code, key.modifiers) {
            // Enter - expand the dropdown
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.expand();
                None
            }
            // Left/Right or h/l - cycle through options without expanding
            (KeyCode::Left, KeyModifiers::NONE) | (KeyCode::Char('h'), KeyModifiers::NONE) => {
                if !self.items.is_empty() {
                    let current = self.selected.unwrap_or(0);
                    if current > 0 {
                        self.selected = Some(current - 1);
                        self.highlighted = current - 1;
                        let item = &self.items[current - 1];
                        return Some(DropdownAction::Select(item.id.clone(), item.label.clone()));
                    }
                }
                None
            }
            (KeyCode::Right, KeyModifiers::NONE) | (KeyCode::Char('l'), KeyModifiers::NONE) => {
                if !self.items.is_empty() {
                    // If nothing selected, select the first item
                    if self.selected.is_none() {
                        self.selected = Some(0);
                        self.highlighted = 0;
                        let item = &self.items[0];
                        return Some(DropdownAction::Select(item.id.clone(), item.label.clone()));
                    }

                    let current = self.selected.unwrap();
                    if current < self.items.len() - 1 {
                        self.selected = Some(current + 1);
                        self.highlighted = current + 1;
                        let item = &self.items[current + 1];
                        return Some(DropdownAction::Select(item.id.clone(), item.label.clone()));
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Handle input when the dropdown is expanded.
    fn handle_expanded_input(&mut self, key: KeyEvent) -> Option<DropdownAction> {
        let item_count = self.expanded_item_count();

        match (key.code, key.modifiers) {
            // Navigation down with j or arrow
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                if item_count > 0 && self.highlighted < item_count - 1 {
                    self.highlighted += 1;
                }
                None
            }
            // Navigation up with k or arrow
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                if self.highlighted > 0 {
                    self.highlighted -= 1;
                }
                None
            }
            // Select with Enter
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.expanded = false;

                // For optional dropdowns, index 0 is the "None" option
                if !self.required && self.highlighted == 0 {
                    self.selected = None;
                    Some(DropdownAction::Select(
                        String::new(),
                        self.placeholder.clone(),
                    ))
                } else {
                    // Adjust index for optional dropdowns (items start at index 1)
                    let item_idx = if self.required {
                        self.highlighted
                    } else {
                        self.highlighted - 1
                    };

                    if let Some(item) = self.items.get(item_idx) {
                        self.selected = Some(item_idx);
                        Some(DropdownAction::Select(item.id.clone(), item.label.clone()))
                    } else {
                        None
                    }
                }
            }
            // Cancel with Esc or q
            (KeyCode::Esc, _) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.expanded = false;
                Some(DropdownAction::Cancel)
            }
            _ => None,
        }
    }

    /// Render the dropdown.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `area` - The area to render in
    /// * `focused` - Whether this dropdown is currently focused
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let t = theme();

        // Determine display text
        let display_text = if let Some(item) = self.selected_item() {
            item.label.clone()
        } else if self.items.is_empty() {
            "No options available".to_string()
        } else {
            self.placeholder.clone()
        };

        // Build the label with required indicator
        let label = if self.required {
            format!(" {} * ", self.label)
        } else {
            format!(" {} ", self.label)
        };

        // Determine styles
        let (text_style, border_style, title_style) = if focused {
            (
                Style::default().fg(t.accent),
                Style::default().fg(t.border_focused),
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
            )
        } else if self.selected.is_none() {
            (
                Style::default().fg(t.input_placeholder),
                Style::default().fg(t.border),
                Style::default().fg(t.fg),
            )
        } else {
            (
                Style::default().fg(t.input_fg),
                Style::default().fg(t.border),
                Style::default().fg(t.fg),
            )
        };

        // Add indicator that this is a dropdown
        let indicator = if self.expanded { "▲" } else { "▼" };
        let display_with_indicator = if self.items.is_empty() {
            display_text
        } else {
            format!("{} {}", display_text, indicator)
        };

        let block = Block::default()
            .title(Span::styled(label, title_style))
            .borders(Borders::ALL)
            .border_style(border_style);

        let paragraph = Paragraph::new(display_with_indicator)
            .style(text_style)
            .block(block);

        frame.render_widget(paragraph, area);
    }

    /// Render the expanded dropdown list as an overlay.
    ///
    /// This should be called after render() when the dropdown is expanded.
    /// It renders the list as a popup overlay below or above the dropdown.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `dropdown_area` - The area of the dropdown field (used to position the list)
    /// * `screen_area` - The full screen area (to determine if list should go above or below)
    pub fn render_expanded_list(&self, frame: &mut Frame, dropdown_area: Rect, screen_area: Rect) {
        let item_count = self.expanded_item_count();
        if !self.expanded || item_count == 0 {
            return;
        }

        let t = theme();

        // Calculate list dimensions
        let max_visible_items = 8;
        let list_height = (item_count.min(max_visible_items) + 2) as u16; // +2 for borders

        // Determine if list should go below or above the dropdown
        let space_below = screen_area
            .height
            .saturating_sub(dropdown_area.y + dropdown_area.height);
        let space_above = dropdown_area.y.saturating_sub(screen_area.y);

        let list_area = if space_below >= list_height || space_below >= space_above {
            // Position below the dropdown, overlapping by 1 to connect borders
            Rect::new(
                dropdown_area.x,
                dropdown_area.y + dropdown_area.height - 1,
                dropdown_area.width,
                list_height.min(space_below),
            )
        } else {
            // Position above the dropdown, overlapping by 1 to connect borders
            let y = dropdown_area.y.saturating_sub(list_height - 1);
            Rect::new(
                dropdown_area.x,
                y,
                dropdown_area.width,
                list_height.min(space_above),
            )
        };

        // Clear the background and fill with solid color
        frame.render_widget(Clear, list_area);

        // Create outer block with solid background
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.border_focused))
            .style(Style::default().bg(Color::Black));

        let inner_area = outer_block.inner(list_area);
        frame.render_widget(outer_block, list_area);

        // Build list items
        let mut items: Vec<ListItem> = Vec::with_capacity(item_count);

        // For optional dropdowns, add "None" option at the beginning
        if !self.required {
            let style = if self.selected.is_none() {
                // Currently selected - show with accent color
                Style::default()
                    .fg(t.accent)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                // Not selected - same style as other unselected items
                Style::default().fg(t.fg).bg(Color::Black)
            };
            items.push(ListItem::new(self.placeholder.clone()).style(style));
        }

        // Add actual items
        for (idx, item) in self.items.iter().enumerate() {
            let style = if Some(idx) == self.selected {
                Style::default()
                    .fg(t.accent)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.fg).bg(Color::Black)
            };
            items.push(ListItem::new(item.label.clone()).style(style));
        }

        // Render list inside the block
        let list = List::new(items)
            .style(Style::default().bg(Color::Black).fg(t.fg))
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .bg(t.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(Some(self.highlighted));

        frame.render_stateful_widget(list, inner_area, &mut state);
    }

    /// Get the label of this dropdown.
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Default for Dropdown {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_items() -> Vec<DropdownItem> {
        vec![
            DropdownItem::new("1", "Option 1"),
            DropdownItem::new("2", "Option 2"),
            DropdownItem::new("3", "Option 3"),
        ]
    }

    #[test]
    fn test_new_dropdown() {
        let dropdown = Dropdown::new("Test");
        assert!(!dropdown.is_expanded());
        assert!(dropdown.selected_item().is_none());
        assert!(dropdown.is_empty());
        assert_eq!(dropdown.label(), "Test");
    }

    #[test]
    fn test_set_items() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_items(create_test_items());

        assert_eq!(dropdown.item_count(), 3);
        assert!(!dropdown.is_empty());
    }

    #[test]
    fn test_select_by_index() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_items(create_test_items());

        dropdown.select_index(1);
        assert_eq!(dropdown.selected_index(), Some(1));
        assert_eq!(dropdown.selected_item().unwrap().label, "Option 2");
    }

    #[test]
    fn test_select_by_id() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_items(create_test_items());

        dropdown.select_by_id("3");
        assert_eq!(dropdown.selected_index(), Some(2));
        assert_eq!(dropdown.selected_item().unwrap().label, "Option 3");
    }

    #[test]
    fn test_expand_collapse() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_items(create_test_items());

        assert!(!dropdown.is_expanded());

        dropdown.expand();
        assert!(dropdown.is_expanded());

        dropdown.collapse();
        assert!(!dropdown.is_expanded());
    }

    #[test]
    fn test_expand_empty_dropdown() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_required(true); // Required dropdowns should not expand when empty
        dropdown.expand();
        // Should not expand when empty and required
        assert!(!dropdown.is_expanded());
    }

    #[test]
    fn test_expand_empty_optional_dropdown() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_required(false); // Optional dropdowns CAN expand to show "None" option
        dropdown.expand();
        // Should expand even when empty (to show "None" option)
        assert!(dropdown.is_expanded());
    }

    #[test]
    fn test_enter_expands_dropdown() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_items(create_test_items());

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = dropdown.handle_input(key);

        assert!(dropdown.is_expanded());
        assert!(action.is_none());
    }

    #[test]
    fn test_navigation_in_expanded() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_required(true); // Use required dropdown to test without "None" option
        dropdown.set_items(create_test_items());
        dropdown.expand();

        assert_eq!(dropdown.highlighted, 0);

        // Navigate down with j
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        dropdown.handle_input(key);
        assert_eq!(dropdown.highlighted, 1);

        // Navigate down with arrow
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        dropdown.handle_input(key);
        assert_eq!(dropdown.highlighted, 2);

        // Can't go past the end
        dropdown.handle_input(key);
        assert_eq!(dropdown.highlighted, 2);

        // Navigate up with k
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        dropdown.handle_input(key);
        assert_eq!(dropdown.highlighted, 1);

        // Navigate up with arrow
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        dropdown.handle_input(key);
        assert_eq!(dropdown.highlighted, 0);

        // Can't go past the start
        dropdown.handle_input(key);
        assert_eq!(dropdown.highlighted, 0);
    }

    #[test]
    fn test_select_with_enter() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_required(true); // Use required dropdown to test without "None" option
        dropdown.set_items(create_test_items());
        dropdown.expand();

        // Navigate to second option
        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        dropdown.handle_input(down);

        // Select
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = dropdown.handle_input(enter);

        assert_eq!(
            action,
            Some(DropdownAction::Select(
                "2".to_string(),
                "Option 2".to_string()
            ))
        );
        assert!(!dropdown.is_expanded());
        assert_eq!(dropdown.selected_index(), Some(1));
    }

    #[test]
    fn test_cancel_with_esc() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_items(create_test_items());
        dropdown.expand();

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = dropdown.handle_input(key);

        assert_eq!(action, Some(DropdownAction::Cancel));
        assert!(!dropdown.is_expanded());
    }

    #[test]
    fn test_cancel_with_q() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_items(create_test_items());
        dropdown.expand();

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = dropdown.handle_input(key);

        assert_eq!(action, Some(DropdownAction::Cancel));
        assert!(!dropdown.is_expanded());
    }

    #[test]
    fn test_left_right_navigation_collapsed() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_items(create_test_items());

        // Initially nothing selected, right should select first
        let right = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        let action = dropdown.handle_input(right);

        assert_eq!(
            action,
            Some(DropdownAction::Select(
                "1".to_string(),
                "Option 1".to_string()
            ))
        );
        assert_eq!(dropdown.selected_index(), Some(0));

        // Right again to select second
        let action = dropdown.handle_input(right);
        assert_eq!(
            action,
            Some(DropdownAction::Select(
                "2".to_string(),
                "Option 2".to_string()
            ))
        );
        assert_eq!(dropdown.selected_index(), Some(1));

        // Left to go back
        let left = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let action = dropdown.handle_input(left);
        assert_eq!(
            action,
            Some(DropdownAction::Select(
                "1".to_string(),
                "Option 1".to_string()
            ))
        );
        assert_eq!(dropdown.selected_index(), Some(0));

        // Left at start does nothing
        let action = dropdown.handle_input(left);
        assert!(action.is_none());
    }

    #[test]
    fn test_h_l_navigation_collapsed() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_items(create_test_items());
        dropdown.select_index(1);

        // Navigate with l
        let l = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
        let action = dropdown.handle_input(l);
        assert_eq!(
            action,
            Some(DropdownAction::Select(
                "3".to_string(),
                "Option 3".to_string()
            ))
        );

        // Navigate with h
        let h = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        let action = dropdown.handle_input(h);
        assert_eq!(
            action,
            Some(DropdownAction::Select(
                "2".to_string(),
                "Option 2".to_string()
            ))
        );
    }

    #[test]
    fn test_clear_selection() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_items(create_test_items());
        dropdown.select_index(1);

        assert!(dropdown.selected_item().is_some());

        dropdown.clear_selection();
        assert!(dropdown.selected_item().is_none());
    }

    #[test]
    fn test_reset() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_items(create_test_items());
        dropdown.select_index(2);
        dropdown.expand();

        dropdown.reset();

        assert!(dropdown.selected_item().is_none());
        assert!(!dropdown.is_expanded());
        assert_eq!(dropdown.highlighted, 0);
    }

    #[test]
    fn test_placeholder() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_placeholder("Choose an option");
        assert_eq!(dropdown.placeholder, "Choose an option");
    }

    #[test]
    fn test_required() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_required(true);
        assert!(dropdown.required);
    }

    #[test]
    fn test_expand_starts_at_selected() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_required(true); // Use required dropdown to test without "None" option offset
        dropdown.set_items(create_test_items());
        dropdown.select_index(2);

        dropdown.expand();

        assert_eq!(dropdown.highlighted, 2);
    }

    #[test]
    fn test_expand_starts_at_selected_optional() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_required(false); // Optional dropdown has "None" at index 0
        dropdown.set_items(create_test_items());
        dropdown.select_index(2); // Select "Option 3" (item index 2)

        dropdown.expand();

        // Highlighted should be 3 because "None" is at index 0, so items are offset by 1
        assert_eq!(dropdown.highlighted, 3);
    }

    #[test]
    fn test_select_none_in_optional_dropdown() {
        let mut dropdown = Dropdown::new("Test");
        dropdown.set_required(false);
        dropdown.set_placeholder("None");
        dropdown.set_items(create_test_items());
        dropdown.select_index(1); // Select something first
        dropdown.expand();

        // Highlighted starts at selected item (index 1 + 1 = 2 due to "None" offset)
        assert_eq!(dropdown.highlighted, 2);

        // Navigate up to "None" option
        let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        dropdown.handle_input(up);
        dropdown.handle_input(up);
        assert_eq!(dropdown.highlighted, 0);

        // Select "None"
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = dropdown.handle_input(enter);

        assert_eq!(
            action,
            Some(DropdownAction::Select(String::new(), "None".to_string()))
        );
        assert!(dropdown.selected_index().is_none());
    }

    #[test]
    fn test_default_impl() {
        let dropdown = Dropdown::default();
        assert!(dropdown.is_empty());
        assert!(dropdown.label().is_empty());
    }

    #[test]
    fn test_dropdown_item() {
        let item = DropdownItem::new("id1", "Label 1");
        assert_eq!(item.id, "id1");
        assert_eq!(item.label, "Label 1");
    }
}
