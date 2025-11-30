//! Theme and styling configuration.

use ratatui::style::{Color, Modifier, Style};

use crate::api::types::{Priority, Status};

/// Color theme for the application.
pub struct Theme {
    /// Primary foreground color.
    pub fg: Color,
    /// Primary background color.
    pub bg: Color,
    /// Highlight color for selected items.
    pub highlight: Color,
    /// Background color for selected rows.
    pub selection_bg: Color,
    /// Color for muted/secondary text.
    pub muted: Color,
    /// Color for borders.
    pub border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: Color::White,
            bg: Color::Black,
            highlight: Color::Cyan,
            selection_bg: Color::DarkGray,
            muted: Color::Gray,
            border: Color::DarkGray,
        }
    }
}

impl Theme {
    /// Get style for status based on its category.
    pub fn status_style(&self, status: &Status) -> Style {
        status_style(status)
    }

    /// Get style for priority.
    pub fn priority_style(&self, priority: Option<&Priority>) -> Style {
        priority_style(priority)
    }

    /// Get style for table headers.
    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.fg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for selected row.
    pub fn selection_style(&self) -> Style {
        Style::default().bg(self.selection_bg)
    }
}

/// Get the style for a status based on its category.
///
/// Status categories are:
/// - "new" (To Do): Blue
/// - "indeterminate" (In Progress): Yellow
/// - "done" (Done): Green
pub fn status_style(status: &Status) -> Style {
    match status.status_category.as_ref().map(|c| c.key.as_str()) {
        Some("new") => Style::default().fg(Color::Blue),
        Some("indeterminate") => Style::default().fg(Color::Yellow),
        Some("done") => Style::default().fg(Color::Green),
        _ => Style::default(),
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

/// Get a prefix/icon for an issue type.
///
/// Issue type prefixes:
/// - Bug: 🐛
/// - Story: 📖
/// - Task: ✓
/// - Epic: ⚡
/// - Sub-task: └
/// - Other: •
pub fn issue_type_prefix(issue_type: &str) -> &'static str {
    match issue_type {
        "Bug" => "🐛",
        "Story" => "📖",
        "Task" => "✓",
        "Epic" => "⚡",
        "Sub-task" | "Subtask" => "└",
        _ => "•",
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
        format!("{}…", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::StatusCategory;

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
    fn test_issue_type_prefix() {
        assert_eq!(issue_type_prefix("Bug"), "🐛");
        assert_eq!(issue_type_prefix("Story"), "📖");
        assert_eq!(issue_type_prefix("Task"), "✓");
        assert_eq!(issue_type_prefix("Epic"), "⚡");
        assert_eq!(issue_type_prefix("Sub-task"), "└");
        assert_eq!(issue_type_prefix("Unknown"), "•");
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
        assert_eq!(truncate("hello world", 8), "hello w…");
    }

    #[test]
    fn test_truncate_very_short_max() {
        assert_eq!(truncate("hello", 2), "he");
    }
}
