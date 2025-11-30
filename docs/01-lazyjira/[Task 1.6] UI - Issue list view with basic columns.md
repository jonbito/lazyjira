# Task 1.6: Issue List View with Basic Columns

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 1.6
**Area:** Frontend/UI
**Estimated Effort:** M (6-8 hours)

## Description

Implement the main issue list view using ratatui's Table widget. Display issues with key columns (Key, Summary, Status, Assignee, Priority) and support basic keyboard navigation.

## Acceptance Criteria

- [x] Table displaying issue list with columns: Key, Summary, Status, Assignee, Priority
- [x] Column headers with visual distinction
- [x] Row highlighting for selected item
- [x] Keyboard navigation: j/k (or arrow keys) to move, Enter to select
- [x] Visual indicators for issue priority (color coding)
- [x] Visual indicators for issue type (icon or prefix)
- [x] Status bar showing current profile and issue count
- [x] Loading indicator while fetching issues
- [x] Empty state when no issues found
- [x] Responsive layout adapting to terminal width

## Implementation Details

### Approach

1. Create `IssueListView` struct with state (selected index, issues)
2. Implement ratatui `Widget` or render method
3. Build table with styled columns
4. Handle keyboard events for navigation
5. Add color scheme for status categories
6. Implement status bar component
7. Add loading and empty states

### Files to Modify/Create

- `src/ui/views/list.rs`: Issue list view implementation
- `src/ui/views/mod.rs`: Export list view
- `src/ui/theme.rs`: Color definitions for statuses/priorities
- `src/ui/components/table.rs`: Reusable table helpers (optional)
- `src/app.rs`: Integrate list view with app state

### Technical Specifications

**IssueListView Struct:**
```rust
pub struct IssueListView {
    issues: Vec<Issue>,
    selected: usize,
    scroll_offset: usize,
    loading: bool,
}

impl IssueListView {
    pub fn new() -> Self { ... }

    pub fn set_issues(&mut self, issues: Vec<Issue>) { ... }

    pub fn selected_issue(&self) -> Option<&Issue> {
        self.issues.get(self.selected)
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::NONE) => {
                // Wait for second 'g'
            }
            KeyCode::Char('G') => self.move_to_end(),
            KeyCode::Enter => return Some(Action::OpenIssue(self.selected)),
            _ => {}
        }
        None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) { ... }
}
```

**Table Layout:**
```rust
fn render(&self, frame: &mut Frame, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Key").style(Style::default().bold()),
        Cell::from("Summary").style(Style::default().bold()),
        Cell::from("Status").style(Style::default().bold()),
        Cell::from("Assignee").style(Style::default().bold()),
        Cell::from("Priority").style(Style::default().bold()),
    ]);

    let rows: Vec<Row> = self.issues.iter().map(|issue| {
        Row::new(vec![
            Cell::from(issue.key.clone()),
            Cell::from(truncate(&issue.fields.summary, 50)),
            Cell::from(issue.fields.status.name.clone())
                .style(status_style(&issue.fields.status)),
            Cell::from(issue.assignee_name().to_string()),
            Cell::from(issue.priority_name().to_string())
                .style(priority_style(&issue.fields.priority)),
        ])
    }).collect();

    let widths = [
        Constraint::Length(12),    // Key: PROJ-1234
        Constraint::Min(30),       // Summary: flexible
        Constraint::Length(15),    // Status
        Constraint::Length(20),    // Assignee
        Constraint::Length(10),    // Priority
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(table, area, &mut self.table_state);
}
```

**Color Scheme (from theme.rs):**
```rust
pub fn status_style(status: &Status) -> Style {
    match status.category.as_ref().map(|c| c.key.as_str()) {
        Some("new") => Style::default().fg(Color::Blue),
        Some("indeterminate") => Style::default().fg(Color::Yellow),
        Some("done") => Style::default().fg(Color::Green),
        _ => Style::default(),
    }
}

pub fn priority_style(priority: &Option<Priority>) -> Style {
    match priority.as_ref().map(|p| p.name.as_str()) {
        Some("Highest") | Some("Blocker") => Style::default().fg(Color::Red).bold(),
        Some("High") | Some("Critical") => Style::default().fg(Color::Red),
        Some("Medium") => Style::default().fg(Color::Yellow),
        Some("Low") => Style::default().fg(Color::Green),
        Some("Lowest") => Style::default().fg(Color::Gray),
        _ => Style::default(),
    }
}
```

**Loading State:**
```rust
fn render_loading(&self, frame: &mut Frame, area: Rect) {
    let loading = Paragraph::new("Loading issues...")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    frame.render_widget(loading, centered_rect(area, 30, 3));
}
```

**Empty State:**
```rust
fn render_empty(&self, frame: &mut Frame, area: Rect) {
    let message = Paragraph::new("No issues found\n\nPress 'f' to change filters")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    frame.render_widget(message, area);
}
```

## Testing Requirements

- [x] Table renders with sample issues
- [x] Selected row highlights correctly
- [x] j/k navigation works at list boundaries
- [x] gg moves to first, G moves to last
- [x] Enter key triggers issue open action
- [x] Truncated summaries show ellipsis
- [x] Loading state displays spinner/message
- [x] Empty state displays helpful message

## Dependencies

- **Prerequisite Tasks:** Task 1.2 (app architecture), Task 1.5 (Issue types)
- **Blocks Tasks:** Task 1.7 (issue detail)
- **External:** ratatui

## Definition of Done

- [x] All acceptance criteria met
- [x] Responsive to different terminal sizes (80+ cols)
- [x] Color scheme consistent and accessible
- [x] Keyboard shortcuts documented in code
- [x] Performance: renders 100+ issues smoothly

## Completion Summary

### Implementation Date
2025-11-29

### Files Modified/Created
- `src/ui/theme.rs`: Added status and priority color styling functions, issue type prefix icons, and string truncation utility
- `src/ui/views/list.rs`: Complete rewrite implementing ListView with table rendering, navigation, loading/empty states, and status bar
- `src/ui/views/mod.rs`: Updated exports to include ListAction
- `src/ui/mod.rs`: Updated exports for new theme functions and ListAction
- `src/app.rs`: Integrated ListView with app state, added keyboard input delegation and view rendering

### Features Implemented
1. **Table Display**: Displays issues with Key, Summary, Status, Assignee, and Priority columns using ratatui Table widget
2. **Column Headers**: Bold white headers with visual distinction
3. **Row Highlighting**: Dark gray background for selected row with ">>" indicator
4. **Keyboard Navigation**:
   - j/↓: Move down
   - k/↑: Move up
   - gg: Go to first issue
   - G: Go to last issue
   - Ctrl+d/PageDown: Page down (10 items)
   - Ctrl+u/PageUp: Page up (10 items)
   - Enter: Open issue detail
   - r: Refresh
   - f: Open filter panel
5. **Visual Indicators**:
   - Status colors: Blue (new), Yellow (in progress), Green (done)
   - Priority colors: Red/bold (Highest), Red (High), Yellow (Medium), Green (Low), Gray (Lowest)
   - Issue type icons: 🐛 (Bug), 📖 (Story), ✓ (Task), ⚡ (Epic), └ (Sub-task)
6. **Status Bar**: Shows profile name, issue count, selected position, and keyboard hints
7. **Loading State**: Centered "Loading issues..." message
8. **Empty State**: "No issues found" message with hints
9. **Responsive Layout**: Summary column expands/contracts with terminal width

### Tests Added
- 20 unit tests for ListView (navigation, input handling, state management)
- 16 unit tests for theme functions (status/priority styles, truncation)
- 7 integration tests in app.rs for list view interaction

### Key Design Decisions
- Used ListAction enum for decoupled action handling between view and app
- Implemented vim-style "gg" navigation with pending state tracking
- Status bar is part of ListView for context-aware display
- Column widths are dynamically calculated based on terminal width
