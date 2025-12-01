# Task 3: DetailView Integration

**Documentation:** [Feature: External Editor for JIRA Description]([Feature]%20External%20Editor%20for%20JIRA%20Description.md)
**Task Number:** 3
**Area:** Frontend
**Estimated Effort:** M (2-3 hours)

## Description

Integrate the external editor functionality into the DetailView component. Add the key binding for `E` (Shift+e) to trigger external editing, create the new `DetailAction::OpenExternalEditor` variant, and handle the flow of entering edit mode with content returned from the external editor.

## Acceptance Criteria

- [x] `E` (Shift+e) key binding triggers external editor when viewing issue detail
- [x] New `DetailAction::OpenExternalEditor(String)` action variant exists
- [x] Action includes the issue key for temp file naming
- [x] After external edit, view enters edit mode with updated content
- [x] If content unchanged, no edit mode is triggered
- [x] Key binding only works when not already in edit mode
- [x] Help text updated to show `E` key binding
- [x] Tests written and passing

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

- [x] Test `E` key triggers `OpenExternalEditor` action
- [x] Test action contains correct issue key
- [x] Test `E` is ignored when already in edit mode
- [x] Test `set_external_edit_content` properly enters edit mode
- [x] Test help text includes new key binding

## Dependencies

- **Prerequisite Tasks:** Task 1 (external editor module must exist)
- **Blocks Tasks:** Task 4
- **External:** None

## Definition of Done

- [x] All acceptance criteria met
- [x] Code follows project standards (cargo fmt, cargo clippy)
- [x] Unit tests passing
- [ ] Key binding works as expected in manual testing
- [x] Help screen updated with `E` key documentation
- [ ] Code reviewed and merged

## Implementation Completion

**Completed:** 2025-12-01
**Branch:** `detail-view-external-editor-integration`

### Files Created/Modified

- `src/ui/views/detail.rs` - **Modified**
  - Added `DetailAction::OpenExternalEditor(String)` variant
  - Added `E` key binding handler in `handle_input()` (only works when not in edit mode)
  - Added `set_external_edit_content()` method for receiving external editor content
  - Updated status bar help text to show `E:ext-edit`
- `src/app.rs` - **Modified**
  - Added `pending_external_edit: Option<(String, String)>` field
  - Added handler for `DetailAction::OpenExternalEditor` that stores pending request
  - Added `take_pending_external_edit()` method for main loop consumption
- `src/ui/components/text_editor.rs` - **Modified**
  - Added `set_original_content()` method for proper change tracking with external editor content

### Test Coverage

8 new unit tests added:
- `test_shift_e_triggers_open_external_editor` - Verifies E key triggers action
- `test_shift_e_contains_correct_issue_key` - Verifies action contains correct issue key
- `test_shift_e_without_issue_does_nothing` - Verifies no action without issue
- `test_shift_e_ignored_in_edit_mode` - Verifies E is ignored during edit mode
- `test_set_external_edit_content_enters_edit_mode` - Verifies edit mode is entered
- `test_set_external_edit_content_focuses_description` - Verifies description field is focused
- `test_set_external_edit_content_marks_as_modified` - Verifies change tracking works
- `test_set_external_edit_content_without_issue` - Verifies no-op without issue

### Key Implementation Decisions

1. **Change tracking**: Used `TextEditor::set_original_content()` to compare against issue's original description, not external editor content
2. **Action payload**: Stores `(issue_key, description)` in pending_external_edit for main loop to use
3. **Focus behavior**: External edit content sets focus to Description field, not Summary
4. **Help text**: Added `E:ext-edit` to status bar (abbreviated due to space constraints)
