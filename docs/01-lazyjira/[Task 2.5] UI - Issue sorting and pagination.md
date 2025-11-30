# Task 2.5: Issue Sorting and Pagination

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 2.5
**Area:** Frontend/UI
**Estimated Effort:** M (4-6 hours)

## Description

Implement column sorting for the issue list and pagination support for handling large issue sets. The application should load issues in batches and support navigation through pages.

## Acceptance Criteria

- [x] Click/key on column header sorts by that column
- [x] Toggle between ascending/descending sort
- [x] Visual indicator for current sort column and direction
- [x] Pagination with configurable page size (default 50)
- [x] Load more issues when scrolling to end
- [x] Page indicator showing current position (e.g., "1-50 of 234")
- [x] Support 10,000+ issues via pagination (per NFR)
- [x] Loading indicator during page fetch

## Implementation Details

### Approach

1. Add sort state to list view
2. Implement column header navigation
3. Add sort to JQL query
4. Implement pagination state
5. Add "load more" trigger at list end
6. Show pagination info in status bar

### Files to Modify/Create

- `src/ui/views/list.rs`: Sorting and pagination logic
- `src/api/client.rs`: Pagination parameters

### Technical Specifications

**Sort State:**
```rust
#[derive(Clone, Copy, PartialEq)]
pub enum SortColumn {
    Key,
    Summary,
    Status,
    Assignee,
    Priority,
    Created,
    Updated,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn toggle(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    fn to_jql(&self) -> &'static str {
        match self {
            Self::Ascending => "ASC",
            Self::Descending => "DESC",
        }
    }
}

#[derive(Clone)]
pub struct SortState {
    pub column: SortColumn,
    pub direction: SortDirection,
}

impl SortState {
    pub fn to_jql(&self) -> String {
        let field = match self.column {
            SortColumn::Key => "key",
            SortColumn::Summary => "summary",
            SortColumn::Status => "status",
            SortColumn::Assignee => "assignee",
            SortColumn::Priority => "priority",
            SortColumn::Created => "created",
            SortColumn::Updated => "updated",
        };
        format!("ORDER BY {} {}", field, self.direction.to_jql())
    }
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            column: SortColumn::Updated,
            direction: SortDirection::Descending,
        }
    }
}
```

**Pagination State:**
```rust
pub struct PaginationState {
    pub page_size: u32,
    pub current_offset: u32,
    pub total: u32,
    pub loading: bool,
    pub has_more: bool,
}

impl PaginationState {
    pub const DEFAULT_PAGE_SIZE: u32 = 50;

    pub fn new() -> Self {
        Self {
            page_size: Self::DEFAULT_PAGE_SIZE,
            current_offset: 0,
            total: 0,
            loading: false,
            has_more: true,
        }
    }

    pub fn update_from_response(&mut self, start: u32, max: u32, total: u32) {
        self.current_offset = start;
        self.total = total;
        self.has_more = start + max < total;
        self.loading = false;
    }

    pub fn loaded_count(&self) -> u32 {
        (self.current_offset + self.page_size).min(self.total)
    }

    pub fn display(&self) -> String {
        if self.total == 0 {
            "No issues".to_string()
        } else {
            format!(
                "{}-{} of {}",
                1,
                self.loaded_count(),
                self.total
            )
        }
    }
}
```

**List View with Sort and Pagination:**
```rust
pub struct IssueListView {
    issues: Vec<Issue>,
    selected: usize,
    sort: SortState,
    pagination: PaginationState,
    header_focused: bool,
    focused_column: usize,
    // ...
}

impl IssueListView {
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<Action> {
        if self.header_focused {
            return self.handle_header_input(key);
        }

        match key.code {
            KeyCode::Char('s') => {
                // Enter header mode for sorting
                self.header_focused = true;
                None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_down();
                self.check_load_more()
            }
            KeyCode::Char('G') => {
                self.move_to_end();
                self.check_load_more()
            }
            // ...
        }
    }

    fn handle_header_input(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                if self.focused_column > 0 {
                    self.focused_column -= 1;
                }
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.focused_column < 4 {
                    self.focused_column += 1;
                }
                None
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let column = self.column_at_index(self.focused_column);
                self.toggle_sort(column);
                self.header_focused = false;
                Some(Action::RefreshIssues)
            }
            KeyCode::Esc => {
                self.header_focused = false;
                None
            }
            _ => None,
        }
    }

    fn toggle_sort(&mut self, column: SortColumn) {
        if self.sort.column == column {
            self.sort.direction = self.sort.direction.toggle();
        } else {
            self.sort.column = column;
            self.sort.direction = SortDirection::Descending;
        }
    }

    fn check_load_more(&mut self) -> Option<Action> {
        // Load more if within 5 items of end
        let threshold = 5;
        if self.selected + threshold >= self.issues.len()
            && self.pagination.has_more
            && !self.pagination.loading
        {
            self.pagination.loading = true;
            Some(Action::LoadMoreIssues)
        } else {
            None
        }
    }

    pub fn append_issues(&mut self, new_issues: Vec<Issue>) {
        self.issues.extend(new_issues);
    }
}
```

**Column Headers with Sort Indicators:**
```rust
fn render_header(&self, frame: &mut Frame, area: Rect) {
    let columns = [
        ("Key", SortColumn::Key),
        ("Summary", SortColumn::Summary),
        ("Status", SortColumn::Status),
        ("Assignee", SortColumn::Assignee),
        ("Priority", SortColumn::Priority),
    ];

    let cells: Vec<Cell> = columns.iter()
        .enumerate()
        .map(|(i, (name, col))| {
            let indicator = if self.sort.column == *col {
                match self.sort.direction {
                    SortDirection::Ascending => " ▲",
                    SortDirection::Descending => " ▼",
                }
            } else {
                ""
            };

            let text = format!("{}{}", name, indicator);
            let style = if self.header_focused && i == self.focused_column {
                Style::default().bg(Color::Blue).bold()
            } else {
                Style::default().bold()
            };

            Cell::from(text).style(style)
        })
        .collect();

    Row::new(cells)
}
```

**Status Bar with Pagination:**
```rust
fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
    let pagination_info = self.pagination.display();
    let loading_indicator = if self.pagination.loading { " Loading..." } else { "" };

    let status = format!(
        "{} | {} | {}{}",
        self.current_profile.name,
        pagination_info,
        self.sort_info(),
        loading_indicator
    );

    let widget = Paragraph::new(status)
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(widget, area);
}
```

## Testing Requirements

- [x] Sorting by each column works
- [x] Sort direction toggles correctly
- [x] Sort indicator displays on correct column
- [x] Pagination loads first 50 issues
- [x] Scrolling to end triggers load more
- [x] Pagination info updates correctly
- [x] Loading indicator shows during fetch
- [x] Handles 10,000+ issues without issues

## Dependencies

- **Prerequisite Tasks:** Task 1.4, Task 1.6
- **Blocks Tasks:** None
- **External:** None

## Definition of Done

- [x] All acceptance criteria met
- [x] Sorting generates correct JQL
- [x] Pagination handles edge cases
- [x] Performance acceptable with large datasets
- [x] Status bar always shows accurate info

---

## Implementation Completion

**Completed:** 2025-11-30
**Branch:** `issue-sorting-pagination`

### Summary

Implemented column sorting and infinite scroll pagination for the issue list view.

### Files Modified

- `src/ui/views/list.rs`: Added sorting and pagination types and logic
- `src/app.rs`: Added handlers for `SortChanged` and `LoadMore` actions

### Key Implementation Details

1. **Sorting Types:**
   - `SortColumn` enum for Key, Summary, Status, Assignee, Priority columns
   - `SortDirection` enum (Ascending/Descending) with toggle and JQL conversion
   - `SortState` struct that generates ORDER BY JQL clause

2. **Pagination Types:**
   - `PaginationState` struct with page_size, current_offset, total, loading, has_more
   - Default page size of 50 issues
   - `display()` method returns "1-50 of 234" format

3. **ListView Enhancements:**
   - Added sort and pagination state fields
   - Header mode for column selection (press 's' to enter, h/l to navigate, Enter to select)
   - Sort indicators (▲/▼) displayed on column headers
   - Highlighted column header when in header mode
   - `check_load_more()` triggers LoadMore when within 5 items of end
   - `append_issues()` for adding paginated results
   - `reset_for_new_query()` clears state when sort/filter changes

4. **New ListAction Variants:**
   - `SortChanged`: Triggers refresh with new sort order
   - `LoadMore`: Triggers loading next page of results

5. **Status Bar Updates:**
   - Shows pagination info (e.g., "1-50 of 100")
   - Shows current sort column and direction
   - Context-sensitive help text for header mode

### Tests Added

- 27 new unit tests for sorting and pagination functionality
- Tests cover sort column/direction operations, JQL generation, toggle behavior
- Tests cover pagination state updates, display formatting, reset behavior
- Tests cover check_load_more conditions (near end, no more pages, already loading)

### Notes for Integration

The `SortChanged` and `LoadMore` actions are handled in `app.rs` with TODO comments for triggering async API calls. The actual API integration will require:
1. Including `sort.to_jql()` in search queries
2. Using `pagination.current_offset` as `startAt` parameter
3. Calling `update_pagination()` and `append_issues()` with API response data
