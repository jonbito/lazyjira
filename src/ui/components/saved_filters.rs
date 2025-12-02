//! Saved filters dialog component.
//!
//! This module provides a popup dialog for managing saved filters.
//! Users can select, create, and delete saved filter configurations.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::api::types::{FilterState, SavedFilter};
use crate::ui::components::input::TextInput;
use crate::ui::theme::theme;

/// Mode for the saved filters dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SavedFiltersMode {
    /// Browsing/selecting existing filters.
    #[default]
    Browse,
    /// Entering a name for a new filter.
    CreateNew,
    /// Confirming deletion of a filter.
    ConfirmDelete,
}

/// Action returned from the saved filters dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SavedFiltersAction {
    /// User selected a saved filter to apply.
    Select(FilterState),
    /// User wants to save the current filter with a name.
    Save(String),
    /// User wants to delete a saved filter.
    Delete(String),
    /// User cancelled the dialog.
    Cancel,
}

/// A popup component for managing saved filters.
#[derive(Debug)]
pub struct SavedFiltersDialog {
    /// The list of saved filters.
    filters: Vec<SavedFilter>,
    /// Currently selected index.
    selected: usize,
    /// Whether the dialog is visible.
    visible: bool,
    /// List state for ratatui.
    list_state: ListState,
    /// Current mode.
    mode: SavedFiltersMode,
    /// Input for new filter name.
    name_input: TextInput,
    /// The current filter state to save (when creating new).
    current_filter: Option<FilterState>,
}

impl Default for SavedFiltersDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SavedFiltersDialog {
    /// Create a new saved filters dialog.
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            selected: 0,
            visible: false,
            list_state: ListState::default(),
            mode: SavedFiltersMode::Browse,
            name_input: TextInput::new(),
            current_filter: None,
        }
    }

    /// Show the dialog with the given saved filters.
    ///
    /// # Arguments
    ///
    /// * `filters` - List of saved filters to display
    /// * `current_filter` - The current filter state (for saving)
    pub fn show(&mut self, filters: Vec<SavedFilter>, current_filter: FilterState) {
        self.filters = filters;
        self.current_filter = Some(current_filter);
        self.selected = 0;
        self.list_state.select(if self.filters.is_empty() {
            None
        } else {
            Some(0)
        });
        self.mode = SavedFiltersMode::Browse;
        self.name_input.clear();
        self.visible = true;
    }

    /// Hide the dialog.
    pub fn hide(&mut self) {
        self.visible = false;
        self.mode = SavedFiltersMode::Browse;
        self.name_input.clear();
    }

    /// Check if the dialog is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the number of saved filters.
    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }

    /// Move selection down.
    fn move_down(&mut self) {
        if self.filters.is_empty() {
            return;
        }
        if self.selected < self.filters.len() - 1 {
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

    /// Start creating a new filter.
    fn start_create(&mut self) {
        self.mode = SavedFiltersMode::CreateNew;
        self.name_input.clear();
    }

    /// Start deletion confirmation.
    fn start_delete(&mut self) {
        if !self.filters.is_empty() {
            self.mode = SavedFiltersMode::ConfirmDelete;
        }
    }

    /// Cancel the current mode and go back to browse.
    fn cancel_mode(&mut self) {
        self.mode = SavedFiltersMode::Browse;
        self.name_input.clear();
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action when the user makes a selection or cancels.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<SavedFiltersAction> {
        match self.mode {
            SavedFiltersMode::Browse => self.handle_browse_input(key),
            SavedFiltersMode::CreateNew => self.handle_create_input(key),
            SavedFiltersMode::ConfirmDelete => self.handle_delete_input(key),
        }
    }

    /// Handle input in browse mode.
    fn handle_browse_input(&mut self, key: KeyEvent) -> Option<SavedFiltersAction> {
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
            // Select filter with Enter
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(filter) = self.filters.get(self.selected) {
                    self.visible = false;
                    Some(SavedFiltersAction::Select(filter.filter.clone()))
                } else {
                    None
                }
            }
            // Create new filter with 'n' or 's' (save)
            (KeyCode::Char('n'), KeyModifiers::NONE)
            | (KeyCode::Char('s'), KeyModifiers::NONE) => {
                self.start_create();
                None
            }
            // Delete filter with 'd'
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                self.start_delete();
                None
            }
            // Cancel with q or Esc
            (KeyCode::Esc, _) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.visible = false;
                Some(SavedFiltersAction::Cancel)
            }
            _ => None,
        }
    }

    /// Handle input in create new mode.
    fn handle_create_input(&mut self, key: KeyEvent) -> Option<SavedFiltersAction> {
        match key.code {
            KeyCode::Enter => {
                let value = self.name_input.value().trim().to_string();
                if !value.is_empty() {
                    self.visible = false;
                    self.mode = SavedFiltersMode::Browse;
                    self.name_input.clear();
                    Some(SavedFiltersAction::Save(value))
                } else {
                    None
                }
            }
            KeyCode::Esc => {
                self.cancel_mode();
                None
            }
            _ => {
                // Delegate to TextInput for character handling
                self.name_input.handle_input(key);
                None
            }
        }
    }

    /// Handle input in confirm delete mode.
    fn handle_delete_input(&mut self, key: KeyEvent) -> Option<SavedFiltersAction> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                if let Some(filter) = self.filters.get(self.selected) {
                    let name = filter.name.clone();
                    // Remove from local list
                    self.filters.remove(self.selected);
                    // Adjust selection
                    if self.selected >= self.filters.len() && self.selected > 0 {
                        self.selected -= 1;
                    }
                    self.list_state.select(if self.filters.is_empty() {
                        None
                    } else {
                        Some(self.selected)
                    });
                    self.mode = SavedFiltersMode::Browse;
                    Some(SavedFiltersAction::Delete(name))
                } else {
                    self.cancel_mode();
                    None
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.cancel_mode();
                None
            }
            _ => None,
        }
    }

    /// Render the saved filters dialog.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate dialog dimensions
        // Width needs to fit the longest hint text: "j/k:navigate  Enter:apply  n/s:save  d:delete  q/Esc:close"
        let dialog_width = 65u16.min(area.width.saturating_sub(4));
        let max_visible_items = 10u16;
        let item_count = self.filters.len() as u16;
        // Height calculation depends on mode
        let dialog_height = match self.mode {
            SavedFiltersMode::CreateNew => {
                // border (2) + margin (2) + input field (3) + spacer (1) + hint (1) = 9
                9u16
            }
            SavedFiltersMode::ConfirmDelete => {
                // border (2) + margin (2) + message (4) + hint (1) = 9
                9u16
            }
            SavedFiltersMode::Browse => {
                // border (2) + items + hint (2) + margin (1)
                let base = item_count.min(max_visible_items).max(1) * 2 + 5;
                base
            }
        }
        .min(area.height.saturating_sub(4));

        let dialog_area = centered_rect(area, dialog_width, dialog_height);

        // Clear the dialog area
        frame.render_widget(Clear, dialog_area);

        // Create the outer block
        let title = match self.mode {
            SavedFiltersMode::Browse => " Saved Filters ",
            SavedFiltersMode::CreateNew => " Save Current Filter ",
            SavedFiltersMode::ConfirmDelete => " Delete Filter ",
        };
        let block = Block::default()
            .title(Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        match self.mode {
            SavedFiltersMode::Browse => self.render_browse_mode(frame, inner_area),
            SavedFiltersMode::CreateNew => self.render_create_mode(frame, inner_area),
            SavedFiltersMode::ConfirmDelete => self.render_delete_mode(frame, inner_area),
        }
    }

    /// Render browse mode.
    fn render_browse_mode(&mut self, frame: &mut Frame, area: Rect) {
        // Split area for list and hint
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(area);

        // Render list or empty message
        if self.filters.is_empty() {
            let empty_msg = Paragraph::new(vec![Line::from(""), Line::from("No saved filters")])
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_msg, chunks[0]);
        } else {
            let items: Vec<ListItem> = self
                .filters
                .iter()
                .map(|f| {
                    let summary = f.filter.summary();
                    let summary_text = if summary.is_empty() {
                        "(empty filter)".to_string()
                    } else {
                        summary.join(", ")
                    };
                    // Truncate summary if too long
                    let max_len = (area.width as usize).saturating_sub(4);
                    let truncated = if summary_text.len() > max_len {
                        format!("{}...", &summary_text[..max_len.saturating_sub(3)])
                    } else {
                        summary_text
                    };
                    ListItem::new(vec![
                        Line::from(Span::styled(
                            &f.name,
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            truncated,
                            Style::default().fg(Color::DarkGray),
                        )),
                    ])
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

            frame.render_stateful_widget(list, chunks[0], &mut self.list_state);
        }

        // Render hint with colored keys
        let hint_line = if self.filters.is_empty() {
            Line::from(vec![
                Span::styled("n/s", Style::default().fg(Color::Yellow)),
                Span::raw(": save current  "),
                Span::styled("q/Esc", Style::default().fg(Color::Red)),
                Span::raw(": close"),
            ])
        } else {
            Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Yellow)),
                Span::raw(": navigate  "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(": apply  "),
                Span::styled("n/s", Style::default().fg(Color::Cyan)),
                Span::raw(": save  "),
                Span::styled("d", Style::default().fg(Color::Red)),
                Span::raw(": delete  "),
                Span::styled("q/Esc", Style::default().fg(Color::Red)),
                Span::raw(": close"),
            ])
        };
        let hint = Paragraph::new(hint_line).alignment(Alignment::Center);
        frame.render_widget(hint, chunks[1]);
    }

    /// Render create new filter mode.
    fn render_create_mode(&mut self, frame: &mut Frame, area: Rect) {
        let t = theme();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input with label
                Constraint::Min(1),    // Spacer
                Constraint::Length(1), // Hint
            ])
            .margin(1)
            .split(area);

        // Input field using TextInput component
        self.name_input
            .render_with_label(frame, chunks[0], "Filter name", true);

        // Hint with colored keys
        let hint_line = Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": save  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": cancel"),
        ]);
        let hint = Paragraph::new(hint_line).alignment(Alignment::Center);
        frame.render_widget(hint, chunks[2]);
    }

    /// Render delete confirmation mode.
    fn render_delete_mode(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Message
                Constraint::Length(1), // Hint
            ])
            .margin(1)
            .split(area);

        // Confirmation message
        let filter_name = self
            .filters
            .get(self.selected)
            .map(|f| f.name.as_str())
            .unwrap_or("?");
        let message = Paragraph::new(vec![
            Line::from(Span::styled(
                "Delete this saved filter?",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                filter_name,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(message, chunks[0]);

        // Hint with colored keys
        let hint_line = Line::from(vec![
            Span::styled("y", Style::default().fg(Color::Green)),
            Span::raw(": yes  "),
            Span::styled("n/Esc", Style::default().fg(Color::Red)),
            Span::raw(": cancel"),
        ]);
        let hint = Paragraph::new(hint_line).alignment(Alignment::Center);
        frame.render_widget(hint, chunks[1]);
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
    fn test_new_dialog() {
        let dialog = SavedFiltersDialog::new();
        assert!(!dialog.is_visible());
        assert_eq!(dialog.filter_count(), 0);
    }

    #[test]
    fn test_show_dialog() {
        let mut dialog = SavedFiltersDialog::new();
        let filters = vec![
            SavedFilter::new("Filter1", FilterState::default()),
            SavedFilter::new("Filter2", FilterState::default()),
        ];
        dialog.show(filters, FilterState::default());

        assert!(dialog.is_visible());
        assert_eq!(dialog.filter_count(), 2);
        assert_eq!(dialog.selected, 0);
    }

    #[test]
    fn test_hide_dialog() {
        let mut dialog = SavedFiltersDialog::new();
        dialog.show(vec![], FilterState::default());
        dialog.hide();

        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_navigation() {
        let mut dialog = SavedFiltersDialog::new();
        let filters = vec![
            SavedFilter::new("a", FilterState::default()),
            SavedFilter::new("b", FilterState::default()),
            SavedFilter::new("c", FilterState::default()),
        ];
        dialog.show(filters, FilterState::default());

        assert_eq!(dialog.selected, 0);

        // Move down
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        dialog.handle_input(key);
        assert_eq!(dialog.selected, 1);

        // Move down again
        dialog.handle_input(key);
        assert_eq!(dialog.selected, 2);

        // Should not go past the end
        dialog.handle_input(key);
        assert_eq!(dialog.selected, 2);

        // Move up
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        dialog.handle_input(key);
        assert_eq!(dialog.selected, 1);
    }

    #[test]
    fn test_select_filter() {
        let mut dialog = SavedFiltersDialog::new();
        let mut filter_state = FilterState::default();
        filter_state.statuses.push("Open".to_string());
        let filters = vec![SavedFilter::new("My Filter", filter_state.clone())];
        dialog.show(filters, FilterState::default());

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = dialog.handle_input(key);

        assert_eq!(action, Some(SavedFiltersAction::Select(filter_state)));
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_cancel() {
        let mut dialog = SavedFiltersDialog::new();
        dialog.show(vec![], FilterState::default());

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = dialog.handle_input(key);

        assert_eq!(action, Some(SavedFiltersAction::Cancel));
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_start_create_mode() {
        let mut dialog = SavedFiltersDialog::new();
        dialog.show(vec![], FilterState::default());

        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        dialog.handle_input(key);

        assert_eq!(dialog.mode, SavedFiltersMode::CreateNew);
    }

    #[test]
    fn test_create_filter() {
        let mut dialog = SavedFiltersDialog::new();
        dialog.show(vec![], FilterState::default());

        // Enter create mode
        dialog.handle_input(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));

        // Type name
        dialog.handle_input(KeyEvent::new(KeyCode::Char('T'), KeyModifiers::NONE));
        dialog.handle_input(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
        dialog.handle_input(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE));
        dialog.handle_input(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE));

        // Save
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(action, Some(SavedFiltersAction::Save("Test".to_string())));
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_cancel_create_mode() {
        let mut dialog = SavedFiltersDialog::new();
        dialog.show(vec![], FilterState::default());

        // Enter create mode
        dialog.handle_input(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));
        assert_eq!(dialog.mode, SavedFiltersMode::CreateNew);

        // Cancel
        dialog.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(dialog.mode, SavedFiltersMode::Browse);
        assert!(dialog.is_visible()); // Should still be visible
    }

    #[test]
    fn test_delete_filter() {
        let mut dialog = SavedFiltersDialog::new();
        let filters = vec![SavedFilter::new("ToDelete", FilterState::default())];
        dialog.show(filters, FilterState::default());

        // Enter delete mode
        dialog.handle_input(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
        assert_eq!(dialog.mode, SavedFiltersMode::ConfirmDelete);

        // Confirm
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));

        assert_eq!(
            action,
            Some(SavedFiltersAction::Delete("ToDelete".to_string()))
        );
        assert_eq!(dialog.mode, SavedFiltersMode::Browse);
    }

    #[test]
    fn test_cancel_delete() {
        let mut dialog = SavedFiltersDialog::new();
        let filters = vec![SavedFilter::new("ToDelete", FilterState::default())];
        dialog.show(filters, FilterState::default());

        // Enter delete mode
        dialog.handle_input(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));

        // Cancel
        dialog.handle_input(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));
        assert_eq!(dialog.mode, SavedFiltersMode::Browse);
        assert_eq!(dialog.filter_count(), 1); // Filter should still be there
    }

    #[test]
    fn test_vim_navigation() {
        let mut dialog = SavedFiltersDialog::new();
        let filters = vec![
            SavedFilter::new("a", FilterState::default()),
            SavedFilter::new("b", FilterState::default()),
        ];
        dialog.show(filters, FilterState::default());

        // Move down with j
        dialog.handle_input(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        assert_eq!(dialog.selected, 1);

        // Move up with k
        dialog.handle_input(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
        assert_eq!(dialog.selected, 0);
    }

    #[test]
    fn test_empty_name_not_saved() {
        let mut dialog = SavedFiltersDialog::new();
        dialog.show(vec![], FilterState::default());

        // Enter create mode
        dialog.handle_input(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));

        // Try to save empty name
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(action.is_none());
        assert!(dialog.is_visible()); // Should still be in create mode
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
}
