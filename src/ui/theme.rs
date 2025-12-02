//! Theme and styling configuration.
//!
//! This module provides a comprehensive theming system with built-in themes
//! (dark, light, high-contrast) and support for custom themes via configuration.

// Theme style methods are part of the public API
#![allow(dead_code)]

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::api::types::{Priority, Status};

/// Global theme instance for application-wide access.
static THEME: OnceLock<Theme> = OnceLock::new();

/// Initialize the global theme.
///
/// This should be called once at application startup.
/// Subsequent calls will be ignored.
pub fn init_theme(theme: Theme) {
    let _ = THEME.set(theme);
}

/// Get a reference to the current theme.
///
/// # Panics
///
/// Panics if the theme has not been initialized.
pub fn theme() -> &'static Theme {
    THEME
        .get()
        .expect("Theme not initialized. Call init_theme() first.")
}

/// Try to get a reference to the current theme.
///
/// Returns `None` if the theme has not been initialized.
pub fn try_theme() -> Option<&'static Theme> {
    THEME.get()
}

/// Color theme for the application.
///
/// Contains all color definitions used throughout the UI, organized by category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    /// Theme name identifier.
    pub name: String,

    // === Base Colors ===
    /// Primary foreground color for text.
    pub fg: Color,
    /// Primary background color.
    pub bg: Color,
    /// Color for muted/secondary text.
    pub muted: Color,
    /// Color for dimmed/subtle text.
    pub dim: Color,

    // === Accent Colors ===
    /// Primary accent color for highlights and focus.
    pub accent: Color,
    /// Dimmed accent color.
    pub accent_dim: Color,

    // === Status Colors ===
    /// Color for success states.
    pub success: Color,
    /// Color for warning states.
    pub warning: Color,
    /// Color for error states.
    pub error: Color,
    /// Color for informational states.
    pub info: Color,

    // === Priority Colors ===
    /// Color for highest priority.
    pub priority_highest: Color,
    /// Color for high priority.
    pub priority_high: Color,
    /// Color for medium priority.
    pub priority_medium: Color,
    /// Color for low priority.
    pub priority_low: Color,
    /// Color for lowest priority.
    pub priority_lowest: Color,

    // === Issue Status Category Colors ===
    /// Color for "new" status category (To Do).
    pub status_new: Color,
    /// Color for "indeterminate" status category (In Progress).
    pub status_in_progress: Color,
    /// Color for "done" status category (Done).
    pub status_done: Color,

    // === UI Element Colors ===
    /// Color for borders.
    pub border: Color,
    /// Color for focused borders.
    pub border_focused: Color,
    /// Background color for selected items.
    pub selection_bg: Color,
    /// Foreground color for selected items.
    pub selection_fg: Color,
    /// Background color for headers.
    pub header_bg: Color,
    /// Foreground color for headers.
    pub header_fg: Color,

    // === Component Colors ===
    /// Background color for input fields.
    pub input_bg: Color,
    /// Foreground color for input fields.
    pub input_fg: Color,
    /// Color for input placeholder text.
    pub input_placeholder: Color,
    /// Background color for tags/labels.
    pub tag_bg: Color,
    /// Foreground color for tags/labels.
    pub tag_fg: Color,
    /// Background color for components.
    pub component_bg: Color,
    /// Foreground color for components.
    pub component_fg: Color,

    // === Search/Highlight Colors ===
    /// Background color for search matches.
    pub search_match_bg: Color,
    /// Foreground color for search matches.
    pub search_match_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Create the dark theme (default).
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),

            // Base colors
            fg: Color::White,
            bg: Color::Reset, // Use terminal default
            muted: Color::Gray,
            dim: Color::DarkGray,

            // Accent colors
            accent: Color::Cyan,
            accent_dim: Color::DarkGray,

            // Status colors
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,

            // Priority colors
            priority_highest: Color::Red,
            priority_high: Color::LightRed,
            priority_medium: Color::Yellow,
            priority_low: Color::Green,
            priority_lowest: Color::Gray,

            // Issue status category colors
            status_new: Color::Blue,
            status_in_progress: Color::Yellow,
            status_done: Color::Green,

            // UI element colors
            border: Color::DarkGray,
            border_focused: Color::Cyan,
            selection_bg: Color::DarkGray,
            selection_fg: Color::White,
            header_bg: Color::DarkGray,
            header_fg: Color::White,

            // Component colors
            input_bg: Color::Reset,
            input_fg: Color::White,
            input_placeholder: Color::DarkGray,
            tag_bg: Color::Blue,
            tag_fg: Color::White,
            component_bg: Color::Magenta,
            component_fg: Color::White,

            // Search colors
            search_match_bg: Color::Yellow,
            search_match_fg: Color::Black,
        }
    }

    /// Create the light theme.
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),

            // Base colors
            fg: Color::Black,
            bg: Color::Reset, // Use terminal default
            muted: Color::DarkGray,
            dim: Color::Gray,

            // Accent colors
            accent: Color::Blue,
            accent_dim: Color::Gray,

            // Status colors
            success: Color::Green,
            warning: Color::Rgb(180, 140, 0), // Darker yellow for readability
            error: Color::Red,
            info: Color::Blue,

            // Priority colors
            priority_highest: Color::Red,
            priority_high: Color::Rgb(200, 80, 80),
            priority_medium: Color::Rgb(180, 140, 0),
            priority_low: Color::Blue,
            priority_lowest: Color::Gray,

            // Issue status category colors
            status_new: Color::Blue,
            status_in_progress: Color::Rgb(180, 140, 0),
            status_done: Color::Green,

            // UI element colors
            border: Color::Gray,
            border_focused: Color::Blue,
            selection_bg: Color::LightBlue,
            selection_fg: Color::Black,
            header_bg: Color::Gray,
            header_fg: Color::Black,

            // Component colors
            input_bg: Color::Reset,
            input_fg: Color::Black,
            input_placeholder: Color::Gray,
            tag_bg: Color::LightBlue,
            tag_fg: Color::Black,
            component_bg: Color::LightMagenta,
            component_fg: Color::Black,

            // Search colors
            search_match_bg: Color::Yellow,
            search_match_fg: Color::Black,
        }
    }

    /// Create the high contrast theme for accessibility.
    pub fn high_contrast() -> Self {
        Self {
            name: "high-contrast".to_string(),

            // Base colors - maximum contrast
            fg: Color::White,
            bg: Color::Black,
            muted: Color::White, // No dim colors in high contrast
            dim: Color::White,

            // Accent colors
            accent: Color::Yellow,
            accent_dim: Color::Yellow,

            // Status colors - bright variants
            success: Color::LightGreen,
            warning: Color::Yellow,
            error: Color::LightRed,
            info: Color::LightCyan,

            // Priority colors - bright and distinct
            priority_highest: Color::LightRed,
            priority_high: Color::LightRed,
            priority_medium: Color::Yellow,
            priority_low: Color::LightCyan,
            priority_lowest: Color::White,

            // Issue status category colors
            status_new: Color::LightCyan,
            status_in_progress: Color::Yellow,
            status_done: Color::LightGreen,

            // UI element colors - strong contrast
            border: Color::White,
            border_focused: Color::Yellow,
            selection_bg: Color::White,
            selection_fg: Color::Black,
            header_bg: Color::White,
            header_fg: Color::Black,

            // Component colors
            input_bg: Color::Black,
            input_fg: Color::White,
            input_placeholder: Color::White,
            tag_bg: Color::Yellow,
            tag_fg: Color::Black,
            component_bg: Color::LightMagenta,
            component_fg: Color::Black,

            // Search colors
            search_match_bg: Color::Yellow,
            search_match_fg: Color::Black,
        }
    }

    /// Get a theme by name.
    ///
    /// Supported names: "dark", "light", "high-contrast"
    /// Returns the dark theme for unknown names.
    pub fn by_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "light" => Self::light(),
            "high-contrast" | "highcontrast" | "high_contrast" => Self::high_contrast(),
            _ => Self::dark(),
        }
    }

    // === Style Helper Methods ===

    /// Get the default style with foreground color.
    pub fn style_normal(&self) -> Style {
        Style::default().fg(self.fg)
    }

    /// Get style for muted/secondary text.
    pub fn style_muted(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Get style for dimmed text.
    pub fn style_dim(&self) -> Style {
        Style::default().fg(self.dim)
    }

    /// Get style with accent color.
    pub fn style_accent(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Get style with bold accent.
    pub fn style_accent_bold(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for success states.
    pub fn style_success(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Get style for warning states.
    pub fn style_warning(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Get style for error states.
    pub fn style_error(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Get style for error with bold.
    pub fn style_error_bold(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    /// Get style for informational states.
    pub fn style_info(&self) -> Style {
        Style::default().fg(self.info)
    }

    /// Get style for selected/highlighted items.
    pub fn style_selected(&self) -> Style {
        Style::default().bg(self.selection_bg).fg(self.selection_fg)
    }

    /// Get style for borders.
    pub fn style_border(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Get style for focused borders.
    pub fn style_border_focused(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    /// Get style for table headers.
    pub fn style_header(&self) -> Style {
        Style::default()
            .fg(self.header_fg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for input text.
    pub fn style_input(&self) -> Style {
        Style::default().fg(self.input_fg)
    }

    /// Get style for input placeholder text.
    pub fn style_input_placeholder(&self) -> Style {
        Style::default().fg(self.input_placeholder)
    }

    /// Get style for tags/labels.
    pub fn style_tag(&self) -> Style {
        Style::default().bg(self.tag_bg).fg(self.tag_fg)
    }

    /// Get style for components.
    pub fn style_component(&self) -> Style {
        Style::default().bg(self.component_bg).fg(self.component_fg)
    }

    /// Get style for search matches.
    pub fn style_search_match(&self) -> Style {
        Style::default()
            .bg(self.search_match_bg)
            .fg(self.search_match_fg)
    }

    /// Get style for status based on its category.
    pub fn status_style(&self, status: &Status) -> Style {
        match status.status_category.as_ref().map(|c| c.key.as_str()) {
            Some("new") => Style::default().fg(self.status_new),
            Some("indeterminate") => Style::default().fg(self.status_in_progress),
            Some("done") => Style::default().fg(self.status_done),
            _ => Style::default().fg(self.fg),
        }
    }

    /// Get style for priority.
    pub fn priority_style(&self, priority: Option<&Priority>) -> Style {
        match priority.map(|p| p.name.as_str()) {
            Some("Highest") | Some("Blocker") => Style::default()
                .fg(self.priority_highest)
                .add_modifier(Modifier::BOLD),
            Some("High") | Some("Critical") => Style::default().fg(self.priority_high),
            Some("Medium") => Style::default().fg(self.priority_medium),
            Some("Low") => Style::default().fg(self.priority_low),
            Some("Lowest") => Style::default().fg(self.priority_lowest),
            _ => Style::default().fg(self.fg),
        }
    }

    /// Get color for priority by name (case-insensitive).
    pub fn priority_color(&self, priority_name: &str) -> Color {
        match priority_name.to_lowercase().as_str() {
            "highest" | "blocker" | "critical" => self.priority_highest,
            "high" | "major" => self.priority_high,
            "medium" | "normal" => self.priority_medium,
            "low" | "minor" => self.priority_low,
            "lowest" | "trivial" => self.priority_lowest,
            _ => self.fg,
        }
    }

    /// Get style for status category by key.
    pub fn style_for_status_category(&self, category: &str) -> Style {
        match category {
            "new" => Style::default().fg(self.status_new),
            "indeterminate" => Style::default().fg(self.status_in_progress),
            "done" => Style::default().fg(self.status_done),
            _ => Style::default().fg(self.fg),
        }
    }
}

// === Legacy compatibility functions ===

/// Get the style for a status based on its category.
///
/// Status categories are:
/// - "new" (To Do): Blue
/// - "indeterminate" (In Progress): Yellow
/// - "done" (Done): Green
pub fn status_style(status: &Status) -> Style {
    if let Some(theme) = try_theme() {
        theme.status_style(status)
    } else {
        // Fallback when theme not initialized
        match status.status_category.as_ref().map(|c| c.key.as_str()) {
            Some("new") => Style::default().fg(Color::Blue),
            Some("indeterminate") => Style::default().fg(Color::Yellow),
            Some("done") => Style::default().fg(Color::Green),
            _ => Style::default(),
        }
    }
}

/// Get the style for a priority.
///
/// Priority styles:
/// - Highest/Blocker: Bold Red
/// - High/Critical: Red
/// - Medium: Yellow
/// - Low: Green
/// - Lowest: Gray
pub fn priority_style(priority: Option<&Priority>) -> Style {
    if let Some(theme) = try_theme() {
        theme.priority_style(priority)
    } else {
        // Fallback when theme not initialized
        match priority.map(|p| p.name.as_str()) {
            Some("Highest") | Some("Blocker") => {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            }
            Some("High") | Some("Critical") => Style::default().fg(Color::Red),
            Some("Medium") => Style::default().fg(Color::Yellow),
            Some("Low") => Style::default().fg(Color::Green),
            Some("Lowest") => Style::default().fg(Color::Gray),
            _ => Style::default(),
        }
    }
}

/// Get a prefix/icon for an issue type.
///
/// Issue type prefixes:
/// - Bug: B
/// - Story: S
/// - Task: T
/// - Epic: E
/// - Sub-task: -
/// - Other: *
pub fn issue_type_prefix(issue_type: &str) -> &'static str {
    match issue_type {
        "Bug" => "B",
        "Story" => "S",
        "Task" => "T",
        "Epic" => "E",
        "Sub-task" | "Subtask" => "-",
        _ => "*",
    }
}

/// Truncate a string to the given maximum width, adding ellipsis if needed.
pub fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width <= 3 {
        s.chars().take(max_width).collect()
    } else {
        let truncated: String = s.chars().take(max_width - 1).collect();
        format!("{}...", truncated)
    }
}

/// Configuration for custom theme colors.
///
/// Allows overriding individual colors of a base theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomThemeConfig {
    /// Override accent color.
    #[serde(default)]
    pub accent: Option<String>,
    /// Override success color.
    #[serde(default)]
    pub success: Option<String>,
    /// Override warning color.
    #[serde(default)]
    pub warning: Option<String>,
    /// Override error color.
    #[serde(default)]
    pub error: Option<String>,
    /// Override info color.
    #[serde(default)]
    pub info: Option<String>,
    /// Override border color.
    #[serde(default)]
    pub border: Option<String>,
    /// Override border focused color.
    #[serde(default)]
    pub border_focused: Option<String>,
    /// Override tag background color.
    #[serde(default)]
    pub tag_bg: Option<String>,
    /// Override tag foreground color.
    #[serde(default)]
    pub tag_fg: Option<String>,
}

impl CustomThemeConfig {
    /// Apply custom colors to a theme.
    pub fn apply_to(&self, theme: &mut Theme) {
        if let Some(color) = self.accent.as_ref().and_then(|s| parse_color(s)) {
            theme.accent = color;
        }
        if let Some(color) = self.success.as_ref().and_then(|s| parse_color(s)) {
            theme.success = color;
        }
        if let Some(color) = self.warning.as_ref().and_then(|s| parse_color(s)) {
            theme.warning = color;
        }
        if let Some(color) = self.error.as_ref().and_then(|s| parse_color(s)) {
            theme.error = color;
        }
        if let Some(color) = self.info.as_ref().and_then(|s| parse_color(s)) {
            theme.info = color;
        }
        if let Some(color) = self.border.as_ref().and_then(|s| parse_color(s)) {
            theme.border = color;
        }
        if let Some(color) = self.border_focused.as_ref().and_then(|s| parse_color(s)) {
            theme.border_focused = color;
        }
        if let Some(color) = self.tag_bg.as_ref().and_then(|s| parse_color(s)) {
            theme.tag_bg = color;
        }
        if let Some(color) = self.tag_fg.as_ref().and_then(|s| parse_color(s)) {
            theme.tag_fg = color;
        }
    }
}

/// Parse a color string into a ratatui Color.
///
/// Supports:
/// - Named colors: "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white", "gray"
/// - Light variants: "light_red", "light-green", etc.
/// - Hex colors: "#ff0000", "#f00"
/// - RGB: "rgb(255, 0, 0)"
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().to_lowercase();

    // Named colors
    match s.as_str() {
        "black" => return Some(Color::Black),
        "red" => return Some(Color::Red),
        "green" => return Some(Color::Green),
        "yellow" => return Some(Color::Yellow),
        "blue" => return Some(Color::Blue),
        "magenta" => return Some(Color::Magenta),
        "cyan" => return Some(Color::Cyan),
        "white" => return Some(Color::White),
        "gray" | "grey" => return Some(Color::Gray),
        "darkgray" | "darkgrey" | "dark_gray" | "dark-gray" => return Some(Color::DarkGray),
        "lightred" | "light_red" | "light-red" => return Some(Color::LightRed),
        "lightgreen" | "light_green" | "light-green" => return Some(Color::LightGreen),
        "lightyellow" | "light_yellow" | "light-yellow" => return Some(Color::LightYellow),
        "lightblue" | "light_blue" | "light-blue" => return Some(Color::LightBlue),
        "lightmagenta" | "light_magenta" | "light-magenta" => return Some(Color::LightMagenta),
        "lightcyan" | "light_cyan" | "light-cyan" => return Some(Color::LightCyan),
        "reset" | "default" => return Some(Color::Reset),
        _ => {}
    }

    // Hex colors: #rgb or #rrggbb
    if let Some(hex) = s.strip_prefix('#') {
        match hex.len() {
            3 => {
                // #rgb -> #rrggbb
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                return Some(Color::Rgb(r, g, b));
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(Color::Rgb(r, g, b));
            }
            _ => return None,
        }
    }

    // RGB format: rgb(r, g, b)
    if s.starts_with("rgb(") && s.ends_with(')') {
        let inner = &s[4..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            let r: u8 = parts[0].trim().parse().ok()?;
            let g: u8 = parts[1].trim().parse().ok()?;
            let b: u8 = parts[2].trim().parse().ok()?;
            return Some(Color::Rgb(r, g, b));
        }
    }

    None
}

/// Load a theme from settings.
///
/// Uses the base theme specified in settings and applies any custom color overrides.
pub fn load_theme(theme_name: &str, custom_config: Option<&CustomThemeConfig>) -> Theme {
    let mut theme = Theme::by_name(theme_name);

    if let Some(custom) = custom_config {
        custom.apply_to(&mut theme);
    }

    theme
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::StatusCategory;

    #[test]
    fn test_theme_dark() {
        let theme = Theme::dark();
        assert_eq!(theme.name, "dark");
        assert_eq!(theme.fg, Color::White);
        assert_eq!(theme.accent, Color::Cyan);
    }

    #[test]
    fn test_theme_light() {
        let theme = Theme::light();
        assert_eq!(theme.name, "light");
        assert_eq!(theme.fg, Color::Black);
        assert_eq!(theme.accent, Color::Blue);
    }

    #[test]
    fn test_theme_high_contrast() {
        let theme = Theme::high_contrast();
        assert_eq!(theme.name, "high-contrast");
        assert_eq!(theme.fg, Color::White);
        assert_eq!(theme.bg, Color::Black);
        assert_eq!(theme.accent, Color::Yellow);
    }

    #[test]
    fn test_theme_by_name() {
        assert_eq!(Theme::by_name("dark").name, "dark");
        assert_eq!(Theme::by_name("light").name, "light");
        assert_eq!(Theme::by_name("high-contrast").name, "high-contrast");
        assert_eq!(Theme::by_name("highcontrast").name, "high-contrast");
        assert_eq!(Theme::by_name("unknown").name, "dark"); // fallback
    }

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("red"), Some(Color::Red));
        assert_eq!(parse_color("RED"), Some(Color::Red));
        assert_eq!(parse_color("Green"), Some(Color::Green));
        assert_eq!(parse_color("gray"), Some(Color::Gray));
        assert_eq!(parse_color("grey"), Some(Color::Gray));
        assert_eq!(parse_color("darkgray"), Some(Color::DarkGray));
        assert_eq!(parse_color("light-red"), Some(Color::LightRed));
    }

    #[test]
    fn test_parse_color_hex() {
        assert_eq!(parse_color("#ff0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#00ff00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_color("#0000ff"), Some(Color::Rgb(0, 0, 255)));
        assert_eq!(parse_color("#f00"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#0f0"), Some(Color::Rgb(0, 255, 0)));
    }

    #[test]
    fn test_parse_color_rgb() {
        assert_eq!(parse_color("rgb(255, 0, 0)"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("rgb(0,255,0)"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(
            parse_color("rgb( 0 , 0 , 255 )"),
            Some(Color::Rgb(0, 0, 255))
        );
    }

    #[test]
    fn test_parse_color_invalid() {
        assert_eq!(parse_color("invalid"), None);
        assert_eq!(parse_color("#gg0000"), None);
        assert_eq!(parse_color("rgb(256, 0, 0)"), None);
    }

    #[test]
    fn test_custom_theme_apply() {
        let mut theme = Theme::dark();
        let custom = CustomThemeConfig {
            accent: Some("#ff00ff".to_string()),
            success: Some("lightgreen".to_string()),
            ..Default::default()
        };

        custom.apply_to(&mut theme);

        assert_eq!(theme.accent, Color::Rgb(255, 0, 255));
        assert_eq!(theme.success, Color::LightGreen);
        // Unchanged colors should remain
        assert_eq!(theme.error, Color::Red);
    }

    #[test]
    fn test_load_theme() {
        let theme = load_theme("dark", None);
        assert_eq!(theme.name, "dark");

        let custom = CustomThemeConfig {
            accent: Some("magenta".to_string()),
            ..Default::default()
        };
        let theme = load_theme("light", Some(&custom));
        assert_eq!(theme.name, "light");
        assert_eq!(theme.accent, Color::Magenta);
    }

    #[test]
    fn test_status_style_new() {
        let status = Status {
            id: "1".to_string(),
            name: "To Do".to_string(),
            status_category: Some(StatusCategory {
                id: 2,
                key: "new".to_string(),
                name: "To Do".to_string(),
                color_name: Some("blue-gray".to_string()),
            }),
        };
        let style = status_style(&status);
        assert_eq!(style.fg, Some(Color::Blue));
    }

    #[test]
    fn test_status_style_in_progress() {
        let status = Status {
            id: "2".to_string(),
            name: "In Progress".to_string(),
            status_category: Some(StatusCategory {
                id: 4,
                key: "indeterminate".to_string(),
                name: "In Progress".to_string(),
                color_name: Some("yellow".to_string()),
            }),
        };
        let style = status_style(&status);
        assert_eq!(style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_status_style_done() {
        let status = Status {
            id: "3".to_string(),
            name: "Done".to_string(),
            status_category: Some(StatusCategory {
                id: 3,
                key: "done".to_string(),
                name: "Done".to_string(),
                color_name: Some("green".to_string()),
            }),
        };
        let style = status_style(&status);
        assert_eq!(style.fg, Some(Color::Green));
    }

    #[test]
    fn test_status_style_no_category() {
        let status = Status {
            id: "1".to_string(),
            name: "Custom".to_string(),
            status_category: None,
        };
        let style = status_style(&status);
        assert_eq!(style.fg, None);
    }

    #[test]
    fn test_priority_style_highest() {
        let priority = Priority {
            id: "1".to_string(),
            name: "Highest".to_string(),
            icon_url: None,
        };
        let style = priority_style(Some(&priority));
        assert_eq!(style.fg, Some(Color::Red));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_priority_style_high() {
        let priority = Priority {
            id: "2".to_string(),
            name: "High".to_string(),
            icon_url: None,
        };
        let style = priority_style(Some(&priority));
        assert_eq!(style.fg, Some(Color::Red));
    }

    #[test]
    fn test_priority_style_medium() {
        let priority = Priority {
            id: "3".to_string(),
            name: "Medium".to_string(),
            icon_url: None,
        };
        let style = priority_style(Some(&priority));
        assert_eq!(style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_priority_style_low() {
        let priority = Priority {
            id: "4".to_string(),
            name: "Low".to_string(),
            icon_url: None,
        };
        let style = priority_style(Some(&priority));
        assert_eq!(style.fg, Some(Color::Green));
    }

    #[test]
    fn test_priority_style_lowest() {
        let priority = Priority {
            id: "5".to_string(),
            name: "Lowest".to_string(),
            icon_url: None,
        };
        let style = priority_style(Some(&priority));
        assert_eq!(style.fg, Some(Color::Gray));
    }

    #[test]
    fn test_priority_style_none() {
        let style = priority_style(None);
        assert_eq!(style.fg, None);
    }

    #[test]
    fn test_theme_style_helpers() {
        let theme = Theme::dark();

        assert_eq!(theme.style_normal().fg, Some(Color::White));
        assert_eq!(theme.style_muted().fg, Some(Color::Gray));
        assert_eq!(theme.style_accent().fg, Some(Color::Cyan));
        assert_eq!(theme.style_error().fg, Some(Color::Red));
        assert_eq!(theme.style_success().fg, Some(Color::Green));
        assert_eq!(theme.style_selected().bg, Some(Color::DarkGray));
        assert_eq!(theme.style_selected().fg, Some(Color::White));
    }

    #[test]
    fn test_theme_priority_color() {
        let theme = Theme::dark();

        assert_eq!(theme.priority_color("Highest"), Color::Red);
        assert_eq!(theme.priority_color("HIGHEST"), Color::Red);
        assert_eq!(theme.priority_color("high"), Color::LightRed);
        assert_eq!(theme.priority_color("Medium"), Color::Yellow);
        assert_eq!(theme.priority_color("low"), Color::Green);
        assert_eq!(theme.priority_color("Unknown"), Color::White);
    }

    #[test]
    fn test_issue_type_prefix() {
        assert_eq!(issue_type_prefix("Bug"), "B");
        assert_eq!(issue_type_prefix("Story"), "S");
        assert_eq!(issue_type_prefix("Task"), "T");
        assert_eq!(issue_type_prefix("Epic"), "E");
        assert_eq!(issue_type_prefix("Sub-task"), "-");
        assert_eq!(issue_type_prefix("Unknown"), "*");
    }

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate("hello world", 8), "hello w...");
    }

    #[test]
    fn test_truncate_very_short_max() {
        assert_eq!(truncate("hello", 2), "he");
    }
}
