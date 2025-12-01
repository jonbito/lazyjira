//! Command palette component.
//!
//! This module provides a command palette (Ctrl+P style) for quick access to
//! all application features. It supports:
//! - Fuzzy search for commands
//! - Recent commands section
//! - Categories for organization
//! - Keyboard navigation (j/k, arrows)
//! - Command preview with shortcuts

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::commands::{Command, CommandAction, CommandCategory, CommandRegistry};
use crate::ui::components::TextInput;

/// Actions returned from command palette input handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPaletteAction {
    /// Execute a command action.
    Execute(CommandAction),
    /// Cancel and close the palette.
    Cancel,
}

/// Command palette component for quick command access.
pub struct CommandPalette {
    /// Command registry with all available commands.
    registry: CommandRegistry,
    /// Text input for search query.
    search_input: TextInput,
    /// Filtered results based on current query.
    results: Vec<Command>,
    /// Currently selected result index.
    selected: usize,
    /// Whether the palette is visible.
    visible: bool,
}

impl CommandPalette {
    /// Create a new command palette.
    pub fn new() -> Self {
        let registry = CommandRegistry::new();
        let results = registry.search("").into_iter().cloned().collect();

        Self {
            registry,
            search_input: TextInput::new(),
            results,
            selected: 0,
            visible: false,
        }
    }

    /// Show the command palette.
    pub fn show(&mut self) {
        self.visible = true;
        self.search_input.clear();
        self.update_results();
        self.selected = 0;
    }

    /// Hide the command palette.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if the palette is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the current search query.
    pub fn query(&self) -> &str {
        self.search_input.value()
    }

    /// Update the filtered results based on current query.
    fn update_results(&mut self) {
        self.results = self
            .registry
            .search(self.search_input.value())
            .into_iter()
            .cloned()
            .collect();
    }

    /// Handle keyboard input.
    ///
    /// Returns an action if the palette should perform one.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<CommandPaletteAction> {
        if !self.visible {
            return None;
        }

        match (key.code, key.modifiers) {
            // Escape - cancel
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.hide();
                Some(CommandPaletteAction::Cancel)
            }

            // Enter - execute selected command
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if !self.results.is_empty() {
                    let cmd = &self.results[self.selected];
                    self.registry.record_used(&cmd.id);
                    let action = cmd.action.clone();
                    self.hide();
                    Some(CommandPaletteAction::Execute(action))
                } else {
                    None
                }
            }

            // Navigation: Down, Tab, j, Ctrl+n
            (KeyCode::Down, KeyModifiers::NONE)
            | (KeyCode::Tab, KeyModifiers::NONE)
            | (KeyCode::Char('j'), KeyModifiers::CONTROL)
            | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                if !self.results.is_empty() {
                    self.selected = (self.selected + 1) % self.results.len();
                }
                None
            }

            // Navigation: Up, Shift+Tab, k, Ctrl+p
            (KeyCode::Up, KeyModifiers::NONE)
            | (KeyCode::BackTab, KeyModifiers::SHIFT)
            | (KeyCode::Char('k'), KeyModifiers::CONTROL)
            | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                if !self.results.is_empty() {
                    self.selected = if self.selected == 0 {
                        self.results.len() - 1
                    } else {
                        self.selected - 1
                    };
                }
                None
            }

            // Character input (but not j/k in normal mode - those navigate)
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                // Allow j/k as regular characters in search
                self.search_input.handle_input(key);
                self.update_results();
                self.selected = 0;
                None
            }

            // Any other input - delegate to text input
            _ => {
                if self.search_input.handle_input(key) {
                    self.update_results();
                    self.selected = 0;
                }
                None
            }
        }
    }

    /// Render the command palette.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate palette dimensions (centered, ~60 chars wide, ~15 lines tall)
        let width = 60.min(area.width.saturating_sub(4));
        let height = 15.min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(width)) / 2;
        let y = area.height / 6; // Position in upper third

        let palette_area = Rect::new(x, y, width, height);

        // Clear the background
        frame.render_widget(Clear, palette_area);

        // Outer block with border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Command Palette ")
            .title_style(Style::default().add_modifier(Modifier::BOLD));

        let inner = block.inner(palette_area);
        frame.render_widget(block, palette_area);

        // Layout: search input + results
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Search input
                Constraint::Length(1), // Separator
                Constraint::Min(1),    // Results list
            ])
            .split(inner);

        // Render search input
        self.render_search(frame, chunks[0]);

        // Render separator
        let separator = Paragraph::new(Line::from(vec![Span::styled(
            "─".repeat(chunks[1].width as usize),
            Style::default().fg(Color::DarkGray),
        )]));
        frame.render_widget(separator, chunks[1]);

        // Render results
        self.render_results(frame, chunks[2]);
    }

    /// Render the search input area.
    fn render_search(&self, frame: &mut Frame, area: Rect) {
        let query = self.search_input.value();
        let display = if query.is_empty() {
            Line::from(Span::styled(
                "Type to search commands...",
                Style::default().fg(Color::DarkGray),
            ))
        } else {
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Cyan)),
                Span::raw(query),
            ])
        };

        let input = Paragraph::new(display);
        frame.render_widget(input, area);

        // Set cursor position
        let cursor_x = if query.is_empty() {
            area.x
        } else {
            area.x + 2 + self.search_input.cursor() as u16
        };

        frame.set_cursor_position(Position::new(cursor_x, area.y));
    }

    /// Render the results list.
    fn render_results(&self, frame: &mut Frame, area: Rect) {
        if self.results.is_empty() {
            let empty = Paragraph::new(Line::from(Span::styled(
                "No matching commands",
                Style::default().fg(Color::DarkGray),
            )));
            frame.render_widget(empty, area);
            return;
        }

        let visible_count = area.height as usize;

        // Calculate scroll offset to keep selected item visible
        let scroll_offset = if self.selected >= visible_count {
            self.selected - visible_count + 1
        } else {
            0
        };

        // Group results by category for display
        let items: Vec<ListItem> = self
            .results
            .iter()
            .skip(scroll_offset)
            .take(visible_count)
            .enumerate()
            .map(|(idx, cmd)| {
                let actual_idx = scroll_offset + idx;
                let is_selected = actual_idx == self.selected;

                self.render_command_item(cmd, is_selected, area.width as usize)
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, area);
    }

    /// Render a single command item.
    fn render_command_item<'a>(
        &self,
        cmd: &'a Command,
        is_selected: bool,
        width: usize,
    ) -> ListItem<'a> {
        let style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        // Build the line with title, category badge, and shortcut
        let mut spans = Vec::new();

        // Selection indicator
        if is_selected {
            spans.push(Span::styled("> ", Style::default().fg(Color::Cyan)));
        } else {
            spans.push(Span::raw("  "));
        }

        // Command title
        spans.push(Span::styled(&cmd.title, style.add_modifier(Modifier::BOLD)));

        // Category badge
        let category_style = self.category_style(cmd.category);
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("[{}]", cmd.category.display()),
            category_style,
        ));

        // Shortcut (if any)
        if let Some(shortcut) = &cmd.shortcut {
            // Calculate remaining space
            let used_width: usize = spans.iter().map(|s| s.content.len()).sum();
            let shortcut_text = format!(" {}", shortcut);
            let padding_needed = width.saturating_sub(used_width + shortcut_text.len() + 1);

            if padding_needed > 0 {
                spans.push(Span::raw(" ".repeat(padding_needed)));
            }
            spans.push(Span::styled(
                shortcut_text,
                Style::default().fg(Color::DarkGray),
            ));
        }

        ListItem::new(Line::from(spans)).style(style)
    }

    /// Get the style for a category badge.
    fn category_style(&self, category: CommandCategory) -> Style {
        match category {
            CommandCategory::Navigation => Style::default().fg(Color::Blue),
            CommandCategory::Issue => Style::default().fg(Color::Green),
            CommandCategory::Profile => Style::default().fg(Color::Magenta),
            CommandCategory::Filter => Style::default().fg(Color::Yellow),
            CommandCategory::Settings => Style::default().fg(Color::Gray),
            CommandCategory::Help => Style::default().fg(Color::Cyan),
        }
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let palette = CommandPalette::new();
        assert!(!palette.is_visible());
        assert!(palette.query().is_empty());
        assert!(!palette.results.is_empty()); // Should have default commands
    }

    #[test]
    fn test_show_hide() {
        let mut palette = CommandPalette::new();

        assert!(!palette.is_visible());

        palette.show();
        assert!(palette.is_visible());
        assert!(palette.query().is_empty());
        assert_eq!(palette.selected, 0);

        palette.hide();
        assert!(!palette.is_visible());
    }

    #[test]
    fn test_show_resets_state() {
        let mut palette = CommandPalette::new();
        palette.show();

        // Type something and select a different item
        palette.handle_input(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
        palette.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));

        // Show again should reset
        palette.show();
        assert!(palette.query().is_empty());
        assert_eq!(palette.selected, 0);
    }

    #[test]
    fn test_escape_cancels() {
        let mut palette = CommandPalette::new();
        palette.show();

        let action = palette.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert_eq!(action, Some(CommandPaletteAction::Cancel));
        assert!(!palette.is_visible());
    }

    #[test]
    fn test_enter_executes() {
        let mut palette = CommandPalette::new();
        palette.show();

        let action = palette.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(matches!(action, Some(CommandPaletteAction::Execute(_))));
        assert!(!palette.is_visible());
    }

    #[test]
    fn test_enter_on_empty_results() {
        let mut palette = CommandPalette::new();
        palette.show();

        // Type something that matches nothing
        for c in "xyznomatch".chars() {
            palette.handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }

        // Enter should do nothing
        let action = palette.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(action.is_none());
        assert!(palette.is_visible()); // Should still be visible
    }

    #[test]
    fn test_navigation_down() {
        let mut palette = CommandPalette::new();
        palette.show();
        assert_eq!(palette.selected, 0);

        palette.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(palette.selected, 1);
    }

    #[test]
    fn test_navigation_up() {
        let mut palette = CommandPalette::new();
        palette.show();

        // Move down first
        palette.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        palette.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(palette.selected, 2);

        // Move up
        palette.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(palette.selected, 1);
    }

    #[test]
    fn test_navigation_wraps() {
        let mut palette = CommandPalette::new();
        palette.show();

        // Up from first item should wrap to last
        palette.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(palette.selected, palette.results.len() - 1);

        // Down from last item should wrap to first
        palette.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(palette.selected, 0);
    }

    #[test]
    fn test_tab_navigation() {
        let mut palette = CommandPalette::new();
        palette.show();

        palette.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(palette.selected, 1);
    }

    #[test]
    fn test_search_filters_results() {
        let mut palette = CommandPalette::new();
        palette.show();
        let initial_count = palette.results.len();

        // Type "help"
        for c in "help".chars() {
            palette.handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }

        assert!(palette.results.len() < initial_count);
        assert!(palette
            .results
            .iter()
            .any(|c| c.title.to_lowercase().contains("help")));
    }

    #[test]
    fn test_search_resets_selection() {
        let mut palette = CommandPalette::new();
        palette.show();

        // Move selection
        palette.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        palette.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert!(palette.selected > 0);

        // Type something
        palette.handle_input(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));

        // Selection should reset to 0
        assert_eq!(palette.selected, 0);
    }

    #[test]
    fn test_backspace_clears_search() {
        let mut palette = CommandPalette::new();
        palette.show();

        // Type something
        palette.handle_input(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
        assert_eq!(palette.query(), "h");

        // Backspace
        palette.handle_input(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert!(palette.query().is_empty());
    }

    #[test]
    fn test_j_k_as_characters() {
        let mut palette = CommandPalette::new();
        palette.show();

        // j and k should type characters, not navigate
        palette.handle_input(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        palette.handle_input(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));

        assert_eq!(palette.query(), "jk");
    }

    #[test]
    fn test_ctrl_j_k_navigates() {
        let mut palette = CommandPalette::new();
        palette.show();

        // Ctrl+j should navigate down
        palette.handle_input(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL));
        assert_eq!(palette.selected, 1);

        // Ctrl+k should navigate up
        palette.handle_input(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL));
        assert_eq!(palette.selected, 0);
    }

    #[test]
    fn test_navigation_on_empty_results() {
        let mut palette = CommandPalette::new();
        palette.show();

        // Search for something that matches nothing
        for c in "xyznomatch".chars() {
            palette.handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
        assert!(palette.results.is_empty());

        // Navigation should not panic
        palette.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        palette.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    }

    #[test]
    fn test_hidden_palette_ignores_input() {
        let mut palette = CommandPalette::new();
        // Don't call show()

        let action = palette.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(action.is_none());
    }
}
