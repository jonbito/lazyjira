# Task 4: Main Loop Integration

**Documentation:** [Feature: External Editor for JIRA Description]([Feature]%20External%20Editor%20for%20JIRA%20Description.md)
**Task Number:** 4
**Area:** CLI
**Estimated Effort:** M (2-3 hours)

## Description

Integrate all the components into the main event loop. Handle the `OpenExternalEditor` action from the App, perform terminal state transitions, launch the external editor synchronously, and update the view with the result. This is the orchestration task that ties everything together.

## Acceptance Criteria

- [ ] Main loop detects pending external editor request from App
- [ ] Terminal is properly suspended before editor launch
- [ ] External editor is launched synchronously (blocking)
- [ ] Terminal is properly resumed after editor exits
- [ ] New content is passed back to DetailView
- [ ] Unchanged content results in no edit mode activation
- [ ] Error states are handled gracefully with user notification
- [ ] Works correctly with Vim as the test editor
- [ ] Tests written and passing

## Implementation Details

### Approach

1. Add `pending_external_edit` field to `App` struct
2. Handle `DetailAction::OpenExternalEditor` in `App::update()` to set pending edit
3. Add `App::take_pending_external_edit()` method to retrieve and clear pending edit
4. In main loop, check for pending external edit after each update
5. When detected: suspend TUI → launch editor → resume TUI → update view
6. Handle errors by showing notification and continuing

### Files to Modify/Create

- `src/app.rs`: **Modify** - Add pending external edit state and methods
- `src/main.rs`: **Modify** - Add external editor orchestration in main loop

### Technical Specifications

**App struct additions:**
```rust
pub struct App {
    // ... existing fields
    pending_external_edit: Option<PendingExternalEdit>,
}

pub struct PendingExternalEdit {
    pub issue_key: String,
    pub current_content: String,
}

impl App {
    pub fn take_pending_external_edit(&mut self) -> Option<PendingExternalEdit> {
        self.pending_external_edit.take()
    }
}
```

**Main loop integration:**
```rust
// In run_app() main loop
loop {
    terminal.draw(|f| app.view(f))?;

    // Check for pending external edit
    if let Some(edit_request) = app.take_pending_external_edit() {
        // Suspend TUI
        suspend_tui(&mut stdout)?;

        // Launch editor
        let editor = ExternalEditor::new();
        let result = editor.open(&edit_request.issue_key, &edit_request.current_content);

        // Resume TUI
        resume_tui(&mut stdout, &mut terminal)?;

        // Handle result
        match result {
            Ok(edit_result) if edit_result.was_modified => {
                app.apply_external_edit_result(edit_result.content);
            }
            Ok(_) => {
                // No changes, do nothing
            }
            Err(e) => {
                app.show_notification(format!("Editor error: {}", e), NotificationType::Error);
            }
        }

        continue; // Skip normal event handling this iteration
    }

    // Normal event polling...
}
```

**Action handling in App::update():**
```rust
fn update(&mut self, action: AppAction) {
    match action {
        // ... existing handlers
        AppAction::Detail(DetailAction::OpenExternalEditor(issue_key)) => {
            if let Some(issue) = self.get_current_issue() {
                let content = issue.fields.description_text();
                self.pending_external_edit = Some(PendingExternalEdit {
                    issue_key,
                    current_content: content,
                });
            }
        }
    }
}
```

## Testing Requirements

- [ ] Test pending external edit state is set correctly
- [ ] Test `take_pending_external_edit` clears the pending state
- [ ] Manual test: Full flow with Vim
- [ ] Manual test: Editor exit without saving
- [ ] Manual test: Editor crash recovery
- [ ] Test error notification is shown on failure

## Dependencies

- **Prerequisite Tasks:** Task 1, Task 2, Task 3
- **Blocks Tasks:** Task 5 (integration testing)
- **External:** None

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Code follows project standards (cargo fmt, cargo clippy)
- [ ] Full integration works end-to-end
- [ ] Manual testing with Vim confirms correct behavior
- [ ] Terminal state always recovers correctly
- [ ] Code reviewed and merged
