# Task 2.6: Quick Search Within Loaded Issues

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 2.6
**Area:** Frontend/UI
**Estimated Effort:** S (2-4 hours)

## Description

Implement a quick search/filter feature that filters the currently loaded issues in real-time as the user types. This provides instant feedback without making API calls.

## Acceptance Criteria

- [x] Quick search activated with '/' key
- [x] Filter issues as user types (real-time)
- [x] Search across key, summary, and status
- [x] Case-insensitive matching
- [x] Highlight matched text in results
- [x] Show match count
- [x] 'n' and 'N' to jump to next/previous match
- [x] Escape clears search and shows all issues
- [x] Empty search shows all issues

## Implementation Details

### Approach

1. Add search state to list view
2. Implement real-time filtering logic
3. Add match highlighting in table
4. Implement next/previous match navigation
5. Show search bar at bottom of list

### Files to Modify/Create

- `src/ui/views/list.rs`: Quick search integration
- `src/ui/components/search_bar.rs`: Search input component

### Technical Specifications

**Search State:**
```rust
pub struct QuickSearch {
    query: String,
    active: bool,
    matches: Vec<usize>, // Indices of matching issues
    current_match: usize,
}

impl QuickSearch {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            active: false,
            matches: Vec::new(),
            current_match: 0,
        }
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.query.clear();
        self.matches.clear();
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.query.clear();
        self.matches.clear();
    }

    pub fn update_matches(&mut self, issues: &[Issue]) {
        if self.query.is_empty() {
            self.matches.clear();
            return;
        }

        let query_lower = self.query.to_lowercase();
        self.matches = issues.iter()
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

    pub fn next_match(&mut self) -> Option<usize> {
        if self.matches.is_empty() {
            return None;
        }
        self.current_match = (self.current_match + 1) % self.matches.len();
        Some(self.matches[self.current_match])
    }

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

    pub fn current_match_index(&self) -> Option<usize> {
        self.matches.get(self.current_match).copied()
    }

    pub fn is_match(&self, index: usize) -> bool {
        self.matches.contains(&index)
    }

    pub fn match_info(&self) -> String {
        if self.matches.is_empty() {
            "No matches".to_string()
        } else {
            format!("{}/{}", self.current_match + 1, self.matches.len())
        }
    }
}
```

**List View Integration:**
```rust
impl IssueListView {
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<Action> {
        if self.search.active {
            return self.handle_search_input(key);
        }

        match key.code {
            KeyCode::Char('/') => {
                self.search.activate();
                None
            }
            KeyCode::Char('n') if !self.search.query.is_empty() => {
                if let Some(idx) = self.search.next_match() {
                    self.selected = idx;
                    self.ensure_visible();
                }
                None
            }
            KeyCode::Char('N') if !self.search.query.is_empty() => {
                if let Some(idx) = self.search.prev_match() {
                    self.selected = idx;
                    self.ensure_visible();
                }
                None
            }
            // ... other handlers
        }
    }

    fn handle_search_input(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char(c) => {
                self.search.query.push(c);
                self.search.update_matches(&self.issues);
                // Jump to first match
                if let Some(idx) = self.search.current_match_index() {
                    self.selected = idx;
                    self.ensure_visible();
                }
                None
            }
            KeyCode::Backspace => {
                self.search.query.pop();
                self.search.update_matches(&self.issues);
                None
            }
            KeyCode::Enter => {
                // Keep search results but close input
                self.search.active = false;
                None
            }
            KeyCode::Esc => {
                self.search.deactivate();
                None
            }
            _ => None,
        }
    }

    fn visible_issues(&self) -> impl Iterator<Item = (usize, &Issue)> {
        if self.search.query.is_empty() {
            // Show all issues
            self.issues.iter().enumerate()
        } else {
            // Show only matching issues
            self.search.matches.iter()
                .filter_map(|&i| self.issues.get(i).map(|issue| (i, issue)))
        }
    }
}
```

**Text Highlighting:**
```rust
fn highlight_text(text: &str, query: &str) -> Line<'static> {
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

        // Add highlighted match
        spans.push(Span::styled(
            text[start..start + query.len()].to_string(),
            Style::default().bg(Color::Yellow).fg(Color::Black),
        ));

        last_end = start + query.len();
    }

    // Add remaining text
    if last_end < text.len() {
        spans.push(Span::raw(text[last_end..].to_string()));
    }

    Line::from(spans)
}
```

**Search Bar Rendering:**
```rust
fn render_search_bar(&self, frame: &mut Frame, area: Rect) {
    if !self.search.active && self.search.query.is_empty() {
        return;
    }

    let style = if self.search.active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let search_text = if self.search.active {
        format!("/{}", self.search.query)
    } else {
        format!("/{} [{}]", self.search.query, self.search.match_info())
    };

    let widget = Paragraph::new(search_text)
        .style(style);

    frame.render_widget(widget, area);

    if self.search.active {
        frame.set_cursor_position(Position::new(
            area.x + 1 + self.search.query.len() as u16,
            area.y,
        ));
    }
}
```

## Testing Requirements

- [x] '/' activates search mode
- [x] Typing filters issues in real-time
- [x] Case-insensitive matching works
- [x] 'n' jumps to next match
- [x] 'N' jumps to previous match
- [x] Escape clears search
- [x] Enter keeps filter but closes input
- [x] Match count displays correctly
- [x] Highlighting works correctly

## Dependencies

- **Prerequisite Tasks:** Task 1.6
- **Blocks Tasks:** None
- **External:** None

## Definition of Done

- [x] All acceptance criteria met
- [x] Search is responsive (no lag)
- [x] Highlighting is visually clear
- [x] Navigation wraps correctly
- [x] Works with 1000+ loaded issues

---

## Implementation Summary

**Completed:** 2025-11-30

### Files Modified/Created

- `src/ui/components/search_bar.rs` - New file containing `QuickSearch` struct and `highlight_text` function
- `src/ui/components/mod.rs` - Added export for search_bar module
- `src/ui/views/list.rs` - Integrated quick search with ListView
- `src/api/types.rs` - Fixed pre-existing test compilation issues

### Key Implementation Decisions

1. **Separate Search Component**: Created a standalone `QuickSearch` struct in `search_bar.rs` for reusability and separation of concerns
2. **Case-Insensitive Matching**: Search matches against key, summary, and status fields using lowercase comparison
3. **Highlight Preservation**: Text highlighting preserves original case while matching case-insensitively
4. **Search Bar Integration**: Search bar appears at the bottom of the list view when active or when a query exists
5. **Key Bindings Changed**: `/` now activates quick search (previously opened JQL input); `:` still opens JQL input

### Acceptance Criteria Status

- [x] Quick search activated with '/' key
- [x] Filter issues as user types (real-time)
- [x] Search across key, summary, and status
- [x] Case-insensitive matching
- [x] Highlight matched text in results (yellow background, black text, bold)
- [x] Show match count in status bar
- [x] 'n' and 'N' to jump to next/previous match
- [x] Escape clears search and shows all issues
- [x] Empty search shows all issues

### Test Coverage

- 50+ unit tests added covering:
  - QuickSearch struct functionality
  - Text highlighting
  - Search input handling
  - Navigation (n/N keys)
  - Search activation/deactivation
  - Match counting and navigation
