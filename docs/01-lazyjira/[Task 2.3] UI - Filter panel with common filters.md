# Task 2.3: Filter Panel with Common Filters

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 2.3
**Area:** Frontend/UI
**Estimated Effort:** L (8-12 hours)

## Description

Implement a filter panel that allows users to filter issues by common criteria including status, assignee, project, labels, components, and sprint. The panel should be accessible via keyboard and support combining multiple filters.

## Acceptance Criteria

- [ ] Filter panel accessible via 'f' key
- [ ] Filter by status (multi-select)
- [ ] Filter by assignee (including "Assigned to me")
- [ ] Filter by project
- [ ] Filter by labels (multi-select)
- [ ] Filter by components (multi-select)
- [ ] Filter by sprint (including "Current sprint")
- [ ] Clear all filters option
- [ ] Active filter indicators in list view
- [ ] Filters persist during session
- [ ] Filters generate valid JQL

## Implementation Details

### Approach

1. Create FilterPanelView with filter sections
2. Fetch filter options from JIRA API (statuses, users, projects, etc.)
3. Implement multi-select component for each filter type
4. Build JQL query from selected filters
5. Add filter summary to list view header
6. Store filter state in app

### Files to Modify/Create

- `src/ui/views/filter.rs`: Filter panel view
- `src/ui/components/multiselect.rs`: Multi-select widget
- `src/api/client.rs`: Add API calls for filter options
- `src/api/types.rs`: Filter option types
- `src/app.rs`: Filter state management

### Technical Specifications

**Filter State:**
```rust
#[derive(Default, Clone)]
pub struct FilterState {
    pub statuses: Vec<String>,
    pub assignees: Vec<String>, // account IDs
    pub assignee_is_me: bool,
    pub project: Option<String>,
    pub labels: Vec<String>,
    pub components: Vec<String>,
    pub sprint: Option<SprintFilter>,
}

#[derive(Clone)]
pub enum SprintFilter {
    Current,
    Specific(String), // Sprint ID
}

impl FilterState {
    pub fn to_jql(&self, current_user: &str) -> String {
        let mut clauses = Vec::new();

        if !self.statuses.is_empty() {
            let statuses = self.statuses.iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ");
            clauses.push(format!("status IN ({})", statuses));
        }

        if self.assignee_is_me {
            clauses.push("assignee = currentUser()".to_string());
        } else if !self.assignees.is_empty() {
            let assignees = self.assignees.iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ");
            clauses.push(format!("assignee IN ({})", assignees));
        }

        if let Some(project) = &self.project {
            clauses.push(format!("project = \"{}\"", project));
        }

        if !self.labels.is_empty() {
            let labels = self.labels.iter()
                .map(|l| format!("\"{}\"", l))
                .collect::<Vec<_>>()
                .join(", ");
            clauses.push(format!("labels IN ({})", labels));
        }

        match &self.sprint {
            Some(SprintFilter::Current) => {
                clauses.push("sprint IN openSprints()".to_string());
            }
            Some(SprintFilter::Specific(id)) => {
                clauses.push(format!("sprint = {}", id));
            }
            None => {}
        }

        if clauses.is_empty() {
            String::new()
        } else {
            clauses.join(" AND ")
        }
    }

    pub fn is_empty(&self) -> bool {
        self.statuses.is_empty()
            && self.assignees.is_empty()
            && !self.assignee_is_me
            && self.project.is_none()
            && self.labels.is_empty()
            && self.components.is_empty()
            && self.sprint.is_none()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
```

**Filter Panel View:**
```rust
pub struct FilterPanelView {
    sections: Vec<FilterSection>,
    focused_section: usize,
    filter_options: FilterOptions,
    current_filter: FilterState,
}

struct FilterSection {
    title: String,
    filter_type: FilterType,
    options: Vec<FilterOption>,
    selected: HashSet<String>,
    expanded: bool,
}

enum FilterType {
    Status,
    Assignee,
    Project,
    Labels,
    Components,
    Sprint,
}

impl FilterPanelView {
    pub fn new(options: FilterOptions) -> Self {
        let sections = vec![
            FilterSection::new("Status", FilterType::Status, &options.statuses),
            FilterSection::new("Assignee", FilterType::Assignee, &options.users),
            FilterSection::new("Project", FilterType::Project, &options.projects),
            FilterSection::new("Labels", FilterType::Labels, &options.labels),
            FilterSection::new("Sprint", FilterType::Sprint, &options.sprints),
        ];

        Self {
            sections,
            focused_section: 0,
            filter_options: options,
            current_filter: FilterState::default(),
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.sections[self.focused_section].move_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.sections[self.focused_section].move_up();
            }
            KeyCode::Tab => self.next_section(),
            KeyCode::BackTab => self.prev_section(),
            KeyCode::Char(' ') => {
                self.sections[self.focused_section].toggle_selected();
            }
            KeyCode::Enter => {
                return Some(Action::ApplyFilters(self.build_filter_state()));
            }
            KeyCode::Char('c') => {
                self.clear_all();
            }
            KeyCode::Esc | KeyCode::Char('f') => {
                return Some(Action::CloseFilterPanel);
            }
            _ => {}
        }
        None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Filters")
            .borders(Borders::ALL);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into columns for each filter section
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // Status
                Constraint::Percentage(20), // Assignee
                Constraint::Percentage(20), // Project
                Constraint::Percentage(20), // Labels
                Constraint::Percentage(20), // Sprint
            ])
            .split(inner);

        for (i, section) in self.sections.iter().enumerate() {
            let is_focused = i == self.focused_section;
            section.render(frame, chunks[i], is_focused);
        }
    }
}
```

**Multi-Select Component:**
```rust
pub struct MultiSelect {
    items: Vec<SelectItem>,
    selected: HashSet<String>,
    cursor: usize,
    scroll: usize,
}

struct SelectItem {
    id: String,
    label: String,
}

impl MultiSelect {
    pub fn toggle_current(&mut self) {
        let item_id = &self.items[self.cursor].id;
        if self.selected.contains(item_id) {
            self.selected.remove(item_id);
        } else {
            self.selected.insert(item_id.clone());
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let visible_items = area.height as usize - 2; // Account for borders

        let items: Vec<ListItem> = self.items.iter()
            .skip(self.scroll)
            .take(visible_items)
            .enumerate()
            .map(|(i, item)| {
                let checkbox = if self.selected.contains(&item.id) {
                    "[x]"
                } else {
                    "[ ]"
                };
                let text = format!("{} {}", checkbox, item.label);
                ListItem::new(text)
            })
            .collect();

        let style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).border_style(style))
            .highlight_style(Style::default().bg(Color::DarkGray));

        let mut state = ListState::default();
        state.select(Some(self.cursor - self.scroll));

        frame.render_stateful_widget(list, area, &mut state);
    }
}
```

**API Calls for Filter Options:**
```rust
impl JiraClient {
    pub async fn get_statuses(&self) -> Result<Vec<Status>> {
        self.get("/rest/api/3/status").await
    }

    pub async fn get_projects(&self) -> Result<Vec<Project>> {
        self.get("/rest/api/3/project").await
    }

    pub async fn search_users(&self, query: &str) -> Result<Vec<User>> {
        self.get(&format!("/rest/api/3/user/search?query={}", query)).await
    }

    pub async fn get_labels(&self) -> Result<Vec<String>> {
        self.get("/rest/api/3/label").await
    }

    pub async fn get_sprints(&self, board_id: &str) -> Result<Vec<Sprint>> {
        self.get(&format!("/rest/agile/1.0/board/{}/sprint", board_id)).await
    }
}
```

**Filter Summary in List View:**
```rust
fn render_filter_summary(&self, frame: &mut Frame, area: Rect) {
    if self.filter_state.is_empty() {
        return;
    }

    let mut parts = Vec::new();

    if !self.filter_state.statuses.is_empty() {
        parts.push(format!("Status: {}", self.filter_state.statuses.join(", ")));
    }

    if self.filter_state.assignee_is_me {
        parts.push("Assigned to me".to_string());
    }

    // ... other filters

    let summary = parts.join(" | ");
    let widget = Paragraph::new(format!("Filters: {}", summary))
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(widget, area);
}
```

## Testing Requirements

- [ ] Filter panel opens with 'f' key
- [ ] All filter sections display options
- [ ] Space toggles selection
- [ ] Tab moves between sections
- [ ] Enter applies filters
- [ ] 'c' clears all filters
- [ ] JQL generation is correct
- [ ] Filters persist after closing panel
- [ ] "Assigned to me" works correctly

## Dependencies

- **Prerequisite Tasks:** Task 1.4, Task 1.6
- **Blocks Tasks:** Task 2.4 (JQL input)
- **External:** JIRA REST API for filter options

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Filters generate valid JQL
- [ ] UI is responsive and keyboard-navigable
- [ ] Filter options loaded from JIRA
- [ ] Multi-select works correctly
