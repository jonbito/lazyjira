# Task 1.7: Issue Detail View (Read-Only)

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 1.7
**Area:** Frontend/UI
**Estimated Effort:** M (6-8 hours)

## Description

Implement the issue detail view displaying all issue fields in a readable format. This view is initially read-only, with editing capabilities added in Phase 3.

## Acceptance Criteria

- [x] Full issue display with all fields from API
- [x] Scrollable content for long descriptions
- [x] Markdown-style rendering for description (basic formatting)
- [x] Clear visual hierarchy (key, summary, metadata, description)
- [x] Labels and components displayed as tags
- [x] Linked issues list (if available) - Stubbed, will be enhanced when API provides linked issues
- [x] Subtasks list (if available) - Stubbed, will be enhanced when API provides subtasks
- [x] Keyboard navigation: q to go back, j/k to scroll
- [x] Status bar showing issue key and context

## Implementation Details

### Approach

1. Create `IssueDetailView` struct with issue data
2. Layout: Header (key, type), Summary, Metadata block, Description
3. Use Block widgets for section separation
4. Implement scrolling for description
5. Parse Atlassian Document Format to displayable text
6. Handle keyboard events for navigation

### Files to Modify/Create

- `src/ui/views/detail.rs`: Issue detail view implementation
- `src/ui/views/mod.rs`: Export detail view
- `src/app.rs`: Integrate detail view with app state

### Technical Specifications

**IssueDetailView Struct:**
```rust
pub struct IssueDetailView {
    issue: Issue,
    scroll: u16,
    max_scroll: u16,
}

impl IssueDetailView {
    pub fn new(issue: Issue) -> Self {
        Self {
            issue,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::GoBack),
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_down();
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_up();
                None
            }
            KeyCode::Char('e') => Some(Action::EditIssue), // Future
            KeyCode::Char('c') => Some(Action::AddComment), // Future
            _ => None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) { ... }
}
```

**Layout Structure:**
```
┌─────────────────────────────────────────────────────────┐
│ [Bug] PROJ-123                                    [q]   │
├─────────────────────────────────────────────────────────┤
│ Fix login timeout issue on slow connections             │
├─────────────────────────────────────────────────────────┤
│ Status: In Progress    Priority: High                   │
│ Assignee: John Doe     Reporter: Jane Smith             │
│ Created: 2024-01-15    Updated: 2024-01-16              │
│ Labels: [backend] [urgent]                              │
│ Components: [Authentication]                            │
├─────────────────────────────────────────────────────────┤
│ Description                                             │
│ ─────────────────────────────────────────────────────── │
│ When users are on slow connections (< 1Mbps), the      │
│ login request times out before the server responds.     │
│                                                         │
│ Steps to reproduce:                                     │
│ 1. Throttle network to 500kbps                         │
│ 2. Attempt login                                        │
│ 3. Observe timeout error                                │
│                                                         │
│ Expected: Login should succeed with longer timeout      │
│ Actual: Timeout error after 5 seconds                   │
├─────────────────────────────────────────────────────────┤
│ Linked Issues                                           │
│ • PROJ-100 blocks this issue                           │
│ • PROJ-125 is blocked by this issue                    │
├─────────────────────────────────────────────────────────┤
│ [j/k scroll] [q back] [e edit] [c comment]              │
└─────────────────────────────────────────────────────────┘
```

**Render Implementation:**
```rust
fn render(&mut self, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(6),  // Metadata
            Constraint::Min(10),    // Description (scrollable)
            Constraint::Length(1),  // Help bar
        ])
        .split(area);

    self.render_header(frame, chunks[0]);
    self.render_metadata(frame, chunks[1]);
    self.render_description(frame, chunks[2]);
    self.render_help_bar(frame, chunks[3]);
}

fn render_header(&self, frame: &mut Frame, area: Rect) {
    let issue_type = &self.issue.fields.issue_type.name;
    let key = &self.issue.key;
    let title = format!("[{}] {}", issue_type, key);

    let header = Paragraph::new(title)
        .style(Style::default().bold().fg(Color::Cyan))
        .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, area);
}

fn render_metadata(&self, frame: &mut Frame, area: Rect) {
    let fields = &self.issue.fields;
    let lines = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().bold()),
            Span::styled(&fields.status.name, status_style(&fields.status)),
            Span::raw("    "),
            Span::styled("Priority: ", Style::default().bold()),
            Span::raw(self.issue.priority_name()),
        ]),
        Line::from(vec![
            Span::styled("Assignee: ", Style::default().bold()),
            Span::raw(self.issue.assignee_name()),
            Span::raw("    "),
            Span::styled("Reporter: ", Style::default().bold()),
            Span::raw(fields.reporter.as_ref()
                .map(|u| u.display_name.as_str())
                .unwrap_or("Unknown")),
        ]),
        // ... more metadata lines
    ];

    let metadata = Paragraph::new(lines)
        .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(metadata, area);
}

fn render_description(&mut self, frame: &mut Frame, area: Rect) {
    let description = self.issue.fields.description
        .as_ref()
        .map(|d| d.to_plain_text())
        .unwrap_or_else(|| "No description".to_string());

    let paragraph = Paragraph::new(description)
        .block(Block::default()
            .title("Description")
            .borders(Borders::ALL))
        .scroll((self.scroll, 0))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}
```

**Labels as Tags:**
```rust
fn render_labels(&self, labels: &[String]) -> Line<'_> {
    let spans: Vec<Span> = labels.iter()
        .flat_map(|label| {
            vec![
                Span::styled(
                    format!(" {} ", label),
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                ),
                Span::raw(" "),
            ]
        })
        .collect();

    Line::from(spans)
}
```

## Testing Requirements

- [x] Detail view renders with sample issue
- [x] Long descriptions scroll correctly
- [x] j/k scroll at boundaries handled
- [x] q key returns to list view
- [x] Missing fields (null assignee) handled gracefully
- [x] Empty description shows placeholder
- [x] Labels render as colored tags
- [x] Layout adapts to terminal width

## Dependencies

- **Prerequisite Tasks:** Task 1.2, Task 1.5 (Issue types), Task 1.6 (for navigation integration)
- **Blocks Tasks:** Task 3.1, 3.2, 3.3, 3.4 (editing features)
- **External:** ratatui

## Definition of Done

- [x] All acceptance criteria met
- [x] Smooth scrolling experience
- [x] Readable on 80-column terminals
- [x] All issue metadata displayed
- [x] Navigation intuitive and documented

---

## Completion Summary

**Completed:** 2025-01-29
**Branch:** `issue-detail-view`
**Commit:** `8a26f62 feat(ui): Add issue detail view with read-only display`

### Files Modified

- `src/ui/views/detail.rs` - Complete implementation of `DetailView` (829 lines)
- `src/ui/views/mod.rs` - Export `DetailAction` and `DetailView`
- `src/ui/mod.rs` - Export `DetailAction` from UI module
- `src/app.rs` - Integrated detail view with app state, key handling, rendering

### Implementation Highlights

1. **DetailView struct** with `Option<Issue>` storage, scroll position tracking, and content/visible height management
2. **Layout structure**: Header (type icon + key) → Summary → Metadata → Description (scrollable)
3. **Metadata section** displays:
   - Status with color coding based on category (new=blue, in-progress=yellow, done=green)
   - Priority with appropriate styling (highest=bold red, high=red, medium=yellow, low=green)
   - Assignee and Reporter names
   - Project key
   - Created and Updated dates (formatted YYYY-MM-DD)
4. **Labels** displayed as blue tags with white text
5. **Components** displayed as magenta tags with white text
6. **Scrollable description** with word wrapping and scroll position indicator
7. **Keyboard navigation**:
   - `q` / `Esc` - Go back to list
   - `j` / `↓` - Scroll down
   - `k` / `↑` - Scroll up
   - `g` - Go to top
   - `G` - Go to bottom
   - `Ctrl+d` / `PageDown` - Page down
   - `Ctrl+u` / `PageUp` - Page up
   - `e` - Edit issue (stubbed for Phase 3)
   - `c` - Add comment (stubbed for Phase 3)
8. **Status bar** showing issue key and scroll position

### Tests Added

28 new tests for `DetailView`:
- View creation, issue setting/clearing
- Scroll operations (up, down, page navigation, boundary conditions)
- Keyboard input handling for all keybindings
- Date formatting helper
- Line count estimation for scroll calculation
- Full issue display with all fields
- Minimal issue display with missing optional fields

5 new tests for app integration:
- Opening issue detail from list
- Escape and 'q' navigation back to list
- Scroll operations in detail view context
- Detail view accessor methods

### Notes

- Linked issues and subtasks sections are ready in the layout but will be fully implemented when the API provides these relationships
- Edit and comment actions return `DetailAction` variants but are not yet implemented (Phase 3)
