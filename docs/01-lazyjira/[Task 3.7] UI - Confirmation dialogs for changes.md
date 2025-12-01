# Task 3.7: Confirmation Dialogs for Changes

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 3.7
**Area:** Frontend/UI
**Estimated Effort:** S (2-4 hours)

## Description

Implement confirmation dialogs for destructive or significant actions to prevent accidental changes. This provides a safety net for important operations.

## Acceptance Criteria

- [x] Generic confirmation dialog component
- [x] Confirmation for discarding unsaved changes
- [x] Confirmation for status transitions (optional, configurable)
- [x] Confirmation for profile deletion (already existed via DeleteProfileDialog)
- [x] 'y' to confirm, 'n' or Esc to cancel
- [x] Clear action description in dialog
- [x] Keyboard focus on dialog when open
- [x] Visual distinction for destructive actions

## Implementation Details

### Approach

1. Create reusable confirmation dialog component
2. Integrate with actions requiring confirmation
3. Add configuration for optional confirmations
4. Style destructive actions differently

### Files to Modify/Create

- `src/ui/components/confirm_dialog.rs`: Confirmation dialog
- `src/app.rs`: Dialog state management
- `src/config/settings.rs`: Confirmation preferences

### Technical Specifications

**Confirmation Dialog:**
```rust
pub struct ConfirmDialog {
    title: String,
    message: String,
    confirm_text: String,
    cancel_text: String,
    destructive: bool,
    visible: bool,
    on_confirm: Option<Box<dyn FnOnce() -> Action>>,
}

impl ConfirmDialog {
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            confirm_text: "Yes".to_string(),
            cancel_text: "No".to_string(),
            destructive: false,
            visible: true,
            on_confirm: None,
        }
    }

    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self.confirm_text = "Delete".to_string();
        self
    }

    pub fn with_labels(mut self, confirm: &str, cancel: &str) -> Self {
        self.confirm_text = confirm.to_string();
        self.cancel_text = cancel.to_string();
        self
    }

    pub fn on_confirm<F>(mut self, action: F) -> Self
    where
        F: FnOnce() -> Action + 'static,
    {
        self.on_confirm = Some(Box::new(action));
        self
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<ConfirmResult> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                self.visible = false;
                Some(ConfirmResult::Confirmed)
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.visible = false;
                Some(ConfirmResult::Cancelled)
            }
            _ => None,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Dim background
        let overlay = Block::default()
            .style(Style::default().bg(Color::Black));
        frame.render_widget(overlay, area);

        // Dialog box
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 8;
        let dialog_area = centered_rect_fixed(area, dialog_width, dialog_height);

        frame.render_widget(Clear, dialog_area);

        let border_style = if self.destructive {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let block = Block::default()
            .title(&self.title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(2),    // Message
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Buttons
            ])
            .split(inner);

        // Message
        let message = Paragraph::new(&self.message)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        frame.render_widget(message, chunks[0]);

        // Buttons
        let confirm_style = if self.destructive {
            Style::default().fg(Color::Red).bold()
        } else {
            Style::default().fg(Color::Green).bold()
        };

        let buttons = Line::from(vec![
            Span::styled(
                format!("[y] {} ", self.confirm_text),
                confirm_style,
            ),
            Span::raw("  "),
            Span::styled(
                format!("[n] {}", self.cancel_text),
                Style::default().fg(Color::Gray),
            ),
        ]);

        let button_widget = Paragraph::new(buttons)
            .alignment(Alignment::Center);
        frame.render_widget(button_widget, chunks[2]);
    }
}

pub enum ConfirmResult {
    Confirmed,
    Cancelled,
}

fn centered_rect_fixed(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
```

**Common Confirmation Builders:**
```rust
impl ConfirmDialog {
    pub fn discard_changes() -> Self {
        Self::new(
            "Discard Changes?",
            "You have unsaved changes. Are you sure you want to discard them?"
        ).destructive()
    }

    pub fn delete_profile(name: &str) -> Self {
        Self::new(
            "Delete Profile?",
            &format!(
                "Are you sure you want to delete profile '{}'?\nThis will remove the saved credentials.",
                name
            )
        ).destructive()
    }

    pub fn clear_cache() -> Self {
        Self::new(
            "Clear Cache?",
            "This will delete all cached issue data. You'll need to fetch issues again."
        ).destructive()
    }

    pub fn transition_issue(from: &str, to: &str) -> Self {
        Self::new(
            "Change Status?",
            &format!("Move issue from '{}' to '{}'?", from, to)
        ).with_labels("Confirm", "Cancel")
    }
}
```

**App Integration:**
```rust
pub struct App {
    // ... other fields
    confirm_dialog: Option<(ConfirmDialog, PendingAction)>,
}

enum PendingAction {
    DiscardChanges,
    DeleteProfile(String),
    TransitionIssue(String, String),
    ClearCache,
}

impl App {
    pub fn show_confirm(&mut self, dialog: ConfirmDialog, action: PendingAction) {
        self.confirm_dialog = Some((dialog, action));
    }

    pub fn handle_confirm_result(&mut self, result: ConfirmResult) -> Option<Action> {
        let (_, pending) = self.confirm_dialog.take()?;

        match result {
            ConfirmResult::Confirmed => {
                match pending {
                    PendingAction::DiscardChanges => {
                        self.detail_view.cancel_edit();
                        None
                    }
                    PendingAction::DeleteProfile(name) => {
                        Some(Action::DeleteProfile(name))
                    }
                    PendingAction::TransitionIssue(key, transition_id) => {
                        Some(Action::ExecuteTransition(key, transition_id))
                    }
                    PendingAction::ClearCache => {
                        Some(Action::ClearCache)
                    }
                }
            }
            ConfirmResult::Cancelled => None,
        }
    }

    fn render(&self, frame: &mut Frame) {
        // Render main content first
        self.render_content(frame);

        // Render dialog on top if present
        if let Some((dialog, _)) = &self.confirm_dialog {
            dialog.render(frame, frame.area());
        }
    }
}
```

**Settings for Optional Confirmations:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    // ... other settings
    #[serde(default)]
    pub confirm_transitions: bool,
    #[serde(default = "default_true")]
    pub confirm_discard_changes: bool,
}

fn default_true() -> bool {
    true
}
```

## Testing Requirements

- [x] Dialog renders centered
- [x] 'y' confirms action
- [x] 'n' cancels action
- [x] Esc cancels action
- [x] Destructive dialogs show red styling
- [x] Message wraps correctly
- [x] Background is dimmed (via Clear widget)
- [x] Keyboard focus trapped in dialog

## Dependencies

- **Prerequisite Tasks:** Task 1.2
- **Blocks Tasks:** None
- **External:** None

## Definition of Done

- [x] All acceptance criteria met
- [x] Dialog component is reusable
- [x] Destructive actions clearly marked
- [x] Keyboard interaction works
- [x] Integration with all requiring actions

---

## Implementation Completed

**Date:** 2025-11-30

### Summary

Implemented confirmation dialogs for destructive and significant actions in LazyJira TUI. Enhanced the existing `ConfirmDialog` component with destructive action styling and integrated confirmations for status transitions and discarding unsaved changes.

### Files Modified

1. **`src/ui/components/modal.rs`**
   - Enhanced `ConfirmDialog` struct with `confirm_label`, `cancel_label`, and `destructive` fields
   - Added `show_destructive()` method for red-styled destructive dialogs
   - Added `show_with_labels()` method for custom button labels
   - Added `show_destructive_with_label()` method for custom destructive dialogs
   - Added `is_destructive()` accessor method
   - Updated render method to use red styling for destructive actions
   - Added comprehensive tests for new functionality

2. **`src/ui/components/transition_picker.rs`**
   - Updated `TransitionAction::Execute` to include transition name (for confirmation message)

3. **`src/ui/views/detail.rs`**
   - Updated `DetailAction::ExecuteTransition` to include transition name
   - Updated transition picker input handling to propagate transition name

4. **`src/config/settings.rs`**
   - Added `confirm_transitions` setting (default: false - immediate execution)
   - Added `confirm_discard_changes` setting (default: true)
   - Updated tests to cover new settings

5. **`src/app.rs`**
   - Added `transition_confirm_dialog` field for status change confirmations
   - Added `pending_transition_confirm` to track transitions awaiting confirmation
   - Added `request_transition_with_confirmation()` method
   - Added `confirm_transition()` and `cancel_transition_confirm()` methods
   - Updated discard dialog to use destructive styling
   - Added transition confirmation dialog rendering and input handling

6. **`src/config/mod.rs`**
   - Updated test to include new settings fields

### Key Implementation Decisions

1. **Used existing ConfirmDialog component** - Enhanced rather than creating a new component to maintain consistency
2. **Configurable confirmations** - `confirm_transitions` defaults to false for fast workflows, `confirm_discard_changes` defaults to true for safety
3. **Transition name in action** - Added transition name to `DetailAction::ExecuteTransition` and `TransitionAction::Execute` to show meaningful confirmation messages
4. **Destructive styling** - Red border and confirm button color for destructive actions, green for safe cancel option

### Test Coverage

- Added 9 new tests for `ConfirmDialog` in `modal.rs`:
  - `test_confirm_dialog_show_destructive`
  - `test_confirm_dialog_show_with_labels`
  - `test_confirm_dialog_show_destructive_with_label`
  - `test_confirm_dialog_normal_not_destructive`
  - `test_confirm_dialog_handle_input_y`
  - `test_confirm_dialog_handle_input_n`
  - `test_confirm_dialog_handle_input_esc`
  - `test_confirm_dialog_handle_input_enter_with_selection`
  - `test_confirm_dialog_handle_input_tab_toggles`

- Added `test_confirmation_settings` in `settings.rs`
- Updated existing tests for new signatures

### Configuration

Users can configure confirmation behavior in `~/.config/lazyjira/config.toml`:

```toml
# Show confirmation dialog for status transitions (default: false)
confirm_transitions = true

# Show confirmation dialog when discarding unsaved changes (default: true)
confirm_discard_changes = true
```
