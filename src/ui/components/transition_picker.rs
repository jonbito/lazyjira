//! Transition picker component for status changes.
//!
//! Displays available workflow transitions for an issue and allows
//! the user to select one to change the issue status.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::api::types::{FieldUpdates, Transition};

/// Action resulting from transition picker input.
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionAction {
    /// Execute a transition (transition ID, optional field updates).
    Execute(String, Option<FieldUpdates>),
    /// Transition requires fields - not yet supported, show message.
    RequiresFields(String),
    /// Cancel the transition picker.
    Cancel,
}

/// Transition picker component.
///
/// Shows available transitions for the current issue and allows
/// the user to select one using keyboard navigation.
#[derive(Debug)]
pub struct TransitionPicker {
    /// Available transitions.
    transitions: Vec<Transition>,
    /// Currently selected index.
    selected: usize,
    /// Whether the picker is visible.
    visible: bool,
    /// Whether transitions are loading.
    loading: bool,
    /// Current issue status name (for display).
    current_status: String,
}

impl TransitionPicker {
    /// Create a new transition picker.
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
            selected: 0,
            visible: false,
            loading: false,
            current_status: String::new(),
        }
    }

    /// Check if the picker is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if transitions are loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Set the loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Show the picker with loading state.
    pub fn show_loading(&mut self, current_status: &str) {
        self.current_status = current_status.to_string();
        self.transitions.clear();
        self.selected = 0;
        self.loading = true;
        self.visible = true;
    }

    /// Show the picker with the given transitions.
    pub fn show(&mut self, transitions: Vec<Transition>, current_status: &str) {
        self.current_status = current_status.to_string();
        self.transitions = transitions;
        self.selected = 0;
        self.loading = false;
        self.visible = true;
    }

    /// Hide the picker.
    pub fn hide(&mut self) {
        self.visible = false;
        self.loading = false;
    }

    /// Get the number of available transitions.
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }

    /// Get the currently selected transition.
    pub fn selected_transition(&self) -> Option<&Transition> {
        self.transitions.get(self.selected)
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the parent view.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<TransitionAction> {
        if !self.visible {
            return None;
        }

        // Don't handle input while loading
        if self.loading {
            if key.code == KeyCode::Esc {
                self.hide();
                return Some(TransitionAction::Cancel);
            }
            return None;
        }

        match (key.code, key.modifiers) {
            // Navigation down with j or arrow
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                if !self.transitions.is_empty() && self.selected < self.transitions.len() - 1 {
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
            // Select transition
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(transition) = self.transitions.get(self.selected) {
                    let transition_id = transition.id.clone();

                    // Check if transition has required fields
                    let has_required_fields = transition
                        .fields
                        .values()
                        .any(|f| f.required);

                    self.hide();

                    if has_required_fields {
                        // For now, we don't support required fields form
                        Some(TransitionAction::RequiresFields(transition_id))
                    } else {
                        Some(TransitionAction::Execute(transition_id, None))
                    }
                } else {
                    None
                }
            }
            // Cancel with q or Esc
            (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.hide();
                Some(TransitionAction::Cancel)
            }
            _ => None,
        }
    }

    /// Render the transition picker.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate dialog size and position (centered)
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 15.min(area.height.saturating_sub(4));

        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Clear the background
        frame.render_widget(Clear, dialog_area);

        // Create the dialog block
        let block = Block::default()
            .title(" Change Status ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Split inner area for current status and transitions list
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Current status
                Constraint::Min(3),    // Transitions list
                Constraint::Length(2), // Help text
            ])
            .split(inner);

        // Render current status
        let current_status_text = Line::from(vec![
            Span::styled("Current: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &self.current_status,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        let current_status_paragraph = Paragraph::new(current_status_text);
        frame.render_widget(current_status_paragraph, chunks[0]);

        // Render loading or transitions list
        if self.loading {
            let loading_text = Paragraph::new("Loading transitions...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading_text, chunks[1]);
        } else if self.transitions.is_empty() {
            let empty_text = Paragraph::new("No transitions available")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_text, chunks[1]);
        } else {
            // Build list items
            let items: Vec<ListItem> = self
                .transitions
                .iter()
                .map(|t| {
                    let style = status_category_style(&t.to);
                    let text = format!("{} -> {}", t.name, t.to.name);
                    ListItem::new(text).style(style)
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

impl Default for TransitionPicker {
    fn default() -> Self {
        Self::new()
    }
}

/// Get style based on status category.
fn status_category_style(target: &crate::api::types::TransitionTarget) -> Style {
    if let Some(category) = &target.status_category {
        match category.key.as_str() {
            "new" => Style::default().fg(Color::Blue),
            "indeterminate" => Style::default().fg(Color::Yellow),
            "done" => Style::default().fg(Color::Green),
            _ => Style::default(),
        }
    } else {
        Style::default()
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
    use crate::api::types::{StatusCategory, TransitionField, TransitionTarget};
    use std::collections::HashMap;

    fn create_test_transition(id: &str, name: &str, target_name: &str) -> Transition {
        Transition {
            id: id.to_string(),
            name: name.to_string(),
            to: TransitionTarget {
                id: "1".to_string(),
                name: target_name.to_string(),
                status_category: Some(StatusCategory {
                    id: 2,
                    key: "indeterminate".to_string(),
                    name: "In Progress".to_string(),
                    color_name: Some("yellow".to_string()),
                }),
            },
            fields: HashMap::new(),
        }
    }

    fn create_transition_with_required_field(id: &str, name: &str) -> Transition {
        let mut fields = HashMap::new();
        fields.insert(
            "resolution".to_string(),
            TransitionField {
                required: true,
                name: "Resolution".to_string(),
            },
        );

        Transition {
            id: id.to_string(),
            name: name.to_string(),
            to: TransitionTarget {
                id: "1".to_string(),
                name: "Resolved".to_string(),
                status_category: Some(StatusCategory {
                    id: 3,
                    key: "done".to_string(),
                    name: "Done".to_string(),
                    color_name: Some("green".to_string()),
                }),
            },
            fields,
        }
    }

    #[test]
    fn test_new_picker() {
        let picker = TransitionPicker::new();
        assert!(!picker.is_visible());
        assert!(!picker.is_loading());
        assert_eq!(picker.transition_count(), 0);
    }

    #[test]
    fn test_show_loading() {
        let mut picker = TransitionPicker::new();
        picker.show_loading("Open");

        assert!(picker.is_visible());
        assert!(picker.is_loading());
        assert_eq!(picker.current_status, "Open");
    }

    #[test]
    fn test_show_transitions() {
        let mut picker = TransitionPicker::new();
        let transitions = vec![
            create_test_transition("11", "Start Progress", "In Progress"),
            create_test_transition("21", "Done", "Done"),
        ];

        picker.show(transitions, "Open");

        assert!(picker.is_visible());
        assert!(!picker.is_loading());
        assert_eq!(picker.transition_count(), 2);
        assert_eq!(picker.current_status, "Open");
    }

    #[test]
    fn test_hide() {
        let mut picker = TransitionPicker::new();
        picker.show_loading("Open");
        assert!(picker.is_visible());

        picker.hide();
        assert!(!picker.is_visible());
        assert!(!picker.is_loading());
    }

    #[test]
    fn test_navigation_down() {
        let mut picker = TransitionPicker::new();
        let transitions = vec![
            create_test_transition("11", "Start Progress", "In Progress"),
            create_test_transition("21", "Done", "Done"),
        ];
        picker.show(transitions, "Open");

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
        let mut picker = TransitionPicker::new();
        let transitions = vec![
            create_test_transition("11", "Start Progress", "In Progress"),
            create_test_transition("21", "Done", "Done"),
        ];
        picker.show(transitions, "Open");
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
    fn test_select_transition() {
        let mut picker = TransitionPicker::new();
        let transitions = vec![
            create_test_transition("11", "Start Progress", "In Progress"),
            create_test_transition("21", "Done", "Done"),
        ];
        picker.show(transitions, "Open");

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(TransitionAction::Execute("11".to_string(), None)));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_select_transition_with_required_fields() {
        let mut picker = TransitionPicker::new();
        let transitions = vec![create_transition_with_required_field("31", "Resolve Issue")];
        picker.show(transitions, "In Progress");

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(
            action,
            Some(TransitionAction::RequiresFields("31".to_string()))
        );
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_cancel_with_esc() {
        let mut picker = TransitionPicker::new();
        picker.show(vec![create_test_transition("11", "Start", "In Progress")], "Open");

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(TransitionAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_cancel_with_q() {
        let mut picker = TransitionPicker::new();
        picker.show(vec![create_test_transition("11", "Start", "In Progress")], "Open");

        // 'q' should cancel (vim-style)
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(TransitionAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_navigation_with_j_k() {
        let mut picker = TransitionPicker::new();
        let transitions = vec![
            create_test_transition("11", "Start Progress", "In Progress"),
            create_test_transition("21", "Done", "Done"),
        ];
        picker.show(transitions, "Open");

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
        let mut picker = TransitionPicker::new();
        picker.show_loading("Open");

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(TransitionAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_input_ignored_while_loading() {
        let mut picker = TransitionPicker::new();
        picker.show_loading("Open");

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
        let mut picker = TransitionPicker::new();

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_selected_transition() {
        let mut picker = TransitionPicker::new();
        let transitions = vec![
            create_test_transition("11", "Start Progress", "In Progress"),
            create_test_transition("21", "Done", "Done"),
        ];
        picker.show(transitions, "Open");

        assert_eq!(picker.selected_transition().unwrap().id, "11");

        picker.selected = 1;
        assert_eq!(picker.selected_transition().unwrap().id, "21");
    }

    #[test]
    fn test_empty_transitions() {
        let mut picker = TransitionPicker::new();
        picker.show(vec![], "Open");

        assert!(picker.is_visible());
        assert_eq!(picker.transition_count(), 0);
        assert!(picker.selected_transition().is_none());

        // Enter should do nothing with empty list
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_default_impl() {
        let picker = TransitionPicker::default();
        assert!(!picker.is_visible());
    }
}
