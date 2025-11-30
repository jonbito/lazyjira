# Task 3.4: Assignee and Priority Changes

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 3.4
**Area:** Frontend/UI
**Estimated Effort:** M (4-6 hours)

## Description

Implement UI for changing issue assignee with user search and priority selection. The assignee picker should support searching users by name, and the priority picker should show available priorities with visual indicators.

## Acceptance Criteria

- [ ] Assignee change via 'a' key in detail view
- [ ] User search with autocomplete
- [ ] "Assign to me" option
- [ ] "Unassign" option
- [ ] Priority change via 'P' key
- [ ] Visual priority indicators (colors)
- [ ] Immediate feedback after change
- [ ] Loading indicator during search
- [ ] Handle users without permission to assign

## Implementation Details

### Approach

1. Create assignee picker with search
2. Implement debounced user search
3. Create priority picker
4. Add visual indicators for priorities
5. Handle API updates

### Files to Modify/Create

- `src/ui/components/assignee_picker.rs`: User search and selection
- `src/ui/components/priority_picker.rs`: Priority selection
- `src/ui/views/detail.rs`: Picker integration

### Technical Specifications

**Assignee Picker:**
```rust
pub struct AssigneePicker {
    search_input: TextInput,
    results: Vec<User>,
    selected: usize,
    visible: bool,
    loading: bool,
    current_user: Option<User>,
}

impl AssigneePicker {
    pub fn new(current_user: Option<User>) -> Self {
        Self {
            search_input: TextInput::new(),
            results: Vec::new(),
            selected: 0,
            visible: false,
            loading: false,
            current_user,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.search_input.clear();
        self.results.clear();
        self.selected = 0;
    }

    pub fn set_results(&mut self, users: Vec<User>) {
        self.results = users;
        self.selected = 0;
        self.loading = false;
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<AssigneeAction> {
        match key.code {
            KeyCode::Char('m') if self.search_input.value().is_empty() => {
                // Assign to me
                if let Some(user) = &self.current_user {
                    self.visible = false;
                    return Some(AssigneeAction::Assign(user.account_id.clone()));
                }
                None
            }
            KeyCode::Char('u') if self.search_input.value().is_empty() => {
                // Unassign
                self.visible = false;
                Some(AssigneeAction::Unassign)
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.results.is_empty() && self.selected < self.results.len() - 1 {
                    self.selected += 1;
                }
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            KeyCode::Enter => {
                if !self.results.is_empty() {
                    let user = &self.results[self.selected];
                    self.visible = false;
                    return Some(AssigneeAction::Assign(user.account_id.clone()));
                }
                None
            }
            KeyCode::Esc => {
                self.visible = false;
                Some(AssigneeAction::Cancel)
            }
            KeyCode::Char(c) => {
                self.search_input.handle_input(key);
                // Trigger search (debounced in app layer)
                if self.search_input.value().len() >= 2 {
                    self.loading = true;
                    return Some(AssigneeAction::Search(self.search_input.value().to_string()));
                }
                None
            }
            KeyCode::Backspace => {
                self.search_input.handle_input(key);
                if self.search_input.value().len() >= 2 {
                    self.loading = true;
                    return Some(AssigneeAction::Search(self.search_input.value().to_string()));
                } else {
                    self.results.clear();
                }
                None
            }
            _ => None,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let dialog_area = centered_rect(area, 50, 60);
        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title("Change Assignee")
            .borders(Borders::ALL);

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search input
                Constraint::Length(2), // Shortcuts hint
                Constraint::Min(5),    // Results
            ])
            .split(inner);

        // Search input
        let input = Paragraph::new(self.search_input.value())
            .block(Block::default()
                .title("Search users")
                .borders(Borders::ALL));
        frame.render_widget(input, chunks[0]);

        // Shortcuts
        let shortcuts = Paragraph::new("[m] Assign to me  [u] Unassign")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(shortcuts, chunks[1]);

        // Results
        if self.loading {
            let loading = Paragraph::new("Searching...")
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(loading, chunks[2]);
        } else if self.results.is_empty() && !self.search_input.value().is_empty() {
            let empty = Paragraph::new("No users found")
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(empty, chunks[2]);
        } else {
            let items: Vec<ListItem> = self.results.iter()
                .map(|user| {
                    let text = format!("{} <{}>",
                        user.display_name,
                        user.email.as_deref().unwrap_or("")
                    );
                    ListItem::new(text)
                })
                .collect();

            let list = List::new(items)
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("> ");

            let mut state = ListState::default();
            state.select(Some(self.selected));

            frame.render_stateful_widget(list, chunks[2], &mut state);
        }

        // Cursor in search input
        frame.set_cursor_position(Position::new(
            chunks[0].x + 1 + self.search_input.cursor() as u16,
            chunks[0].y + 1,
        ));
    }
}

pub enum AssigneeAction {
    Assign(String),
    Unassign,
    Search(String),
    Cancel,
}
```

**Priority Picker:**
```rust
pub struct PriorityPicker {
    priorities: Vec<Priority>,
    selected: usize,
    current_priority: Option<String>,
    visible: bool,
}

impl PriorityPicker {
    pub fn new() -> Self {
        Self {
            priorities: Vec::new(),
            selected: 0,
            current_priority: None,
            visible: false,
        }
    }

    pub fn show(&mut self, priorities: Vec<Priority>, current: Option<&str>) {
        self.priorities = priorities;
        self.current_priority = current.map(String::from);
        self.visible = true;

        // Select current priority
        if let Some(current) = &self.current_priority {
            self.selected = self.priorities.iter()
                .position(|p| &p.id == current)
                .unwrap_or(0);
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<PriorityAction> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.priorities.len() - 1 {
                    self.selected += 1;
                }
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            KeyCode::Enter => {
                let priority = &self.priorities[self.selected];
                self.visible = false;
                Some(PriorityAction::Select(priority.id.clone()))
            }
            KeyCode::Esc => {
                self.visible = false;
                Some(PriorityAction::Cancel)
            }
            _ => None,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let dialog_area = centered_rect(area, 40, 50);
        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title("Change Priority")
            .borders(Borders::ALL);

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let items: Vec<ListItem> = self.priorities.iter()
            .map(|priority| {
                let icon = priority_icon(&priority.name);
                let style = priority_style(&priority.name);
                let marker = if Some(&priority.id) == self.current_priority.as_ref() {
                    " (current)"
                } else {
                    ""
                };
                ListItem::new(format!("{} {}{}", icon, priority.name, marker))
                    .style(style)
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(Some(self.selected));

        frame.render_stateful_widget(list, inner, &mut state);
    }
}

fn priority_icon(name: &str) -> &'static str {
    match name.to_lowercase().as_str() {
        "highest" | "blocker" => "⬆⬆",
        "high" | "critical" => "⬆",
        "medium" => "━",
        "low" => "⬇",
        "lowest" => "⬇⬇",
        _ => "•",
    }
}

fn priority_style(name: &str) -> Style {
    match name.to_lowercase().as_str() {
        "highest" | "blocker" => Style::default().fg(Color::Red).bold(),
        "high" | "critical" => Style::default().fg(Color::Red),
        "medium" => Style::default().fg(Color::Yellow),
        "low" => Style::default().fg(Color::Blue),
        "lowest" => Style::default().fg(Color::Gray),
        _ => Style::default(),
    }
}

pub enum PriorityAction {
    Select(String),
    Cancel,
}
```

**Detail View Integration:**
```rust
impl IssueDetailView {
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<Action> {
        // Check active pickers first
        if self.assignee_picker.visible {
            return match self.assignee_picker.handle_input(key) {
                Some(AssigneeAction::Assign(account_id)) => {
                    Some(Action::UpdateAssignee(self.issue.key.clone(), Some(account_id)))
                }
                Some(AssigneeAction::Unassign) => {
                    Some(Action::UpdateAssignee(self.issue.key.clone(), None))
                }
                Some(AssigneeAction::Search(query)) => {
                    Some(Action::SearchUsers(query))
                }
                Some(AssigneeAction::Cancel) | None => None,
            };
        }

        if self.priority_picker.visible {
            return match self.priority_picker.handle_input(key) {
                Some(PriorityAction::Select(id)) => {
                    Some(Action::UpdatePriority(self.issue.key.clone(), id))
                }
                Some(PriorityAction::Cancel) | None => None,
            };
        }

        match key.code {
            KeyCode::Char('a') => {
                self.assignee_picker.show();
                None
            }
            KeyCode::Char('P') => {
                Some(Action::LoadPriorities)
            }
            // ... other handlers
        }
    }

    pub fn set_priorities(&mut self, priorities: Vec<Priority>) {
        let current = self.issue.fields.priority.as_ref().map(|p| p.id.as_str());
        self.priority_picker.show(priorities, current);
    }

    pub fn set_user_search_results(&mut self, users: Vec<User>) {
        self.assignee_picker.set_results(users);
    }
}
```

## Testing Requirements

- [ ] 'a' opens assignee picker
- [ ] User search returns results
- [ ] "Assign to me" works
- [ ] "Unassign" works
- [ ] 'P' opens priority picker
- [ ] Priority selection works
- [ ] Visual indicators display correctly
- [ ] Changes persist after save

## Dependencies

- **Prerequisite Tasks:** Task 1.7, Task 3.1
- **Blocks Tasks:** None
- **External:** JIRA user search API

## Definition of Done

- [x] All acceptance criteria met
- [x] User search is responsive
- [x] Priority colors are accessible
- [x] Keyboard navigation smooth
- [x] Changes update local state

---

## Implementation Notes (Completed 2025-11-30)

### Files Created
- `src/ui/components/assignee_picker.rs`: AssigneePicker component with inline search/filter
- `src/ui/components/priority_picker.rs`: PriorityPicker component with color-coded priorities

### Files Modified
- `src/ui/components/mod.rs`: Added exports for new components
- `src/ui/views/detail.rs`: Integrated pickers, added key handlers ('a' and 'P'), new DetailAction variants
- `src/app.rs`: Added pending request fields, handler methods for assignee/priority operations
- `src/main.rs`: Added async handlers for API calls

### Key Implementation Decisions
1. **Inline filtering instead of API search**: AssigneePicker loads all assignable users upfront and filters locally for better UX
2. **Pre-selection**: PriorityPicker pre-selects the current priority when opened
3. **Unassign option**: Added "Unassigned" as the first option in AssigneePicker
4. **Color coding**: Priority colors follow JIRA conventions (red=highest, yellow=medium, green=low)
5. **Existing API methods**: Leveraged existing `update_assignee` and `update_priority` client methods

### Test Coverage
- 530 tests passing
- New tests for AssigneePicker: navigation, selection, filtering, cancel, loading states
- New tests for PriorityPicker: navigation, selection, cancel, pre-selection, color functions
