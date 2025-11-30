# Task 2.1: Multiple Profile Support with Switching

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 2.1
**Area:** Configuration
**Estimated Effort:** M (4-6 hours)

## Description

Extend the configuration system to support multiple JIRA profiles with seamless switching between them. Users should be able to work with different JIRA instances without reconfiguring.

## Acceptance Criteria

- [ ] Configuration supports array of profiles
- [ ] Default profile setting respected on startup
- [ ] Profile switching via keyboard shortcut (p)
- [ ] Active profile indicator in status bar
- [ ] Session data cleared on profile switch
- [ ] API client recreated for new profile
- [ ] Support at least 20 configured profiles (per NFR)
- [ ] Profile names must be unique

## Implementation Details

### Approach

1. Extend Config to handle multiple profiles
2. Add profile selection state to App
3. Create profile picker UI component
4. Implement profile switching logic
5. Handle session cleanup on switch
6. Update status bar to show current profile

### Files to Modify/Create

- `src/config/mod.rs`: Multi-profile handling
- `src/config/profile.rs`: Profile validation, uniqueness check
- `src/app.rs`: Profile switching state
- `src/ui/components/profile_picker.rs`: Profile selection popup
- `src/ui/views/list.rs`: Status bar update

### Technical Specifications

**Profile Selection State:**
```rust
pub struct App {
    config: Config,
    current_profile: Profile,
    client: Option<JiraClient>,
    // ... other fields
}

impl App {
    pub fn switch_profile(&mut self, profile_name: &str) -> Result<()> {
        let profile = self.config.profiles
            .iter()
            .find(|p| p.name == profile_name)
            .ok_or(ConfigError::ProfileNotFound(profile_name.to_string()))?
            .clone();

        // Clear session data
        self.issue_list.clear();
        self.client = None;

        // Set new profile
        self.current_profile = profile;

        // Client will be recreated on next API call
        tracing::info!(profile = profile_name, "Switched profile");
        Ok(())
    }

    pub async fn ensure_client(&mut self) -> Result<&JiraClient> {
        if self.client.is_none() {
            self.client = Some(JiraClient::new(&self.current_profile).await?);
        }
        Ok(self.client.as_ref().unwrap())
    }
}
```

**Profile Picker Component:**
```rust
pub struct ProfilePicker {
    profiles: Vec<String>,
    selected: usize,
    visible: bool,
}

impl ProfilePicker {
    pub fn show(&mut self, profiles: &[Profile], current: &str) {
        self.profiles = profiles.iter().map(|p| p.name.clone()).collect();
        self.selected = self.profiles.iter()
            .position(|n| n == current)
            .unwrap_or(0);
        self.visible = true;
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<ProfileAction> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Enter => {
                self.visible = false;
                Some(ProfileAction::Select(self.profiles[self.selected].clone()))
            }
            KeyCode::Esc => {
                self.visible = false;
                Some(ProfileAction::Cancel)
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

        let items: Vec<ListItem> = self.profiles.iter()
            .map(|name| ListItem::new(name.as_str()))
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title("Switch Profile")
                .borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Blue))
            .highlight_symbol("> ");

        frame.render_stateful_widget(
            list,
            dialog_area,
            &mut ListState::default().with_selected(Some(self.selected))
        );
    }
}
```

**Status Bar with Profile:**
```rust
fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
    let profile_text = format!("Profile: {}", self.current_profile.name);
    let issue_count = format!("Issues: {}", self.issue_list.len());
    let connection = if self.client.is_some() { "●" } else { "○" };

    let left = Span::styled(
        format!("{} {} | {}", connection, profile_text, issue_count),
        Style::default().fg(Color::Cyan)
    );

    let right = Span::styled(
        "[p]rofile [f]ilter [?]help",
        Style::default().fg(Color::DarkGray)
    );

    let status = Paragraph::new(Line::from(vec![left]))
        .block(Block::default().borders(Borders::TOP));

    frame.render_widget(status, area);
}
```

**Profile Uniqueness Validation:**
```rust
impl Config {
    pub fn validate(&self) -> Result<()> {
        let mut seen_names = HashSet::new();
        for profile in &self.profiles {
            if !seen_names.insert(&profile.name) {
                return Err(ConfigError::DuplicateProfile(profile.name.clone()));
            }
            profile.validate()?;
        }

        if self.profiles.len() > 20 {
            tracing::warn!("More than 20 profiles configured, may impact performance");
        }

        Ok(())
    }
}
```

## Testing Requirements

- [ ] Multiple profiles load from config
- [ ] Profile switching clears session data
- [ ] Profile picker shows all profiles
- [ ] j/k navigation in picker works
- [ ] Enter selects profile
- [ ] Esc cancels selection
- [ ] Status bar shows current profile
- [ ] Duplicate profile names rejected

## Dependencies

- **Prerequisite Tasks:** Task 1.3, Task 1.4 (API client)
- **Blocks Tasks:** Task 2.2 (Profile TUI management)
- **External:** None

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Profile switching is seamless
- [ ] No data leaks between profiles
- [ ] Status bar always accurate
- [ ] Handles 20+ profiles without issues
