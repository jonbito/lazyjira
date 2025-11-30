//! Assignee picker component for user assignment.
//!
//! Displays assignable users for an issue and allows the user to select one
//! to change the assignee.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::api::types::User;

/// Action resulting from assignee picker input.
#[derive(Debug, Clone, PartialEq)]
pub enum AssigneeAction {
    /// Select a user as assignee (account_id, display_name).
    Select(String, String),
    /// Unassign (set to no assignee).
    Unassign,
    /// Cancel the picker.
    Cancel,
}

/// Assignee picker component.
///
/// Shows assignable users for the current issue and allows the user to select
/// one using keyboard navigation. Also provides option to unassign.
#[derive(Debug)]
pub struct AssigneePicker {
    /// Available users for assignment.
    users: Vec<User>,
    /// Currently selected index (0 = Unassigned, 1+ = users).
    selected: usize,
    /// Whether the picker is visible.
    visible: bool,
    /// Whether users are loading.
    loading: bool,
    /// Current assignee name (for display).
    current_assignee: String,
    /// Search/filter query.
    search_query: String,
    /// Filtered user indices (into users vec).
    filtered_indices: Vec<usize>,
}

impl AssigneePicker {
    /// Create a new assignee picker.
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            selected: 0,
            visible: false,
            loading: false,
            current_assignee: String::new(),
            search_query: String::new(),
            filtered_indices: Vec::new(),
        }
    }

    /// Check if the picker is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if users are loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Set the loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Show the picker with loading state.
    pub fn show_loading(&mut self, current_assignee: &str) {
        self.current_assignee = current_assignee.to_string();
        self.users.clear();
        self.selected = 0;
        self.search_query.clear();
        self.filtered_indices.clear();
        self.loading = true;
        self.visible = true;
    }

    /// Show the picker with the given users.
    pub fn show(&mut self, users: Vec<User>, current_assignee: &str) {
        self.current_assignee = current_assignee.to_string();
        self.users = users;
        self.selected = 0;
        self.search_query.clear();
        self.update_filtered_indices();
        self.loading = false;
        self.visible = true;
    }

    /// Hide the picker.
    pub fn hide(&mut self) {
        self.visible = false;
        self.loading = false;
        self.search_query.clear();
    }

    /// Get the number of available users.
    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    /// Update filtered indices based on search query.
    fn update_filtered_indices(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.users.len()).collect();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_indices = self
                .users
                .iter()
                .enumerate()
                .filter(|(_, u)| u.display_name.to_lowercase().contains(&query_lower))
                .map(|(i, _)| i)
                .collect();
        }
        // Reset selection when filter changes
        self.selected = 0;
    }

    /// Get total number of selectable items (Unassigned + filtered users).
    fn selectable_count(&self) -> usize {
        1 + self.filtered_indices.len() // 1 for "Unassigned" option
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the parent view.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<AssigneeAction> {
        if !self.visible {
            return None;
        }

        // Don't handle input while loading (except Esc)
        if self.loading {
            if key.code == KeyCode::Esc {
                self.hide();
                return Some(AssigneeAction::Cancel);
            }
            return None;
        }

        match (key.code, key.modifiers) {
            // Navigation down
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                if self.selected < self.selectable_count().saturating_sub(1) {
                    self.selected += 1;
                }
                None
            }
            // Navigation up
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            // Select
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.hide();
                if self.selected == 0 {
                    // "Unassigned" selected
                    Some(AssigneeAction::Unassign)
                } else {
                    // Get the actual user from filtered indices
                    let filtered_idx = self.selected - 1;
                    if let Some(&user_idx) = self.filtered_indices.get(filtered_idx) {
                        if let Some(user) = self.users.get(user_idx) {
                            return Some(AssigneeAction::Select(
                                user.account_id.clone(),
                                user.display_name.clone(),
                            ));
                        }
                    }
                    None
                }
            }
            // Cancel
            (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.hide();
                Some(AssigneeAction::Cancel)
            }
            // Search input (backspace)
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                if !self.search_query.is_empty() {
                    self.search_query.pop();
                    self.update_filtered_indices();
                }
                None
            }
            // Search input (character)
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) if c.is_alphabetic() || c.is_whitespace() => {
                self.search_query.push(c);
                self.update_filtered_indices();
                None
            }
            _ => None,
        }
    }

    /// Render the assignee picker.
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
            .title(" Change Assignee ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Split inner area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Current assignee
                Constraint::Length(2), // Search bar
                Constraint::Min(3),    // Users list
                Constraint::Length(2), // Help text
            ])
            .split(inner);

        // Render current assignee
        let current_text = if self.current_assignee.is_empty() {
            "Unassigned".to_string()
        } else {
            self.current_assignee.clone()
        };
        let current_line = Line::from(vec![
            Span::styled("Current: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                current_text,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(current_line), chunks[0]);

        // Render search bar
        let search_text = if self.search_query.is_empty() {
            "Type to filter...".to_string()
        } else {
            self.search_query.clone()
        };
        let search_style = if self.search_query.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };
        let search_line = Line::from(vec![
            Span::styled("🔍 ", Style::default().fg(Color::Cyan)),
            Span::styled(search_text, search_style),
        ]);
        frame.render_widget(Paragraph::new(search_line), chunks[1]);

        // Render loading or users list
        if self.loading {
            let loading_text = Paragraph::new("Loading assignable users...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading_text, chunks[2]);
        } else if self.users.is_empty() {
            let empty_text = Paragraph::new("No assignable users found")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_text, chunks[2]);
        } else {
            // Build list items: "Unassigned" first, then filtered users
            let mut items: Vec<ListItem> = Vec::with_capacity(1 + self.filtered_indices.len());

            // Add "Unassigned" option
            items.push(
                ListItem::new("  Unassigned")
                    .style(Style::default().fg(Color::DarkGray)),
            );

            // Add filtered users
            for &idx in &self.filtered_indices {
                if let Some(user) = self.users.get(idx) {
                    let text = format!("  {}", user.display_name);
                    items.push(ListItem::new(text));
                }
            }

            let list = List::new(items)
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            let mut state = ListState::default();
            state.select(Some(self.selected));

            frame.render_stateful_widget(list, chunks[2], &mut state);
        }

        // Render help text
        let help_text = Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(": navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": select  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": cancel"),
        ]);
        frame.render_widget(
            Paragraph::new(help_text).alignment(Alignment::Center),
            chunks[3],
        );
    }
}

impl Default for AssigneePicker {
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
    use crate::api::types::AvatarUrls;

    fn create_test_user(account_id: &str, display_name: &str) -> User {
        User {
            account_id: account_id.to_string(),
            display_name: display_name.to_string(),
            email_address: None,
            active: true,
            avatar_urls: None,
        }
    }

    #[test]
    fn test_new_picker() {
        let picker = AssigneePicker::new();
        assert!(!picker.is_visible());
        assert!(!picker.is_loading());
        assert_eq!(picker.user_count(), 0);
    }

    #[test]
    fn test_show_loading() {
        let mut picker = AssigneePicker::new();
        picker.show_loading("John Doe");

        assert!(picker.is_visible());
        assert!(picker.is_loading());
        assert_eq!(picker.current_assignee, "John Doe");
    }

    #[test]
    fn test_show_users() {
        let mut picker = AssigneePicker::new();
        let users = vec![
            create_test_user("user1", "Alice Smith"),
            create_test_user("user2", "Bob Jones"),
        ];

        picker.show(users, "Current User");

        assert!(picker.is_visible());
        assert!(!picker.is_loading());
        assert_eq!(picker.user_count(), 2);
        assert_eq!(picker.current_assignee, "Current User");
    }

    #[test]
    fn test_hide() {
        let mut picker = AssigneePicker::new();
        picker.show_loading("User");
        assert!(picker.is_visible());

        picker.hide();
        assert!(!picker.is_visible());
        assert!(!picker.is_loading());
    }

    #[test]
    fn test_navigation_down() {
        let mut picker = AssigneePicker::new();
        let users = vec![
            create_test_user("user1", "Alice"),
            create_test_user("user2", "Bob"),
        ];
        picker.show(users, "Current");

        // Initial selection is 0 (Unassigned)
        assert_eq!(picker.selected, 0);

        // Navigate down
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 1);

        // Navigate down again
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 2);

        // Can't go past the end (2 users + 1 unassigned = 3 items, max index = 2)
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 2);
    }

    #[test]
    fn test_navigation_up() {
        let mut picker = AssigneePicker::new();
        let users = vec![
            create_test_user("user1", "Alice"),
            create_test_user("user2", "Bob"),
        ];
        picker.show(users, "Current");
        picker.selected = 2;

        // Navigate up
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 1);

        // Navigate up again
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 0);

        // Can't go past the beginning
        let action = picker.handle_input(key);
        assert!(action.is_none());
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_select_unassigned() {
        let mut picker = AssigneePicker::new();
        let users = vec![create_test_user("user1", "Alice")];
        picker.show(users, "Current");

        // Select "Unassigned" (index 0)
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(AssigneeAction::Unassign));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_select_user() {
        let mut picker = AssigneePicker::new();
        let users = vec![
            create_test_user("user1", "Alice"),
            create_test_user("user2", "Bob"),
        ];
        picker.show(users, "Current");

        // Navigate to first user
        picker.selected = 1;

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(
            action,
            Some(AssigneeAction::Select(
                "user1".to_string(),
                "Alice".to_string()
            ))
        );
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_cancel_with_esc() {
        let mut picker = AssigneePicker::new();
        picker.show(vec![create_test_user("user1", "Alice")], "Current");

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(AssigneeAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_cancel_with_q() {
        let mut picker = AssigneePicker::new();
        picker.show(vec![create_test_user("user1", "Alice")], "Current");

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(AssigneeAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_esc_while_loading() {
        let mut picker = AssigneePicker::new();
        picker.show_loading("Current");

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_input(key);

        assert_eq!(action, Some(AssigneeAction::Cancel));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_input_ignored_while_loading() {
        let mut picker = AssigneePicker::new();
        picker.show_loading("Current");

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
        let mut picker = AssigneePicker::new();

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_input(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_search_filter() {
        let mut picker = AssigneePicker::new();
        let users = vec![
            create_test_user("user1", "Alice Smith"),
            create_test_user("user2", "Bob Jones"),
            create_test_user("user3", "Alice Jones"),
        ];
        picker.show(users, "Current");

        // Initially all 3 users visible + Unassigned = 4 items
        assert_eq!(picker.selectable_count(), 4);
        assert_eq!(picker.filtered_indices.len(), 3);

        // Type 'a' to filter
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        picker.handle_input(key);

        // Should now show only "Alice Smith" and "Alice Jones" + Unassigned
        assert_eq!(picker.filtered_indices.len(), 2);
        assert_eq!(picker.selectable_count(), 3);
        assert_eq!(picker.selected, 0); // Reset to 0 on filter

        // Type 'l' to filter to "alice" (still 2 matches)
        let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.filtered_indices.len(), 2);

        // Type 'ice s' to filter to "Alice S" -> "Alice Smith" only
        for c in ['i', 'c', 'e', ' ', 's'] {
            let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            picker.handle_input(key);
        }
        assert_eq!(picker.filtered_indices.len(), 1); // Only "Alice Smith"

        // Backspace to remove 's'
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        picker.handle_input(key);
        assert_eq!(picker.filtered_indices.len(), 2); // Back to both Alices
    }

    #[test]
    fn test_default_impl() {
        let picker = AssigneePicker::default();
        assert!(!picker.is_visible());
    }
}
