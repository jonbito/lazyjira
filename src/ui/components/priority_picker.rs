//! Priority picker component for priority changes.
//!
//! Displays available priorities for an issue and allows the user to select
//! one to change the priority.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::api::types::Priority;

/// Action resulting from priority picker input.
#[derive(Debug, Clone, PartialEq)]
pub enum PriorityAction {
    /// Select a priority (id, name).
    Select(String, String),
    /// Cancel the picker.
    Cancel,
}

/// Priority picker component.
///
/// Shows available priorities and allows the user to select one using keyboard
/// navigation.
#[derive(Debug)]
pub struct PriorityPicker {
    /// Available priorities.
    priorities: Vec<Priority>,
    /// Currently selected index.
    selected: usize,
    /// Whether the picker is visible.
    visible: bool,
    /// Whether priorities are loading.
    loading: bool,
    /// Current priority name (for display).
    current_priority: String,
}

impl PriorityPicker {
    /// Create a new priority picker.
    pub fn new() -> Self {
        Self {
            priorities: Vec::new(),
            selected: 0,
            visible: false,
            loading: false,
            current_priority: String::new(),
        }
    }

    /// Check if the picker is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if priorities are loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Set the loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Show the picker with loading state.
    pub fn show_loading(&mut self, current_priority: &str) {
        self.current_priority = current_priority.to_string();
        self.priorities.clear();
        self.selected = 0;
        self.loading = true;
        self.visible = true;
    }

    /// Show the picker with the given priorities.
    pub fn show(&mut self, priorities: Vec<Priority>, current_priority: &str) {
        self.current_priority = current_priority.to_string();
        // Try to pre-select the current priority
        let preselect = priorities
            .iter()
            .position(|p| p.name == current_priority)
            .unwrap_or(0);
        self.priorities = priorities;
        self.selected = preselect;
        self.loading = false;
        self.visible = true;
    }

    /// Hide the picker.
    pub fn hide(&mut self) {
        self.visible = false;
        self.loading = false;
    }

    /// Get the number of available priorities.
    pub fn priority_count(&self) -> usize {
        self.priorities.len()
    }

    /// Get the currently selected priority.
    pub fn selected_priority(&self) -> Option<&Priority> {
        self.priorities.get(self.selected)
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the parent view.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<PriorityAction> {
        if !self.visible {
            return None;
        }

        // Don't handle input while loading (except Esc)
        if self.loading {
            if key.code == KeyCode::Esc {
                self.hide();
                return Some(PriorityAction::Cancel);
            }
            return None;
        }

        match (key.code, key.modifiers) {
            // Navigation down with j or arrow
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                if !self.priorities.is_empty() && self.selected < self.priorities.len() - 1 {
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
            // Select priority
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(priority) = self.priorities.get(self.selected) {
                    let action = PriorityAction::Select(priority.id.clone(), priority.name.clone());
                    self.hide();
                    Some(action)
                } else {
                    None
                }
            }
            // Cancel with q or Esc
            (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.hide();
                Some(PriorityAction::Cancel)
            }
            _ => None,
        }
    }

    /// Render the priority picker.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate dialog size and position (centered)
        let dialog_width = 40.min(area.width.saturating_sub(4));
        let dialog_height = 14.min(area.height.saturating_sub(4));

        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Clear the background
        frame.render_widget(Clear, dialog_area);

        // Create the dialog block
        let block = Block::default()
            .title(" Change Priority ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Split inner area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Current priority
                Constraint::Min(3),    // Priorities list
                Constraint::Length(2), // Help text
            ])
            .split(inner);

        // Render current priority
        let current_text = if self.current_priority.is_empty() {
            "None".to_string()
        } else {
            self.current_priority.clone()
        };
        let current_line = Line::from(vec![
            Span::styled("Current: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                current_text,
                Style::default()
                    .fg(priority_color(&self.current_priority))
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(current_line), chunks[0]);

        // Render loading or priorities list
        if self.loading {
            let loading_text = Paragraph::new("Loading priorities...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading_text, chunks[1]);
        } else if self.priorities.is_empty() {
            let empty_text = Paragraph::new("No priorities available")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_text, chunks[1]);
        } else {
            // Build list items
            let items: Vec<ListItem> = self
                .priorities
                .iter()
                .map(|p| {
                    let color = priority_color(&p.name);
                    let indicator = priority_indicator(&p.name);
                    let text = format!("{} {}", indicator, p.name);
                    ListItem::new(text).style(Style::default().fg(color))
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
        frame.render_widget(
            Paragraph::new(help_text).alignment(Alignment::Center),
            chunks[2],
        );
    }
}

impl Default for PriorityPicker {
    fn default() -> Self {
        Self::new()
    }
}

/// Get color for a priority name.
fn priority_color(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "highest" | "blocker" | "critical" => Color::Red,
        "high" | "major" => Color::LightRed,
        "medium" | "normal" => Color::Yellow,
        "low" | "minor" => Color::Green,
        "lowest" | "trivial" => Color::DarkGray,
        _ => Color::White,
    }
}

/// Get indicator symbol for a priority name.
fn priority_indicator(name: &str) -> &'static str {
    match name.to_lowercase().as_str() {
        "highest" | "blocker" | "critical" => "⬆⬆",
        "high" | "major" => "⬆",
        "medium" | "normal" => "⬌",
        "low" | "minor" => "⬇",
        "lowest" | "trivial" => "⬇⬇",
        _ => "•",
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

    fn create_test_priority(id: &str, name: &str) -> Priority {
        Priority {
            id: id.to_string(),
            name: name.to_string(),
            icon_url: None,
        }
    }

    fn create_standard_priorities() -> Vec<Priority> {
        vec![
            create_test_priority("1", "Highest"),
            create_test_priority("2", "High"),
            create_test_priority("3", "Medium"),
            create_test_priority("4", "Low"),
            create_test_priority("5", "Lowest"),
        ]
    }

    #[test]
    fn test_new_picker() {
        let picker = PriorityPicker::new();
        assert!(!picker.is_visible());
        assert!(!picker.is_loading());
        assert_eq!(picker.priority_count(), 0);
    }

    #[test]
    fn test_show_loading() {
        let mut picker = PriorityPicker::new();
        picker.show_loading("High");

        assert!(picker.is_visible());
        assert!(picker.is_loading());
        assert_eq!(picker.current_priority, "High");
    }

    #[test]
    fn test_show_priorities() {
        let mut picker = PriorityPicker::new();
        let priorities = create_standard_priorities();

        picker.show(priorities, "High");

        assert!(picker.is_visible());
        assert!(!picker.is_loading());
        assert_eq!(picker.priority_count(), 5);
        // Should pre-select "High" which is index 1
        assert_eq!(picker.selected, 1);
    }

    #[test]
    fn test_show_priorities_no_match() {
        let mut picker = PriorityPicker::new();
        let priorities = create_standard_priorities();

        picker.show(priorities, "Unknown");

        // Should default to index 0 when no match
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_hide() {
        let mut picker = PriorityPicker::new();
        picker.show_loading("High");
        assert!(picker.is_visible());

        picker.hide();
        assert!(!picker.is_visible());
        assert!(!picker.is_loading());
    }

    #[test]
    fn test_navigation_down() {
        let mut picker = PriorityPicker::new();
        picker.show(create_standard_priorities(), "Highest");

        // Initial selection is 0 (Highest)
        assert_eq!(picker.selected, 0);

        // Navigate down with arrow key
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 1);

        // Navigate down more times
        for _ in 0..3 {
            picker.handle_input(key);
        }
        assert_eq!(picker.selected, 4);

        // Can't go past the end
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 4);
    }

    #[test]
    fn test_navigation_up() {
        let mut picker = PriorityPicker::new();
        picker.show(create_standard_priorities(), "Lowest");
        // Preselected to Lowest (index 4)
        assert_eq!(picker.selected, 4);

        // Navigate up with arrow key
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 3);

        // Navigate up to beginning
        for _ in 0..3 {
            picker.handle_input(key);
        }
        assert_eq!(picker.selected, 0);

        // Can't go past the beginning
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_select_priority() {
        let mut picker = PriorityPicker::new();
        picker.show(create_standard_priorities(), "Highest");

        // Navigate to "High" with arrow key
        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        picker.handle_input(down);
        assert_eq!(picker.selected, 1);

        // Select
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(enter);

        assert_eq!(
            action,
            Some(PriorityAction::Select("2".to_string(), "High".to_string()))
        );
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_cancel_with_esc() {
        let mut picker = PriorityPicker::new();
        picker.show(create_standard_priorities(), "High");

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(PriorityAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_cancel_with_q() {
        let mut picker = PriorityPicker::new();
        picker.show(create_standard_priorities(), "High");

        // 'q' should cancel (vim-style)
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(PriorityAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_navigation_with_j_k() {
        let mut picker = PriorityPicker::new();
        picker.show(create_standard_priorities(), "Highest");

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
        let mut picker = PriorityPicker::new();
        picker.show_loading("High");

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(PriorityAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_input_ignored_while_loading() {
        let mut picker = PriorityPicker::new();
        picker.show_loading("High");

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
        let mut picker = PriorityPicker::new();

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_selected_priority() {
        let mut picker = PriorityPicker::new();
        picker.show(create_standard_priorities(), "Medium");

        // Preselected to Medium (index 2)
        assert_eq!(picker.selected_priority().unwrap().name, "Medium");

        picker.selected = 0;
        assert_eq!(picker.selected_priority().unwrap().name, "Highest");
    }

    #[test]
    fn test_empty_priorities() {
        let mut picker = PriorityPicker::new();
        picker.show(vec![], "High");

        assert!(picker.is_visible());
        assert_eq!(picker.priority_count(), 0);
        assert!(picker.selected_priority().is_none());

        // Enter should do nothing with empty list
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_default_impl() {
        let picker = PriorityPicker::default();
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_priority_color() {
        assert_eq!(priority_color("Highest"), Color::Red);
        assert_eq!(priority_color("HIGHEST"), Color::Red);
        assert_eq!(priority_color("Critical"), Color::Red);
        assert_eq!(priority_color("High"), Color::LightRed);
        assert_eq!(priority_color("Medium"), Color::Yellow);
        assert_eq!(priority_color("Low"), Color::Green);
        assert_eq!(priority_color("Lowest"), Color::DarkGray);
        assert_eq!(priority_color("Unknown"), Color::White);
    }

    #[test]
    fn test_priority_indicator() {
        assert_eq!(priority_indicator("Highest"), "⬆⬆");
        assert_eq!(priority_indicator("High"), "⬆");
        assert_eq!(priority_indicator("Medium"), "⬌");
        assert_eq!(priority_indicator("Low"), "⬇");
        assert_eq!(priority_indicator("Lowest"), "⬇⬇");
        assert_eq!(priority_indicator("Unknown"), "•");
    }
}
