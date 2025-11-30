//! Theme and styling configuration.

use ratatui::style::Color;

/// Color theme for the application.
pub struct Theme {
    /// Primary foreground color.
    pub fg: Color,
    /// Primary background color.
    pub bg: Color,
    /// Highlight color for selected items.
    pub highlight: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: Color::White,
            bg: Color::Black,
            highlight: Color::Cyan,
        }
    }
}
