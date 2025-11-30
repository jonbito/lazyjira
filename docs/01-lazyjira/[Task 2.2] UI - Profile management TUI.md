# Task 2.2: Profile Management TUI

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 2.2
**Area:** Frontend/UI
**Estimated Effort:** L (8-12 hours)

## Description

Implement a full TUI interface for managing JIRA profiles including adding, editing, and removing profiles. The interface should guide users through profile creation with validation.

## Acceptance Criteria

- [ ] Profile list view showing all configured profiles
- [ ] Add new profile wizard/form
- [ ] Edit existing profile
- [ ] Delete profile with confirmation
- [ ] Validate profile connection before saving
- [ ] Secure token input (hidden characters)
- [ ] Form validation with inline errors
- [ ] Keyboard-only navigation

## Implementation Details

### Approach

1. Create ProfileListView with profile cards
2. Implement ProfileFormView for add/edit
3. Add confirmation dialog for delete
4. Build text input component with masking
5. Integrate with keyring for token storage
6. Add connection validation step

### Files to Modify/Create

- `src/ui/views/profile.rs`: Profile list and form views
- `src/ui/components/input.rs`: Text input with password masking
- `src/ui/components/form.rs`: Form field helpers
- `src/config/profile.rs`: Profile CRUD operations

### Technical Specifications

**Profile List View:**
```rust
pub struct ProfileListView {
    profiles: Vec<ProfileSummary>,
    selected: usize,
}

struct ProfileSummary {
    name: String,
    url: String,
    email: String,
    is_default: bool,
}

impl ProfileListView {
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('a') => Some(Action::AddProfile),
            KeyCode::Char('e') | KeyCode::Enter => Some(Action::EditProfile(self.selected)),
            KeyCode::Char('d') => Some(Action::ConfirmDeleteProfile(self.selected)),
            KeyCode::Char('s') => Some(Action::SetDefaultProfile(self.selected)),
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::GoBack),
            _ => None,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Profiles")
            .borders(Borders::ALL);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        for (i, profile) in self.profiles.iter().enumerate() {
            let is_selected = i == self.selected;
            self.render_profile_card(frame, profile, is_selected, inner);
        }

        // Help bar
        let help = "[a]dd [e]dit [d]elete [s]et default [q]back";
        // ...
    }
}
```

**Profile Form View:**
```rust
pub struct ProfileFormView {
    mode: FormMode,
    fields: ProfileFormFields,
    focus: FormField,
    error: Option<String>,
    validating: bool,
}

enum FormMode {
    Add,
    Edit(String), // Original profile name
}

enum FormField {
    Name,
    Url,
    Email,
    Token,
    Submit,
}

struct ProfileFormFields {
    name: TextInput,
    url: TextInput,
    email: TextInput,
    token: TextInput, // Masked
}

impl ProfileFormView {
    pub fn new_add() -> Self { ... }
    pub fn new_edit(profile: &Profile, token: &str) -> Self { ... }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Tab => self.next_field(),
            KeyCode::BackTab => self.prev_field(),
            KeyCode::Enter if self.focus == FormField::Submit => {
                self.validate_and_submit()
            }
            KeyCode::Esc => Some(Action::CancelForm),
            _ => {
                // Delegate to focused input
                self.current_input().handle_input(key);
                None
            }
        }
    }

    fn validate_and_submit(&mut self) -> Option<Action> {
        // Clear previous errors
        self.error = None;

        // Validate fields
        if self.fields.name.value().is_empty() {
            self.error = Some("Profile name is required".to_string());
            return None;
        }

        if !self.fields.url.value().starts_with("https://") {
            self.error = Some("URL must start with https://".to_string());
            return None;
        }

        if !self.fields.email.value().contains('@') {
            self.error = Some("Invalid email address".to_string());
            return None;
        }

        if self.fields.token.value().is_empty() {
            self.error = Some("API token is required".to_string());
            return None;
        }

        // Trigger connection validation
        self.validating = true;
        Some(Action::ValidateProfile(self.to_profile()))
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let title = match &self.mode {
            FormMode::Add => "Add Profile",
            FormMode::Edit(name) => &format!("Edit Profile: {}", name),
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Name
                Constraint::Length(3), // URL
                Constraint::Length(3), // Email
                Constraint::Length(3), // Token
                Constraint::Length(1), // Error
                Constraint::Length(3), // Submit button
            ])
            .split(inner);

        self.render_field(frame, "Name:", &self.fields.name, chunks[0], self.focus == FormField::Name);
        self.render_field(frame, "URL:", &self.fields.url, chunks[1], self.focus == FormField::Url);
        self.render_field(frame, "Email:", &self.fields.email, chunks[2], self.focus == FormField::Email);
        self.render_masked_field(frame, "Token:", &self.fields.token, chunks[3], self.focus == FormField::Token);

        if let Some(error) = &self.error {
            let error_widget = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red));
            frame.render_widget(error_widget, chunks[4]);
        }

        self.render_submit_button(frame, chunks[5]);
    }
}
```

**Text Input with Masking:**
```rust
pub struct TextInput {
    value: String,
    cursor: usize,
    masked: bool,
}

impl TextInput {
    pub fn new() -> Self { ... }
    pub fn masked() -> Self {
        Self { value: String::new(), cursor: 0, masked: true }
    }

    pub fn handle_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor, c);
                self.cursor += 1;
            }
            KeyCode::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.value.remove(self.cursor);
            }
            KeyCode::Left if self.cursor > 0 => self.cursor -= 1,
            KeyCode::Right if self.cursor < self.value.len() => self.cursor += 1,
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.value.len(),
            _ => {}
        }
    }

    pub fn display_value(&self) -> String {
        if self.masked {
            "•".repeat(self.value.len())
        } else {
            self.value.clone()
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let display = self.display_value();
        let style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let input = Paragraph::new(display)
            .style(style)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(input, area);

        // Show cursor if focused
        if focused {
            frame.set_cursor_position(Position::new(
                area.x + 1 + self.cursor as u16,
                area.y + 1,
            ));
        }
    }
}
```

**Delete Confirmation:**
```rust
pub struct ConfirmDialog {
    message: String,
    confirm_text: String,
    visible: bool,
}

impl ConfirmDialog {
    pub fn delete_profile(name: &str) -> Self {
        Self {
            message: format!("Delete profile '{}'? This cannot be undone.", name),
            confirm_text: "Delete".to_string(),
            visible: true,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                self.visible = false;
                Some(true)
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.visible = false;
                Some(false)
            }
            _ => None,
        }
    }
}
```

## Testing Requirements

- [ ] Profile list shows all profiles
- [ ] Add profile form validates all fields
- [ ] Edit profile populates existing values
- [ ] Token field is masked
- [ ] Delete confirmation prevents accidents
- [ ] Connection validation catches bad credentials
- [ ] Tab navigation through form works
- [ ] Escape cancels form without saving

## Dependencies

- **Prerequisite Tasks:** Task 1.4, Task 2.1
- **Blocks Tasks:** None
- **External:** keyring (for token storage)

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Forms are accessible via keyboard only
- [ ] Token never displayed in plain text
- [ ] Validation errors are clear
- [ ] Connection validation works
