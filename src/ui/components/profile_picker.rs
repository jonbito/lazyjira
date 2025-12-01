//! Profile picker component.
//!
//! This module provides a popup dialog for selecting between configured profiles.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

/// Action returned from the profile picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfilePickerAction {
    /// User selected a profile.
    Select(String),
    /// User cancelled the selection.
    Cancel,
}

/// A popup component for selecting between configured profiles.
#[derive(Debug)]
pub struct ProfilePicker {
    /// The list of profile names.
    profiles: Vec<String>,
    /// Currently selected index.
    selected: usize,
    /// Whether the picker is visible.
    visible: bool,
    /// The name of the current active profile.
    current_profile: String,
    /// List state for ratatui.
    list_state: ListState,
}

impl Default for ProfilePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfilePicker {
    /// Create a new profile picker.
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            selected: 0,
            visible: false,
            current_profile: String::new(),
            list_state: ListState::default(),
        }
    }

    /// Show the picker with the given profiles.
    ///
    /// # Arguments
    ///
    /// * `profiles` - List of profile names to display
    /// * `current` - Name of the currently active profile
    pub fn show(&mut self, profiles: Vec<String>, current: &str) {
        self.profiles = profiles;
        self.current_profile = current.to_string();

        // Find the index of the current profile
        self.selected = self
            .profiles
            .iter()
            .position(|n| n == current)
            .unwrap_or(0);

        self.list_state.select(Some(self.selected));
        self.visible = true;
    }

    /// Hide the picker.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if the picker is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the currently selected profile name.
    pub fn selected_profile(&self) -> Option<&String> {
        self.profiles.get(self.selected)
    }

    /// Get the number of profiles.
    pub fn profile_count(&self) -> usize {
        self.profiles.len()
    }

    /// Move selection down.
    fn move_down(&mut self) {
        if self.profiles.is_empty() {
            return;
        }
        if self.selected < self.profiles.len() - 1 {
            self.selected += 1;
            self.list_state.select(Some(self.selected));
        }
    }

    /// Move selection up.
    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
        }
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action when the user makes a selection or cancels.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<ProfilePickerAction> {
        match (key.code, key.modifiers) {
            // Navigation with j/k or arrow keys
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                self.move_down();
                None
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                self.move_up();
                None
            }
            // Selection
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.visible = false;
                self.profiles
                    .get(self.selected)
                    .cloned()
                    .map(ProfilePickerAction::Select)
            }
            // Cancel with q or Esc
            (KeyCode::Esc, _) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.visible = false;
                Some(ProfilePickerAction::Cancel)
            }
            _ => None,
        }
    }

    /// Render the profile picker.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate dialog dimensions
        let dialog_width = 40u16.min(area.width.saturating_sub(4));
        let max_visible_items = 10u16;
        let item_count = self.profiles.len() as u16;
        // Height: title (1) + border (2) + items + hint (1) + margin (1)
        let dialog_height = (item_count.min(max_visible_items) + 5).min(area.height.saturating_sub(4));

        let dialog_area = centered_rect(area, dialog_width, dialog_height);

        // Clear the dialog area
        frame.render_widget(Clear, dialog_area);

        // Create the outer block
        let block = Block::default()
            .title(Span::styled(
                " Switch Profile ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Split inner area for list and hint
        let list_height = inner_area.height.saturating_sub(1);
        let list_area = Rect {
            x: inner_area.x,
            y: inner_area.y,
            width: inner_area.width,
            height: list_height,
        };
        let hint_area = Rect {
            x: inner_area.x,
            y: inner_area.y + list_height,
            width: inner_area.width,
            height: 1,
        };

        // Create list items
        let items: Vec<ListItem> = self
            .profiles
            .iter()
            .map(|name| {
                let is_current = name == &self.current_profile;
                let display = if is_current {
                    format!("{} (current)", name)
                } else {
                    name.clone()
                };
                let style = if is_current {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                };
                ListItem::new(Span::styled(display, style))
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

        frame.render_stateful_widget(list, list_area, &mut self.list_state);

        // Render hint
        let hint = ratatui::widgets::Paragraph::new(Span::styled(
            "j/k:navigate  Enter:select  q/Esc:cancel",
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(hint, hint_area);
    }
}

/// Calculate a centered rectangle within the given area.
fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_picker() {
        let picker = ProfilePicker::new();
        assert!(!picker.is_visible());
        assert_eq!(picker.profile_count(), 0);
    }

    #[test]
    fn test_show_picker() {
        let mut picker = ProfilePicker::new();
        let profiles = vec!["work".to_string(), "personal".to_string()];
        picker.show(profiles, "work");

        assert!(picker.is_visible());
        assert_eq!(picker.profile_count(), 2);
        assert_eq!(picker.selected, 0);
        assert_eq!(picker.selected_profile(), Some(&"work".to_string()));
    }

    #[test]
    fn test_show_selects_current_profile() {
        let mut picker = ProfilePicker::new();
        let profiles = vec![
            "work".to_string(),
            "personal".to_string(),
            "client".to_string(),
        ];
        picker.show(profiles, "personal");

        assert_eq!(picker.selected, 1);
        assert_eq!(picker.selected_profile(), Some(&"personal".to_string()));
    }

    #[test]
    fn test_hide_picker() {
        let mut picker = ProfilePicker::new();
        picker.show(vec!["work".to_string()], "work");
        picker.hide();

        assert!(!picker.is_visible());
    }

    #[test]
    fn test_navigation_down() {
        let mut picker = ProfilePicker::new();
        picker.show(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            "a",
        );

        assert_eq!(picker.selected, 0);

        // Move down with Down arrow
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 1);

        // Move down again
        picker.handle_input(key);
        assert_eq!(picker.selected, 2);

        // Should not go past the end
        picker.handle_input(key);
        assert_eq!(picker.selected, 2);
    }

    #[test]
    fn test_navigation_up() {
        let mut picker = ProfilePicker::new();
        picker.show(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            "c",
        );
        assert_eq!(picker.selected, 2);

        // Move up with Up arrow
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 1);

        // Move up again
        picker.handle_input(key);
        assert_eq!(picker.selected, 0);

        // Should not go below 0
        picker.handle_input(key);
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_select_with_enter() {
        let mut picker = ProfilePicker::new();
        picker.show(vec!["work".to_string(), "personal".to_string()], "work");

        // Navigate to second item with arrow key
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        picker.handle_input(key);

        // Press Enter
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(
            action,
            Some(ProfilePickerAction::Select("personal".to_string()))
        );
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_cancel_with_esc() {
        let mut picker = ProfilePicker::new();
        picker.show(vec!["work".to_string()], "work");

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(ProfilePickerAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_cancel_with_q() {
        let mut picker = ProfilePicker::new();
        picker.show(vec!["work".to_string()], "work");

        // 'q' should cancel (vim-style)
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(ProfilePickerAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_empty_profiles() {
        let mut picker = ProfilePicker::new();
        picker.show(vec![], "");

        assert!(picker.is_visible());
        assert_eq!(picker.profile_count(), 0);
        assert!(picker.selected_profile().is_none());

        // Navigation should not panic
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_navigation_with_j_k() {
        let mut picker = ProfilePicker::new();
        picker.show(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            "a",
        );

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
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = centered_rect(area, 40, 20);

        assert_eq!(centered.x, 30);
        assert_eq!(centered.y, 15);
        assert_eq!(centered.width, 40);
        assert_eq!(centered.height, 20);
    }

    #[test]
    fn test_centered_rect_larger_than_area() {
        let area = Rect::new(0, 0, 30, 20);
        let centered = centered_rect(area, 50, 30);

        assert_eq!(centered.width, 30);
        assert_eq!(centered.height, 20);
    }
}
