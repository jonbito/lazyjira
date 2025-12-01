# Task 3: DetailView Integration

**Documentation:** [Feature: External Editor for JIRA Description]([Feature]%20External%20Editor%20for%20JIRA%20Description.md)
**Task Number:** 3
**Area:** Frontend
**Estimated Effort:** M (2-3 hours)

## Description

Integrate the external editor functionality into the DetailView component. Add the key binding for `E` (Shift+e) to trigger external editing, create the new `DetailAction::OpenExternalEditor` variant, and handle the flow of entering edit mode with content returned from the external editor.

## Acceptance Criteria

- [ ] `E` (Shift+e) key binding triggers external editor when viewing issue detail
- [ ] New `DetailAction::OpenExternalEditor(String)` action variant exists
- [ ] Action includes the issue key for temp file naming
- [ ] After external edit, view enters edit mode with updated content
- [ ] If content unchanged, no edit mode is triggered
- [ ] Key binding only works when not already in edit mode
- [ ] Help text updated to show `E` key binding
- [ ] Tests written and passing

## Implementation Details

### Approach

1. Add `OpenExternalEditor(String)` variant to `DetailAction` enum
2. Modify `DetailView::handle_input()` to detect `E` key press
3. Return the new action with issue key when triggered
4. Add method `DetailView::set_external_edit_content()` to populate edit state
5. Update help text to document the `E` key binding
6. Ensure `E` is ignored during edit mode to prevent conflicts

### Files to Modify/Create

- `src/ui/views/detail.rs`: **Modify** - Add key handling and new action
- `src/app.rs`: **Modify** - Handle new `DetailAction::OpenExternalEditor` variant

### Technical Specifications

**New action variant:**
```rust
pub enum DetailAction {
    // ... existing variants
    OpenExternalEditor(String), // issue_key
}
```

**Key handling in DetailView:**
```rust
fn handle_input(&mut self, key: KeyEvent) -> Option<DetailAction> {
    // Only allow external edit when not in edit mode
    if !self.is_editing() {
        if key.code == KeyCode::Char('E') && key.modifiers.contains(KeyModifiers::SHIFT) {
            if let Some(issue) = &self.issue {
                return Some(DetailAction::OpenExternalEditor(issue.key.clone()));
            }
        }
    }
    // ... rest of handling
}
```

**Method to receive external edit content:**
```rust
impl DetailView {
    /// Called after external editor returns with modified content
    pub fn set_external_edit_content(&mut self, content: String) {
        self.enter_edit_mode();
        self.set_description_content(content);
        self.mark_as_modified();
    }
}
```

**Help text addition:**
```
E       Open description in external editor ($EDITOR)
```

## Testing Requirements

- [ ] Test `E` key triggers `OpenExternalEditor` action
- [ ] Test action contains correct issue key
- [ ] Test `E` is ignored when already in edit mode
- [ ] Test `set_external_edit_content` properly enters edit mode
- [ ] Test help text includes new key binding

## Dependencies

- **Prerequisite Tasks:** Task 1 (external editor module must exist)
- **Blocks Tasks:** Task 4
- **External:** None

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Code follows project standards (cargo fmt, cargo clippy)
- [ ] Unit tests passing
- [ ] Key binding works as expected in manual testing
- [ ] Help screen updated with `E` key documentation
- [ ] Code reviewed and merged
