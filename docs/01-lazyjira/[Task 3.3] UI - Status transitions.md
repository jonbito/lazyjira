# Task 3.3: Status Transitions

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 3.3
**Area:** Frontend/UI
**Estimated Effort:** M (4-6 hours)

## Description

Implement a workflow-aware status transition UI that shows available transitions for an issue and allows users to change the issue status. Must respect JIRA workflow rules and handle required fields.

## Acceptance Criteria

- [ ] Transition menu activated with 't' key in detail view
- [ ] Shows only available transitions for current status
- [ ] Displays target status for each transition
- [ ] Handles required fields for transitions
- [ ] Success feedback after transition
- [ ] Updates local state after transition
- [ ] Keyboard navigation in transition menu
- [ ] Shows current status prominently

## Implementation Details

### Approach

1. Fetch available transitions from API
2. Create transition picker popup
3. Handle required field prompts
4. Execute transition
5. Update local cache and UI

### Files to Modify/Create

- `src/ui/views/detail.rs`: Transition trigger
- `src/ui/components/transition_picker.rs`: Transition selection
- `src/ui/components/transition_form.rs`: Required fields form

### Technical Specifications

**Transition Picker:**
```rust
pub struct TransitionPicker {
    transitions: Vec<Transition>,
    selected: usize,
    visible: bool,
    loading: bool,
}

impl TransitionPicker {
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
            selected: 0,
            visible: false,
            loading: false,
        }
    }

    pub fn show(&mut self, transitions: Vec<Transition>) {
        self.transitions = transitions;
        self.selected = 0;
        self.visible = true;
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<TransitionAction> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.transitions.len() - 1 {
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
                let transition = self.transitions[self.selected].clone();
                self.visible = false;

                if transition.has_required_fields() {
                    Some(TransitionAction::ShowFieldsForm(transition))
                } else {
                    Some(TransitionAction::Execute(transition.id.clone(), None))
                }
            }
            KeyCode::Esc => {
                self.visible = false;
                Some(TransitionAction::Cancel)
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
            .title("Change Status")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        if self.loading {
            let loading = Paragraph::new("Loading transitions...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading, inner);
            return;
        }

        let items: Vec<ListItem> = self.transitions.iter()
            .map(|t| {
                let style = status_category_style(&t.to.category);
                let text = format!(
                    "{} → {}",
                    t.name,
                    t.to.name
                );
                ListItem::new(text).style(style)
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

pub enum TransitionAction {
    Execute(String, Option<FieldUpdates>),
    ShowFieldsForm(Transition),
    Cancel,
}

impl Transition {
    fn has_required_fields(&self) -> bool {
        self.fields.values().any(|f| f.required)
    }
}
```

**Status Category Styling:**
```rust
fn status_category_style(category: &StatusCategory) -> Style {
    match category.key.as_str() {
        "new" => Style::default().fg(Color::Blue),
        "indeterminate" => Style::default().fg(Color::Yellow),
        "done" => Style::default().fg(Color::Green),
        _ => Style::default(),
    }
}
```

**Required Fields Form:**
```rust
pub struct TransitionFieldsForm {
    transition: Transition,
    fields: Vec<FieldInput>,
    focused: usize,
    visible: bool,
}

struct FieldInput {
    name: String,
    field_key: String,
    required: bool,
    value: TextInput,
}

impl TransitionFieldsForm {
    pub fn new(transition: Transition) -> Self {
        let fields: Vec<FieldInput> = transition.fields.iter()
            .filter(|(_, f)| f.required)
            .map(|(key, field)| FieldInput {
                name: field.name.clone(),
                field_key: key.clone(),
                required: field.required,
                value: TextInput::new(),
            })
            .collect();

        Self {
            transition,
            fields,
            focused: 0,
            visible: true,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<TransitionAction> {
        match key.code {
            KeyCode::Tab => {
                self.focused = (self.focused + 1) % self.fields.len();
                None
            }
            KeyCode::Enter => {
                if self.validate() {
                    Some(TransitionAction::Execute(
                        self.transition.id.clone(),
                        Some(self.to_field_updates()),
                    ))
                } else {
                    None
                }
            }
            KeyCode::Esc => {
                self.visible = false;
                Some(TransitionAction::Cancel)
            }
            _ => {
                self.fields[self.focused].value.handle_input(key);
                None
            }
        }
    }

    fn validate(&self) -> bool {
        self.fields.iter()
            .filter(|f| f.required)
            .all(|f| !f.value.value().is_empty())
    }

    fn to_field_updates(&self) -> FieldUpdates {
        // Convert field inputs to FieldUpdates
        // This is simplified - real implementation needs to handle
        // different field types
        FieldUpdates::default()
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let dialog_area = centered_rect(area, 60, 50);
        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(format!("Transition: {}", self.transition.name))
            .borders(Borders::ALL);

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                self.fields.iter()
                    .map(|_| Constraint::Length(3))
                    .chain(std::iter::once(Constraint::Min(1)))
                    .collect::<Vec<_>>()
            )
            .split(inner);

        for (i, field) in self.fields.iter().enumerate() {
            let is_focused = i == self.focused;
            let label = if field.required {
                format!("{}*:", field.name)
            } else {
                format!("{}:", field.name)
            };

            let style = if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let input = Paragraph::new(field.value.value())
                .style(style)
                .block(Block::default()
                    .title(label)
                    .borders(Borders::ALL)
                    .border_style(style));

            frame.render_widget(input, chunks[i]);
        }
    }
}
```

**Integration with Detail View:**
```rust
impl IssueDetailView {
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<Action> {
        // Check if transition picker is active
        if self.transition_picker.visible {
            return match self.transition_picker.handle_input(key) {
                Some(TransitionAction::Execute(id, fields)) => {
                    Some(Action::TransitionIssue(self.issue.key.clone(), id, fields))
                }
                Some(TransitionAction::ShowFieldsForm(transition)) => {
                    self.transition_form = Some(TransitionFieldsForm::new(transition));
                    None
                }
                Some(TransitionAction::Cancel) => None,
                None => None,
            };
        }

        match key.code {
            KeyCode::Char('t') => {
                // Load transitions and show picker
                self.transition_picker.loading = true;
                self.transition_picker.visible = true;
                Some(Action::LoadTransitions(self.issue.key.clone()))
            }
            // ... other handlers
        }
    }

    pub fn set_transitions(&mut self, transitions: Vec<Transition>) {
        self.transition_picker.show(transitions);
    }

    pub fn update_status(&mut self, new_status: Status) {
        self.issue.fields.status = new_status;
    }
}
```

## Testing Requirements

- [ ] 't' opens transition picker
- [ ] Only valid transitions shown
- [ ] Transition executes correctly
- [ ] Required fields form appears when needed
- [ ] Status updates after transition
- [ ] Error handling for failed transitions
- [ ] Keyboard navigation works
- [ ] Cancel closes picker

## Dependencies

- **Prerequisite Tasks:** Task 1.7, Task 3.1
- **Blocks Tasks:** None
- **External:** JIRA workflow rules

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Respects JIRA workflow rules
- [ ] Required fields handled properly
- [ ] UI feedback is clear
- [ ] Status updates immediately after transition
