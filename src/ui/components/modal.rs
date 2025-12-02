//! Modal dialog components.
//!
//! This module provides modal dialogs for displaying information that
//! requires user acknowledgment, such as error messages or confirmations.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::error::AppError;

/// A modal dialog widget.
pub struct Modal {
    /// The modal title.
    pub title: String,
    /// Whether the modal is visible.
    pub visible: bool,
}

impl Modal {
    /// Create a new modal with the given title.
    pub fn new(title: String) -> Self {
        Self {
            title,
            visible: false,
        }
    }
}

/// An error dialog for displaying critical errors.
///
/// Error dialogs require user acknowledgment before they can be dismissed.
#[derive(Debug, Default)]
pub struct ErrorDialog {
    /// The dialog title.
    title: String,
    /// The main error message.
    message: String,
    /// Optional additional details.
    details: Option<String>,
    /// Optional suggested action.
    suggestion: Option<String>,
    /// Whether the dialog is visible.
    visible: bool,
}

impl ErrorDialog {
    /// Create a new error dialog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the dialog with an error.
    pub fn show(&mut self, error: &AppError) {
        self.title = "Error".to_string();
        self.message = error.user_message();
        self.details = Some(format!("{:?}", error));
        self.suggestion = error.suggested_action().map(String::from);
        self.visible = true;
    }

    /// Show the dialog with a custom message.
    pub fn show_message(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.title = title.into();
        self.message = message.into();
        self.details = None;
        self.suggestion = None;
        self.visible = true;
    }

    /// Show the dialog with a message and suggestion.
    pub fn show_with_suggestion(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) {
        self.title = title.into();
        self.message = message.into();
        self.details = None;
        self.suggestion = Some(suggestion.into());
        self.visible = true;
    }

    /// Hide the dialog.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Dismiss the dialog (alias for hide).
    pub fn dismiss(&mut self) {
        self.hide();
    }

    /// Check if the dialog is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Render the error dialog.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate dialog size (60% width, 40% height, but with min/max)
        let dialog_width = (area.width * 60 / 100).max(40).min(80);
        let dialog_height = (area.height * 40 / 100).max(8).min(20);

        let dialog_area = centered_rect(area, dialog_width, dialog_height);

        // Clear the dialog area
        frame.render_widget(Clear, dialog_area);

        // Create the outer block with error styling
        let block = Block::default()
            .title(Span::styled(
                format!(" {} ", self.title),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Create layout for content
        let mut constraints = vec![
            Constraint::Min(3), // Message
        ];
        if self.suggestion.is_some() {
            constraints.push(Constraint::Length(2)); // Suggestion
        }
        constraints.push(Constraint::Length(1)); // Dismiss hint

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(constraints)
            .split(inner_area);

        // Render main message
        let message = Paragraph::new(self.message.as_str())
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);
        frame.render_widget(message, chunks[0]);

        // Render suggestion if present
        let mut hint_index = 1;
        if let Some(ref suggestion) = self.suggestion {
            let suggestion_text = Line::from(vec![
                Span::styled("→ ", Style::default().fg(Color::Yellow)),
                Span::styled(suggestion, Style::default().fg(Color::Yellow)),
            ]);
            let suggestion_paragraph = Paragraph::new(suggestion_text)
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Left);
            frame.render_widget(suggestion_paragraph, chunks[1]);
            hint_index = 2;
        }

        // Render dismiss hint
        let hint = Paragraph::new("Press Esc or Enter to dismiss")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[hint_index]);
    }
}

/// A confirmation dialog for user confirmations.
///
/// Supports both normal and destructive action styling, with customizable
/// button labels and keyboard shortcuts (y/n or Enter/Esc).
#[derive(Debug, Default)]
pub struct ConfirmDialog {
    /// The dialog title.
    title: String,
    /// The confirmation message.
    message: String,
    /// Label for the confirm button.
    confirm_label: String,
    /// Label for the cancel button.
    cancel_label: String,
    /// Whether this is a destructive action (uses red styling).
    destructive: bool,
    /// Whether the dialog is visible.
    visible: bool,
    /// Current selection (true = confirm, false = cancel).
    selected_confirm: bool,
}

impl ConfirmDialog {
    /// Create a new confirmation dialog.
    pub fn new() -> Self {
        Self {
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            ..Self::default()
        }
    }

    /// Show the dialog with a message.
    pub fn show(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.title = title.into();
        self.message = message.into();
        self.confirm_label = "Confirm".to_string();
        self.cancel_label = "Cancel".to_string();
        self.destructive = false;
        self.visible = true;
        self.selected_confirm = false; // Default to cancel for safety
    }

    /// Show the dialog with a destructive action (red styling).
    pub fn show_destructive(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.title = title.into();
        self.message = message.into();
        self.confirm_label = "Delete".to_string();
        self.cancel_label = "Cancel".to_string();
        self.destructive = true;
        self.visible = true;
        self.selected_confirm = false; // Default to cancel for safety
    }

    /// Show the dialog with custom button labels.
    pub fn show_with_labels(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        confirm_label: impl Into<String>,
        cancel_label: impl Into<String>,
    ) {
        self.title = title.into();
        self.message = message.into();
        self.confirm_label = confirm_label.into();
        self.cancel_label = cancel_label.into();
        self.destructive = false;
        self.visible = true;
        self.selected_confirm = false;
    }

    /// Show a destructive dialog with custom confirm label.
    pub fn show_destructive_with_label(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        confirm_label: impl Into<String>,
    ) {
        self.title = title.into();
        self.message = message.into();
        self.confirm_label = confirm_label.into();
        self.cancel_label = "Cancel".to_string();
        self.destructive = true;
        self.visible = true;
        self.selected_confirm = false;
    }

    /// Hide the dialog.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if the dialog is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if the dialog is for a destructive action.
    pub fn is_destructive(&self) -> bool {
        self.destructive
    }

    /// Toggle the selection.
    pub fn toggle_selection(&mut self) {
        self.selected_confirm = !self.selected_confirm;
    }

    /// Select confirm.
    pub fn select_confirm(&mut self) {
        self.selected_confirm = true;
    }

    /// Select cancel.
    pub fn select_cancel(&mut self) {
        self.selected_confirm = false;
    }

    /// Check if confirm is selected.
    pub fn is_confirm_selected(&self) -> bool {
        self.selected_confirm
    }

    /// Confirm and hide the dialog, returning true.
    pub fn confirm(&mut self) -> bool {
        self.hide();
        true
    }

    /// Cancel and hide the dialog, returning false.
    pub fn cancel(&mut self) -> bool {
        self.hide();
        false
    }

    /// Handle keyboard input.
    ///
    /// Returns Some(true) if confirmed, Some(false) if cancelled, None if no action.
    pub fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> Option<bool> {
        use crossterm::event::KeyCode;

        match key.code {
            // Enter confirms the current selection
            KeyCode::Enter => {
                self.hide();
                Some(self.selected_confirm)
            }
            // Escape cancels (use Esc only, not 'q', to allow typing 'q' in text inputs)
            KeyCode::Esc => {
                self.hide();
                Some(false)
            }
            // Tab or arrow keys toggle selection (no h/l to allow typing those chars)
            KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
                self.toggle_selection();
                None
            }
            // Y/N shortcuts (only in dialogs without text input)
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.hide();
                Some(true)
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.hide();
                Some(false)
            }
            _ => None,
        }
    }

    /// Render the confirmation dialog.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let dialog_width = (area.width * 50 / 100).max(40).min(60);
        let dialog_height = 8;

        let dialog_area = centered_rect(area, dialog_width, dialog_height);

        // Clear the dialog area
        frame.render_widget(Clear, dialog_area);

        // Use red border for destructive actions, yellow for normal
        let border_color = if self.destructive {
            Color::Red
        } else {
            Color::Yellow
        };

        // Create the outer block
        let block = Block::default()
            .title(Span::styled(
                format!(" {} ", self.title),
                Style::default()
                    .fg(border_color)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Create layout for content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(2),    // Message
                Constraint::Length(1), // Buttons
            ])
            .split(inner_area);

        // Render message
        let message = Paragraph::new(self.message.as_str())
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        frame.render_widget(message, chunks[0]);

        // Determine button colors based on destructive mode
        let (confirm_base_color, cancel_base_color) = if self.destructive {
            // Destructive: confirm is red (danger), cancel is green (safe)
            (Color::Red, Color::Green)
        } else {
            // Normal: confirm is green (positive), cancel is gray
            (Color::Green, Color::Gray)
        };

        // Render buttons with selection highlighting
        // Selected button uses bold + underline instead of background color for readability
        let confirm_style = if self.selected_confirm {
            Style::default()
                .fg(confirm_base_color)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(confirm_base_color)
        };

        let cancel_style = if !self.selected_confirm {
            Style::default()
                .fg(cancel_base_color)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(cancel_base_color)
        };

        let buttons = Line::from(vec![
            Span::styled(format!(" [Y] {} ", self.confirm_label), confirm_style),
            Span::raw("  "),
            Span::styled(format!(" [N] {} ", self.cancel_label), cancel_style),
        ]);

        let buttons_paragraph = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(buttons_paragraph, chunks[1]);
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
    fn test_modal_new() {
        let modal = Modal::new("Test".to_string());
        assert_eq!(modal.title, "Test");
        assert!(!modal.visible);
    }

    #[test]
    fn test_error_dialog_new() {
        let dialog = ErrorDialog::new();
        assert!(!dialog.is_visible());
        assert!(dialog.message().is_empty());
    }

    #[test]
    fn test_error_dialog_show_message() {
        let mut dialog = ErrorDialog::new();
        dialog.show_message("Error", "Something went wrong");
        assert!(dialog.is_visible());
        assert_eq!(dialog.message(), "Something went wrong");
    }

    #[test]
    fn test_error_dialog_show_with_suggestion() {
        let mut dialog = ErrorDialog::new();
        dialog.show_with_suggestion("Error", "Auth failed", "Check your token");
        assert!(dialog.is_visible());
        assert!(dialog.suggestion.is_some());
    }

    #[test]
    fn test_error_dialog_hide() {
        let mut dialog = ErrorDialog::new();
        dialog.show_message("Error", "Test");
        dialog.hide();
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_error_dialog_dismiss() {
        let mut dialog = ErrorDialog::new();
        dialog.show_message("Error", "Test");
        dialog.dismiss();
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_confirm_dialog_new() {
        let dialog = ConfirmDialog::new();
        assert!(!dialog.is_visible());
        assert!(!dialog.is_confirm_selected());
    }

    #[test]
    fn test_confirm_dialog_show() {
        let mut dialog = ConfirmDialog::new();
        dialog.show("Confirm", "Are you sure?");
        assert!(dialog.is_visible());
        assert!(!dialog.is_confirm_selected()); // Default to cancel
    }

    #[test]
    fn test_confirm_dialog_toggle() {
        let mut dialog = ConfirmDialog::new();
        dialog.show("Confirm", "Test");
        assert!(!dialog.is_confirm_selected());
        dialog.toggle_selection();
        assert!(dialog.is_confirm_selected());
        dialog.toggle_selection();
        assert!(!dialog.is_confirm_selected());
    }

    #[test]
    fn test_confirm_dialog_select_confirm() {
        let mut dialog = ConfirmDialog::new();
        dialog.show("Confirm", "Test");
        dialog.select_confirm();
        assert!(dialog.is_confirm_selected());
    }

    #[test]
    fn test_confirm_dialog_select_cancel() {
        let mut dialog = ConfirmDialog::new();
        dialog.show("Confirm", "Test");
        dialog.select_confirm();
        dialog.select_cancel();
        assert!(!dialog.is_confirm_selected());
    }

    #[test]
    fn test_confirm_dialog_confirm() {
        let mut dialog = ConfirmDialog::new();
        dialog.show("Confirm", "Test");
        let result = dialog.confirm();
        assert!(result);
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_confirm_dialog_cancel() {
        let mut dialog = ConfirmDialog::new();
        dialog.show("Confirm", "Test");
        let result = dialog.cancel();
        assert!(!result);
        assert!(!dialog.is_visible());
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
        // Should be clamped to area size
        assert_eq!(centered.width, 30);
        assert_eq!(centered.height, 20);
    }

    #[test]
    fn test_confirm_dialog_show_destructive() {
        let mut dialog = ConfirmDialog::new();
        dialog.show_destructive("Delete Item?", "This action cannot be undone.");
        assert!(dialog.is_visible());
        assert!(dialog.is_destructive());
        assert!(!dialog.is_confirm_selected()); // Default to cancel for safety
    }

    #[test]
    fn test_confirm_dialog_show_with_labels() {
        let mut dialog = ConfirmDialog::new();
        dialog.show_with_labels("Save Changes?", "Save your work?", "Save", "Discard");
        assert!(dialog.is_visible());
        assert!(!dialog.is_destructive());
        assert!(!dialog.is_confirm_selected());
    }

    #[test]
    fn test_confirm_dialog_show_destructive_with_label() {
        let mut dialog = ConfirmDialog::new();
        dialog.show_destructive_with_label(
            "Clear Cache?",
            "All cached data will be removed.",
            "Clear",
        );
        assert!(dialog.is_visible());
        assert!(dialog.is_destructive());
        assert!(!dialog.is_confirm_selected());
    }

    #[test]
    fn test_confirm_dialog_normal_not_destructive() {
        let mut dialog = ConfirmDialog::new();
        dialog.show("Confirm Action", "Proceed with this action?");
        assert!(!dialog.is_destructive());
    }

    #[test]
    fn test_confirm_dialog_handle_input_y() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut dialog = ConfirmDialog::new();
        dialog.show("Confirm", "Test");

        let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        let result = dialog.handle_input(key);

        assert_eq!(result, Some(true));
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_confirm_dialog_handle_input_n() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut dialog = ConfirmDialog::new();
        dialog.show("Confirm", "Test");

        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        let result = dialog.handle_input(key);

        assert_eq!(result, Some(false));
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_confirm_dialog_handle_input_esc() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut dialog = ConfirmDialog::new();
        dialog.show("Confirm", "Test");

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let result = dialog.handle_input(key);

        assert_eq!(result, Some(false));
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_confirm_dialog_handle_input_enter_with_selection() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut dialog = ConfirmDialog::new();
        dialog.show("Confirm", "Test");
        dialog.select_confirm();

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let result = dialog.handle_input(key);

        assert_eq!(result, Some(true));
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_confirm_dialog_handle_input_tab_toggles() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut dialog = ConfirmDialog::new();
        dialog.show("Confirm", "Test");
        assert!(!dialog.is_confirm_selected());

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let result = dialog.handle_input(key);

        assert_eq!(result, None); // Tab doesn't confirm/cancel
        assert!(dialog.is_confirm_selected()); // But it toggles
    }
}
