//! Quick search component for filtering loaded issues.
//!
//! This component provides real-time filtering of issues as the user types.
//! It supports case-insensitive matching across key, summary, and status fields,
//! with match highlighting and next/previous match navigation.

use ratatui::{
    layout::{Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::api::types::Issue;

/// Quick search state for filtering issues.
#[derive(Debug, Clone, Default)]
pub struct QuickSearch {
    /// The current search query.
    query: String,
    /// Whether search mode is currently active (accepting input).
    active: bool,
    /// Indices of matching issues in the original list.
    matches: Vec<usize>,
    /// Index into the matches vector for current match.
    current_match: usize,
}

impl QuickSearch {
    /// Create a new quick search state.
    pub fn new() -> Self {
        Self {
            query: String::new(),
            active: false,
            matches: Vec::new(),
            current_match: 0,
        }
    }

    /// Activate search mode and clear the query.
    pub fn activate(&mut self) {
        self.active = true;
        self.query.clear();
        self.matches.clear();
        self.current_match = 0;
    }

    /// Deactivate search mode and clear all state.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.query.clear();
        self.matches.clear();
        self.current_match = 0;
    }

    /// Check if search mode is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Set active state without clearing.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Get the current search query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Check if the search query is empty.
    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
    }

    /// Push a character to the query.
    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
    }

    /// Remove the last character from the query.
    pub fn pop_char(&mut self) -> Option<char> {
        self.query.pop()
    }

    /// Update the list of matching issue indices.
    ///
    /// Performs case-insensitive matching against issue key, summary, and status.
    pub fn update_matches(&mut self, issues: &[Issue]) {
        if self.query.is_empty() {
            self.matches.clear();
            self.current_match = 0;
            return;
        }

        let query_lower = self.query.to_lowercase();
        self.matches = issues
            .iter()
            .enumerate()
            .filter(|(_, issue)| {
                issue.key.to_lowercase().contains(&query_lower)
                    || issue.fields.summary.to_lowercase().contains(&query_lower)
                    || issue.fields.status.name.to_lowercase().contains(&query_lower)
            })
            .map(|(i, _)| i)
            .collect();

        // Reset current match if out of bounds
        if self.current_match >= self.matches.len() {
            self.current_match = 0;
        }
    }

    /// Move to the next match and return its index.
    pub fn next_match(&mut self) -> Option<usize> {
        if self.matches.is_empty() {
            return None;
        }
        self.current_match = (self.current_match + 1) % self.matches.len();
        Some(self.matches[self.current_match])
    }

    /// Move to the previous match and return its index.
    pub fn prev_match(&mut self) -> Option<usize> {
        if self.matches.is_empty() {
            return None;
        }
        if self.current_match == 0 {
            self.current_match = self.matches.len() - 1;
        } else {
            self.current_match -= 1;
        }
        Some(self.matches[self.current_match])
    }

    /// Get the index of the current match in the issues list.
    pub fn current_match_index(&self) -> Option<usize> {
        self.matches.get(self.current_match).copied()
    }

    /// Check if a given issue index is in the matches list.
    pub fn is_match(&self, index: usize) -> bool {
        self.matches.contains(&index)
    }

    /// Check if a given issue index is the current match.
    pub fn is_current_match(&self, index: usize) -> bool {
        self.current_match_index() == Some(index)
    }

    /// Get match count information as a formatted string.
    pub fn match_info(&self) -> String {
        if self.matches.is_empty() {
            "No matches".to_string()
        } else {
            format!("{}/{}", self.current_match + 1, self.matches.len())
        }
    }

    /// Get the number of matches.
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Get the list of matching indices.
    pub fn matches(&self) -> &[usize] {
        &self.matches
    }
}

/// Highlight matching text in a string with the search query.
///
/// Returns a Line with spans that have highlighted style for matches.
pub fn highlight_text(text: &str, query: &str) -> Line<'static> {
    if query.is_empty() {
        return Line::from(text.to_string());
    }

    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    let mut spans = Vec::new();
    let mut last_end = 0;

    for (start, _) in text_lower.match_indices(&query_lower) {
        // Add non-matching text before this match
        if start > last_end {
            spans.push(Span::raw(text[last_end..start].to_string()));
        }

        // Add highlighted match (use original case from text)
        spans.push(Span::styled(
            text[start..start + query.len()].to_string(),
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));

        last_end = start + query.len();
    }

    // Add remaining text
    if last_end < text.len() {
        spans.push(Span::raw(text[last_end..].to_string()));
    }

    if spans.is_empty() {
        Line::from(text.to_string())
    } else {
        Line::from(spans)
    }
}

/// Render the search bar at the given area.
pub fn render_search_bar(frame: &mut Frame, area: Rect, search: &QuickSearch) {
    // Only render if search is active or has a query
    if !search.active && search.query.is_empty() {
        return;
    }

    let style = if search.active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let search_text = if search.active {
        format!("/{}", search.query)
    } else {
        format!("/{} [{}]", search.query, search.match_info())
    };

    let widget = Paragraph::new(search_text).style(style);

    frame.render_widget(widget, area);

    // Show cursor when active
    if search.active {
        frame.set_cursor_position(Position::new(
            area.x + 1 + search.query.len() as u16,
            area.y,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{IssueFields, IssueType, Status};

    fn create_test_issue(key: &str, summary: &str, status: &str) -> Issue {
        Issue {
            id: "1".to_string(),
            key: key.to_string(),
            self_url: "https://example.com".to_string(),
            fields: IssueFields {
                summary: summary.to_string(),
                description: None,
                status: Status {
                    id: "1".to_string(),
                    name: status.to_string(),
                    status_category: None,
                },
                issuetype: IssueType {
                    id: "1".to_string(),
                    name: "Bug".to_string(),
                    subtask: false,
                    description: None,
                    icon_url: None,
                },
                priority: None,
                assignee: None,
                reporter: None,
                project: None,
                labels: vec![],
                components: vec![],
                created: None,
                updated: None,
                duedate: None,
                story_points: None,
            },
        }
    }

    #[test]
    fn test_quick_search_new() {
        let search = QuickSearch::new();
        assert!(!search.is_active());
        assert!(search.is_empty());
        assert_eq!(search.match_count(), 0);
    }

    #[test]
    fn test_quick_search_activate() {
        let mut search = QuickSearch::new();
        search.push_char('t');
        search.push_char('e');
        search.activate();

        assert!(search.is_active());
        assert!(search.is_empty()); // Query should be cleared
    }

    #[test]
    fn test_quick_search_deactivate() {
        let mut search = QuickSearch::new();
        search.activate();
        search.push_char('t');
        search.deactivate();

        assert!(!search.is_active());
        assert!(search.is_empty());
    }

    #[test]
    fn test_quick_search_push_pop() {
        let mut search = QuickSearch::new();
        search.push_char('a');
        search.push_char('b');
        search.push_char('c');

        assert_eq!(search.query(), "abc");

        assert_eq!(search.pop_char(), Some('c'));
        assert_eq!(search.query(), "ab");

        assert_eq!(search.pop_char(), Some('b'));
        assert_eq!(search.pop_char(), Some('a'));
        assert_eq!(search.pop_char(), None);
        assert!(search.is_empty());
    }

    #[test]
    fn test_update_matches_empty_query() {
        let mut search = QuickSearch::new();
        let issues = vec![create_test_issue("TEST-1", "First issue", "Open")];

        search.update_matches(&issues);
        assert!(search.matches().is_empty());
    }

    #[test]
    fn test_update_matches_by_key() {
        let mut search = QuickSearch::new();
        let issues = vec![
            create_test_issue("TEST-1", "First issue", "Open"),
            create_test_issue("TEST-2", "Second issue", "Done"),
            create_test_issue("PROJ-1", "Third issue", "Open"),
        ];

        search.push_char('T');
        search.push_char('E');
        search.push_char('S');
        search.push_char('T');
        search.update_matches(&issues);

        assert_eq!(search.match_count(), 2);
        assert!(search.is_match(0));
        assert!(search.is_match(1));
        assert!(!search.is_match(2));
    }

    #[test]
    fn test_update_matches_by_summary() {
        let mut search = QuickSearch::new();
        let issues = vec![
            create_test_issue("TEST-1", "Fix the login bug", "Open"),
            create_test_issue("TEST-2", "Add new feature", "Done"),
            create_test_issue("TEST-3", "Login page redesign", "Open"),
        ];

        // Search for "login" - case insensitive
        search.push_char('l');
        search.push_char('o');
        search.push_char('g');
        search.push_char('i');
        search.push_char('n');
        search.update_matches(&issues);

        assert_eq!(search.match_count(), 2);
        assert!(search.is_match(0));
        assert!(!search.is_match(1));
        assert!(search.is_match(2));
    }

    #[test]
    fn test_update_matches_by_status() {
        let mut search = QuickSearch::new();
        let issues = vec![
            create_test_issue("TEST-1", "First issue", "Open"),
            create_test_issue("TEST-2", "Second issue", "In Progress"),
            create_test_issue("TEST-3", "Third issue", "Done"),
        ];

        search.push_char('o');
        search.push_char('p');
        search.push_char('e');
        search.push_char('n');
        search.update_matches(&issues);

        assert_eq!(search.match_count(), 1);
        assert!(search.is_match(0));
    }

    #[test]
    fn test_update_matches_case_insensitive() {
        let mut search = QuickSearch::new();
        let issues = vec![
            create_test_issue("TEST-1", "Bug in LOGIN", "Open"),
            create_test_issue("TEST-2", "login issue", "Done"),
            create_test_issue("TEST-3", "LoGiN fix", "Open"),
        ];

        search.push_char('L');
        search.push_char('O');
        search.push_char('G');
        search.push_char('I');
        search.push_char('N');
        search.update_matches(&issues);

        assert_eq!(search.match_count(), 3);
    }

    #[test]
    fn test_next_match() {
        let mut search = QuickSearch::new();
        let issues = vec![
            create_test_issue("TEST-1", "First", "Open"),
            create_test_issue("TEST-2", "Second", "Open"),
            create_test_issue("TEST-3", "Third", "Open"),
        ];

        search.push_char('T');
        search.push_char('E');
        search.push_char('S');
        search.push_char('T');
        search.update_matches(&issues);

        // Current match starts at 0 (index 0)
        assert_eq!(search.current_match_index(), Some(0));

        // Next moves to index 1
        assert_eq!(search.next_match(), Some(1));
        assert_eq!(search.current_match_index(), Some(1));

        // Next moves to index 2
        assert_eq!(search.next_match(), Some(2));

        // Next wraps to index 0
        assert_eq!(search.next_match(), Some(0));
    }

    #[test]
    fn test_prev_match() {
        let mut search = QuickSearch::new();
        let issues = vec![
            create_test_issue("TEST-1", "First", "Open"),
            create_test_issue("TEST-2", "Second", "Open"),
            create_test_issue("TEST-3", "Third", "Open"),
        ];

        search.push_char('T');
        search.push_char('E');
        search.push_char('S');
        search.push_char('T');
        search.update_matches(&issues);

        // Current match starts at 0
        assert_eq!(search.current_match_index(), Some(0));

        // Prev wraps to last match (index 2)
        assert_eq!(search.prev_match(), Some(2));

        // Prev moves to index 1
        assert_eq!(search.prev_match(), Some(1));

        // Prev moves to index 0
        assert_eq!(search.prev_match(), Some(0));
    }

    #[test]
    fn test_next_prev_empty_matches() {
        let mut search = QuickSearch::new();
        search.push_char('x');
        search.push_char('y');
        search.push_char('z');

        let issues = vec![create_test_issue("TEST-1", "First", "Open")];
        search.update_matches(&issues);

        assert_eq!(search.next_match(), None);
        assert_eq!(search.prev_match(), None);
        assert_eq!(search.current_match_index(), None);
    }

    #[test]
    fn test_is_current_match() {
        let mut search = QuickSearch::new();
        let issues = vec![
            create_test_issue("TEST-1", "First", "Open"),
            create_test_issue("TEST-2", "Second", "Open"),
        ];

        search.push_char('T');
        search.push_char('E');
        search.push_char('S');
        search.push_char('T');
        search.update_matches(&issues);

        assert!(search.is_current_match(0));
        assert!(!search.is_current_match(1));

        search.next_match();
        assert!(!search.is_current_match(0));
        assert!(search.is_current_match(1));
    }

    #[test]
    fn test_match_info() {
        let mut search = QuickSearch::new();
        assert_eq!(search.match_info(), "No matches");

        let issues = vec![
            create_test_issue("TEST-1", "First", "Open"),
            create_test_issue("TEST-2", "Second", "Open"),
            create_test_issue("TEST-3", "Third", "Open"),
        ];

        search.push_char('T');
        search.push_char('E');
        search.push_char('S');
        search.push_char('T');
        search.update_matches(&issues);

        assert_eq!(search.match_info(), "1/3");

        search.next_match();
        assert_eq!(search.match_info(), "2/3");

        search.next_match();
        assert_eq!(search.match_info(), "3/3");
    }

    #[test]
    fn test_highlight_text_empty_query() {
        let line = highlight_text("Hello world", "");
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "Hello world");
    }

    #[test]
    fn test_highlight_text_no_match() {
        let line = highlight_text("Hello world", "xyz");
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "Hello world");
    }

    #[test]
    fn test_highlight_text_single_match() {
        let line = highlight_text("Hello world", "world");
        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "Hello ");
        assert_eq!(line.spans[1].content, "world");
        // Check that matched span has highlighting
        assert_eq!(line.spans[1].style.bg, Some(Color::Yellow));
        assert_eq!(line.spans[1].style.fg, Some(Color::Black));
    }

    #[test]
    fn test_highlight_text_case_insensitive() {
        let line = highlight_text("Hello WORLD", "world");
        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "Hello ");
        // Should preserve original case
        assert_eq!(line.spans[1].content, "WORLD");
    }

    #[test]
    fn test_highlight_text_multiple_matches() {
        let line = highlight_text("test one test two test", "test");
        // Should have: "test" | " one " | "test" | " two " | "test"
        assert_eq!(line.spans.len(), 5);
        assert_eq!(line.spans[0].content, "test");
        assert_eq!(line.spans[1].content, " one ");
        assert_eq!(line.spans[2].content, "test");
        assert_eq!(line.spans[3].content, " two ");
        assert_eq!(line.spans[4].content, "test");
    }

    #[test]
    fn test_highlight_text_at_start() {
        let line = highlight_text("test hello", "test");
        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "test");
        assert_eq!(line.spans[1].content, " hello");
    }

    #[test]
    fn test_highlight_text_at_end() {
        let line = highlight_text("hello test", "test");
        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "hello ");
        assert_eq!(line.spans[1].content, "test");
    }

    #[test]
    fn test_current_match_resets_when_out_of_bounds() {
        let mut search = QuickSearch::new();
        let issues = vec![
            create_test_issue("TEST-1", "First", "Open"),
            create_test_issue("TEST-2", "Second", "Open"),
            create_test_issue("TEST-3", "Third", "Open"),
        ];

        search.push_char('T');
        search.push_char('E');
        search.push_char('S');
        search.push_char('T');
        search.update_matches(&issues);

        // Move to last match
        search.next_match();
        search.next_match();
        assert_eq!(search.current_match_index(), Some(2));

        // Now search for something with fewer matches
        search.push_char('-');
        search.push_char('1');
        search.update_matches(&issues);

        // Should have reset to 0
        assert_eq!(search.match_count(), 1);
        assert_eq!(search.current_match_index(), Some(0));
    }
}
