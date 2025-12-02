//! Contextual help bar component.
//!
//! Displays context-sensitive keyboard shortcut hints at the bottom of the screen.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::events::{get_context_hints, KeyContext};

/// Render a contextual help bar with hints for the given context.
///
/// This renders a single line of text showing the most commonly used
/// keyboard shortcuts for the current view/context.
pub fn render_context_help(frame: &mut Frame, area: Rect, context: KeyContext) {
    let hints = get_context_hints(context);

    let spans = parse_hints_to_spans(hints);
    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);

    frame.render_widget(paragraph, area);
}

/// Parse hint text into styled spans.
///
/// Highlights the key portion (in brackets) differently from the description.
fn parse_hints_to_spans(hints: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let chars = hints.chars().peekable();
    let mut current = String::new();
    let mut in_bracket = false;

    for c in chars {
        match c {
            '[' => {
                // Flush any pending text
                if !current.is_empty() {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(Color::DarkGray),
                    ));
                    current.clear();
                }
                in_bracket = true;
                current.push(c);
            }
            ']' => {
                current.push(c);
                if in_bracket {
                    // This is a key, style it differently
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(Color::Cyan),
                    ));
                    current.clear();
                    in_bracket = false;
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    // Flush any remaining text
    if !current.is_empty() {
        spans.push(Span::styled(current, Style::default().fg(Color::DarkGray)));
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hints_to_spans_simple() {
        let hints = "[j/k] navigate";
        let spans = parse_hints_to_spans(hints);
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn test_parse_hints_to_spans_multiple_keys() {
        let hints = "[j/k] navigate  [Enter] open  [?] help";
        let spans = parse_hints_to_spans(hints);
        // Should have: [j/k], " navigate  ", [Enter], " open  ", [?], " help"
        assert_eq!(spans.len(), 6);
    }

    #[test]
    fn test_parse_hints_to_spans_empty() {
        let hints = "";
        let spans = parse_hints_to_spans(hints);
        assert!(spans.is_empty());
    }

    #[test]
    fn test_parse_hints_to_spans_no_brackets() {
        let hints = "just text";
        let spans = parse_hints_to_spans(hints);
        assert_eq!(spans.len(), 1);
    }
}
