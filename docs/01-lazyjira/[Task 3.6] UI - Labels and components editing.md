# Task 3.6: Labels and Components Editing

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 3.6
**Area:** Frontend/UI
**Estimated Effort:** M (4-6 hours)

## Description

Implement UI for adding and removing labels and components from issues. Provide a tag-style interface with autocomplete for existing labels/components.

## Acceptance Criteria

- [ ] Labels editing via 'l' key in detail view
- [ ] Components editing via 'L' key
- [ ] Autocomplete for existing labels/components
- [ ] Add new label by typing
- [ ] Remove label with delete/backspace
- [ ] Visual tag display with colors
- [ ] Multiple selection support
- [ ] Immediate update to API
- [ ] Success feedback after change

## Implementation Details

### Approach

1. Create tag editor component with autocomplete
2. Implement label/component APIs
3. Reuse component for both labels and components
4. Handle add/remove operations

### Files to Modify/Create

- `src/ui/components/tag_editor.rs`: Reusable tag editor
- `src/ui/views/detail.rs`: Tag editor integration
- `src/api/client.rs`: Labels/components APIs

### Technical Specifications

**Tag Editor Component:**
```rust
pub struct TagEditor<T> {
    title: String,
    tags: Vec<T>,
    available: Vec<T>,
    filtered: Vec<T>,
    search_input: TextInput,
    selected_idx: usize,
    visible: bool,
    mode: TagEditorMode,
}

enum TagEditorMode {
    View,      // Viewing current tags
    Search,    // Searching/adding tags
}

pub trait Tag: Clone {
    fn id(&self) -> &str;
    fn display(&self) -> &str;
    fn matches(&self, query: &str) -> bool;
}

impl Tag for String {
    fn id(&self) -> &str { self }
    fn display(&self) -> &str { self }
    fn matches(&self, query: &str) -> bool {
        self.to_lowercase().contains(&query.to_lowercase())
    }
}

#[derive(Clone)]
pub struct Component {
    pub id: String,
    pub name: String,
}

impl Tag for Component {
    fn id(&self) -> &str { &self.id }
    fn display(&self) -> &str { &self.name }
    fn matches(&self, query: &str) -> bool {
        self.name.to_lowercase().contains(&query.to_lowercase())
    }
}

impl<T: Tag> TagEditor<T> {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            tags: Vec::new(),
            available: Vec::new(),
            filtered: Vec::new(),
            search_input: TextInput::new(),
            selected_idx: 0,
            visible: false,
            mode: TagEditorMode::View,
        }
    }

    pub fn show(&mut self, current_tags: Vec<T>, available_tags: Vec<T>) {
        self.tags = current_tags;
        self.available = available_tags;
        self.filtered = self.available.clone();
        self.visible = true;
        self.mode = TagEditorMode::View;
        self.selected_idx = 0;
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<TagAction<T>> {
        match self.mode {
            TagEditorMode::View => self.handle_view_input(key),
            TagEditorMode::Search => self.handle_search_input(key),
        }
    }

    fn handle_view_input(&mut self, key: KeyEvent) -> Option<TagAction<T>> {
        match key.code {
            KeyCode::Char('a') | KeyCode::Char('/') => {
                self.mode = TagEditorMode::Search;
                self.search_input.clear();
                self.update_filtered();
                None
            }
            KeyCode::Char('d') | KeyCode::Delete | KeyCode::Backspace => {
                if !self.tags.is_empty() && self.selected_idx < self.tags.len() {
                    let tag = self.tags.remove(self.selected_idx);
                    if self.selected_idx >= self.tags.len() && self.selected_idx > 0 {
                        self.selected_idx -= 1;
                    }
                    return Some(TagAction::Remove(tag));
                }
                None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_idx < self.tags.len().saturating_sub(1) {
                    self.selected_idx += 1;
                }
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
                None
            }
            KeyCode::Esc | KeyCode::Enter => {
                self.visible = false;
                Some(TagAction::Close)
            }
            _ => None,
        }
    }

    fn handle_search_input(&mut self, key: KeyEvent) -> Option<TagAction<T>> {
        match key.code {
            KeyCode::Enter => {
                if !self.filtered.is_empty() && self.selected_idx < self.filtered.len() {
                    let tag = self.filtered[self.selected_idx].clone();

                    // Check if already added
                    if !self.tags.iter().any(|t| t.id() == tag.id()) {
                        self.tags.push(tag.clone());
                        self.mode = TagEditorMode::View;
                        self.search_input.clear();
                        return Some(TagAction::Add(tag));
                    }
                }
                None
            }
            KeyCode::Esc => {
                self.mode = TagEditorMode::View;
                self.search_input.clear();
                None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_idx < self.filtered.len().saturating_sub(1) {
                    self.selected_idx += 1;
                }
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
                None
            }
            _ => {
                self.search_input.handle_input(key);
                self.update_filtered();
                self.selected_idx = 0;
                None
            }
        }
    }

    fn update_filtered(&mut self) {
        let query = self.search_input.value();
        if query.is_empty() {
            self.filtered = self.available.clone();
        } else {
            self.filtered = self.available.iter()
                .filter(|t| t.matches(query))
                .cloned()
                .collect();
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let dialog_area = centered_rect(area, 50, 60);
        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(&self.title)
            .borders(Borders::ALL);

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Current tags
                Constraint::Length(3),  // Search input (if in search mode)
                Constraint::Min(5),     // Available/filtered list
                Constraint::Length(1),  // Help text
            ])
            .split(inner);

        // Current tags
        self.render_current_tags(frame, chunks[0]);

        // Search input or separator
        match self.mode {
            TagEditorMode::Search => {
                let input = Paragraph::new(self.search_input.value())
                    .block(Block::default()
                        .title("Search/Add")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)));
                frame.render_widget(input, chunks[1]);

                frame.set_cursor_position(Position::new(
                    chunks[1].x + 1 + self.search_input.cursor() as u16,
                    chunks[1].y + 1,
                ));
            }
            TagEditorMode::View => {
                // Empty separator
            }
        }

        // Available tags list (in search mode)
        if self.mode == TagEditorMode::Search {
            self.render_filtered_list(frame, chunks[2]);
        }

        // Help text
        let help = match self.mode {
            TagEditorMode::View => "[a] add  [d] remove  [Enter/Esc] close",
            TagEditorMode::Search => "[Enter] select  [Esc] cancel  [j/k] navigate",
        };
        let help_widget = Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help_widget, chunks[3]);
    }

    fn render_current_tags(&self, frame: &mut Frame, area: Rect) {
        if self.tags.is_empty() {
            let empty = Paragraph::new("No tags. Press 'a' to add.")
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(empty, area);
            return;
        }

        let spans: Vec<Span> = self.tags.iter()
            .enumerate()
            .flat_map(|(i, tag)| {
                let style = if self.mode == TagEditorMode::View && i == self.selected_idx {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                };

                vec![
                    Span::styled(format!(" {} ", tag.display()), style),
                    Span::raw(" "),
                ]
            })
            .collect();

        let tags_line = Line::from(spans);
        let paragraph = Paragraph::new(tags_line)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    fn render_filtered_list(&self, frame: &mut Frame, area: Rect) {
        if self.filtered.is_empty() {
            let empty = Paragraph::new("No matching tags")
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(empty, area);
            return;
        }

        let items: Vec<ListItem> = self.filtered.iter()
            .enumerate()
            .map(|(i, tag)| {
                let already_added = self.tags.iter().any(|t| t.id() == tag.id());
                let suffix = if already_added { " (added)" } else { "" };
                let style = if already_added {
                    Style::default().fg(Color::Gray)
                } else {
                    Style::default()
                };
                ListItem::new(format!("{}{}", tag.display(), suffix)).style(style)
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(Some(self.selected_idx));

        frame.render_stateful_widget(list, area, &mut state);
    }
}

pub enum TagAction<T> {
    Add(T),
    Remove(T),
    Close,
}
```

**API Methods:**
```rust
impl JiraClient {
    pub async fn get_labels(&self) -> Result<Vec<String>> {
        let url = format!("{}/rest/api/3/label", self.base_url);
        let response: LabelsResponse = self.get(&url).await?;
        Ok(response.values)
    }

    pub async fn get_project_components(&self, project_key: &str) -> Result<Vec<Component>> {
        let url = format!(
            "{}/rest/api/3/project/{}/components",
            self.base_url, project_key
        );
        self.get(&url).await
    }
}
```

**Detail View Integration:**
```rust
impl IssueDetailView {
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<Action> {
        if self.label_editor.visible {
            return match self.label_editor.handle_input(key) {
                Some(TagAction::Add(label)) => {
                    Some(Action::AddLabel(self.issue.key.clone(), label))
                }
                Some(TagAction::Remove(label)) => {
                    Some(Action::RemoveLabel(self.issue.key.clone(), label))
                }
                Some(TagAction::Close) => None,
                None => None,
            };
        }

        match key.code {
            KeyCode::Char('l') => {
                Some(Action::LoadLabels) // Then show editor
            }
            KeyCode::Char('L') => {
                Some(Action::LoadComponents) // Then show editor
            }
            // ... other handlers
        }
    }
}
```

## Testing Requirements

- [ ] 'l' opens label editor
- [ ] 'L' opens component editor
- [ ] Current tags display correctly
- [ ] Search filters available tags
- [ ] Adding tag updates issue
- [ ] Removing tag updates issue
- [ ] Already-added tags marked
- [ ] Keyboard navigation works

## Dependencies

- **Prerequisite Tasks:** Task 1.7, Task 3.1
- **Blocks Tasks:** None
- **External:** JIRA labels/components API

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Tag editor is reusable
- [ ] Visual feedback is clear
- [ ] Changes persist correctly
- [ ] Autocomplete is responsive
