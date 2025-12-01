//! Help panel view displaying keyboard shortcuts.
//!
//! Provides a scrollable help panel that displays all keyboard shortcuts
//! organized by context. The panel can be opened with '?' and closed with
//! '?', 'q', or Escape.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::events::{get_keybindings_grouped, KeyContext, Keybinding};
use crate::ui::theme::theme;

/// Actions that can be returned from the help view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpAction {
    /// Close the help panel.
    Close,
}

/// The help panel view.
pub struct HelpView {
    /// Keybindings grouped by context.
    grouped_bindings: Vec<(KeyContext, Vec<Keybinding>)>,
    /// Current scroll position.
    scroll: usize,
    /// Total number of lines in the help content.
    total_lines: usize,
    /// Visible height (updated on render).
    visible_height: usize,
}

impl HelpView {
    /// Create a new help view.
    pub fn new() -> Self {
        let grouped_bindings = get_keybindings_grouped();
        let total_lines = Self::calculate_total_lines(&grouped_bindings);

        Self {
            grouped_bindings,
            scroll: 0,
            total_lines,
            visible_height: 0,
        }
    }

    /// Calculate the total number of lines in the help content.
    fn calculate_total_lines(grouped: &[(KeyContext, Vec<Keybinding>)]) -> usize {
        let mut lines = 0;
        for (_, bindings) in grouped {
            // Header line + empty line after header
            lines += 2;
            // Keybindings
            lines += bindings.len();
            // Empty line after section
            lines += 1;
        }
        // Footer line
        lines += 1;
        lines
    }

    /// Reset scroll position to top.
    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
    }

    /// Get the maximum scroll position.
    fn max_scroll(&self) -> usize {
        self.total_lines.saturating_sub(self.visible_height)
    }

    /// Handle keyboard input for the help view.
    ///
    /// Returns `Some(HelpAction)` if an action should be taken, `None` otherwise.
    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<HelpAction> {
        match (key_event.code, key_event.modifiers) {
            // Close help panel
            (KeyCode::Char('?'), KeyModifiers::NONE)
            | (KeyCode::Char('q'), KeyModifiers::NONE)
            | (KeyCode::Esc, KeyModifiers::NONE) => Some(HelpAction::Close),

            // Scroll down
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
                self.scroll = (self.scroll + 1).min(self.max_scroll());
                None
            }

            // Scroll up
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
                self.scroll = self.scroll.saturating_sub(1);
                None
            }

            // Page down
            (KeyCode::Char('d'), KeyModifiers::CONTROL)
            | (KeyCode::PageDown, KeyModifiers::NONE) => {
                let page_size = self.visible_height.saturating_sub(2);
                self.scroll = (self.scroll + page_size).min(self.max_scroll());
                None
            }

            // Page up
            (KeyCode::Char('u'), KeyModifiers::CONTROL) | (KeyCode::PageUp, KeyModifiers::NONE) => {
                let page_size = self.visible_height.saturating_sub(2);
                self.scroll = self.scroll.saturating_sub(page_size);
                None
            }

            // Go to top
            (KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.scroll = 0;
                None
            }

            // Go to bottom
            (KeyCode::Char('G'), KeyModifiers::SHIFT)
            | (KeyCode::Char('G'), KeyModifiers::NONE) => {
                self.scroll = self.max_scroll();
                None
            }

            // Consume all other input when help is open
            _ => None,
        }
    }

    /// Render the help view.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let t = theme();

        // Clear the area first
        frame.render_widget(Clear, area);

        // Create the help panel block
        let block = Block::default()
            .title(" Help - Keyboard Shortcuts ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.accent));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Update visible height for scroll calculations
        self.visible_height = inner.height as usize;

        // Build the content lines
        let lines = self.build_content_lines();

        // Create paragraph with scroll
        let paragraph = Paragraph::new(lines)
            .scroll((self.scroll as u16, 0))
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, inner);

        // Render scrollbar if needed
        if self.total_lines > self.visible_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"));

            let mut scrollbar_state = ScrollbarState::new(self.max_scroll()).position(self.scroll);

            // Render scrollbar in the right margin of the help area
            let scrollbar_area = Rect::new(
                area.x + area.width - 1,
                area.y + 1,
                1,
                area.height.saturating_sub(2),
            );

            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }
    }

    /// Build the content lines for the help panel.
    fn build_content_lines(&self) -> Vec<Line<'static>> {
        let t = theme();
        let mut lines: Vec<Line<'static>> = Vec::new();

        for (context, bindings) in &self.grouped_bindings {
            // Context header
            lines.push(Line::from(vec![Span::styled(
                format!("── {} ──", context.display()),
                Style::default().fg(t.warning).add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));

            // Keybindings in this context
            for binding in bindings {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{:>14}", binding.key),
                        Style::default().fg(t.success).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::raw(binding.description.clone()),
                ]));
            }

            lines.push(Line::from(""));
        }

        // Footer hint
        lines.push(Line::from(vec![Span::styled(
            "Press ?, q, or Esc to close",
            Style::default().fg(t.dim),
        )]));

        lines
    }
}

impl Default for HelpView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::{init_theme, Theme};
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_test_theme() {
        INIT.call_once(|| {
            init_theme(Theme::dark());
        });
    }

    #[test]
    fn test_help_view_new() {
        let view = HelpView::new();
        assert_eq!(view.scroll, 0);
        assert!(view.total_lines > 0);
        assert!(!view.grouped_bindings.is_empty());
    }

    #[test]
    fn test_help_view_default() {
        let view = HelpView::default();
        assert_eq!(view.scroll, 0);
    }

    #[test]
    fn test_reset_scroll() {
        let mut view = HelpView::new();
        view.scroll = 10;
        view.reset_scroll();
        assert_eq!(view.scroll, 0);
    }

    #[test]
    fn test_handle_input_close_question_mark() {
        let mut view = HelpView::new();
        let key = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert_eq!(action, Some(HelpAction::Close));
    }

    #[test]
    fn test_handle_input_close_q() {
        let mut view = HelpView::new();
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert_eq!(action, Some(HelpAction::Close));
    }

    #[test]
    fn test_handle_input_close_escape() {
        let mut view = HelpView::new();
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert_eq!(action, Some(HelpAction::Close));
    }

    #[test]
    fn test_handle_input_scroll_down() {
        let mut view = HelpView::new();
        view.visible_height = 10; // Simulate visible area
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let _ = view.handle_input(key);
        assert_eq!(view.scroll, 1);
    }

    #[test]
    fn test_handle_input_scroll_up() {
        let mut view = HelpView::new();
        view.scroll = 5;
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let _ = view.handle_input(key);
        assert_eq!(view.scroll, 4);
    }

    #[test]
    fn test_handle_input_go_top() {
        let mut view = HelpView::new();
        view.scroll = 10;
        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        let _ = view.handle_input(key);
        assert_eq!(view.scroll, 0);
    }

    #[test]
    fn test_handle_input_go_bottom() {
        let mut view = HelpView::new();
        view.visible_height = 10;
        let key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        let _ = view.handle_input(key);
        assert!(view.scroll <= view.max_scroll());
    }

    #[test]
    fn test_build_content_lines() {
        init_test_theme();
        let view = HelpView::new();
        let lines = view.build_content_lines();
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_scroll_does_not_exceed_max() {
        let mut view = HelpView::new();
        view.visible_height = 100; // Large visible area
                                   // Try to scroll past max
        for _ in 0..200 {
            let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
            let _ = view.handle_input(key);
        }
        assert!(view.scroll <= view.max_scroll());
    }

    #[test]
    fn test_scroll_does_not_go_negative() {
        let mut view = HelpView::new();
        view.scroll = 0;
        // Try to scroll up past 0
        for _ in 0..10 {
            let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
            let _ = view.handle_input(key);
        }
        assert_eq!(view.scroll, 0);
    }
}
