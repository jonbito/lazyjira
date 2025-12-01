# Task 5: Integration Tests

**Documentation:** [Feature: External Editor for JIRA Description]([Feature]%20External%20Editor%20for%20JIRA%20Description.md)
**Task Number:** 5
**Area:** Testing
**Estimated Effort:** M (2-3 hours)

## Description

Create comprehensive integration tests for the external editor feature. This includes unit tests for individual components and integration tests that verify the full workflow. Also includes documentation updates for the help screen.

## Acceptance Criteria

- [ ] Unit tests for ExternalEditor module pass
- [ ] Unit tests for terminal state management pass
- [ ] Unit tests for DetailView key handling pass
- [ ] Integration test for full workflow (mocked editor)
- [ ] Help screen documentation updated
- [ ] All tests pass with `cargo test`
- [ ] Feature manually verified with Vim

## Implementation Details

### Approach

1. Write unit tests for `ExternalEditor` struct methods
2. Write tests for editor detection logic
3. Write tests for temp file creation/cleanup
4. Write tests for DetailView key binding handling
5. Create integration test with mocked/stubbed editor
6. Update help screen with new key binding documentation
7. Perform manual end-to-end testing

### Files to Modify/Create

- `src/ui/components/external_editor.rs`: **Modify** - Add `#[cfg(test)]` module
- `src/ui/views/detail.rs`: **Modify** - Add tests for key handling
- `tests/external_editor_integration.rs`: **Create (optional)** - Integration tests
- `src/ui/views/help.rs` (or equivalent): **Modify** - Add `E` key documentation

### Technical Specifications

**Unit tests for ExternalEditor:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_editor_from_editor_var() {
        env::set_var("EDITOR", "nvim");
        env::remove_var("VISUAL");
        assert_eq!(get_editor(), "nvim");
    }

    #[test]
    fn test_get_editor_falls_back_to_visual() {
        env::remove_var("EDITOR");
        env::set_var("VISUAL", "code");
        assert_eq!(get_editor(), "code");
    }

    #[test]
    fn test_get_editor_falls_back_to_vi() {
        env::remove_var("EDITOR");
        env::remove_var("VISUAL");
        assert_eq!(get_editor(), "vi");
    }

    #[test]
    fn test_temp_file_naming() {
        let path = create_temp_file("PROJ-123", "test content").unwrap();
        assert!(path.to_string_lossy().contains("lazyjira-PROJ-123"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_content_roundtrip() {
        let content = "# Test\n\nSome description";
        let path = create_temp_file("TEST-1", content).unwrap();
        let read_back = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, read_back);
        std::fs::remove_file(path).ok();
    }
}
```

**Tests for DetailView:**
```rust
#[test]
fn test_shift_e_triggers_external_editor() {
    let mut view = DetailView::new();
    view.set_issue(mock_issue());

    let key = KeyEvent::new(KeyCode::Char('E'), KeyModifiers::SHIFT);
    let action = view.handle_input(key);

    assert!(matches!(action, Some(DetailAction::OpenExternalEditor(_))));
}

#[test]
fn test_shift_e_ignored_in_edit_mode() {
    let mut view = DetailView::new();
    view.set_issue(mock_issue());
    view.enter_edit_mode();

    let key = KeyEvent::new(KeyCode::Char('E'), KeyModifiers::SHIFT);
    let action = view.handle_input(key);

    assert!(!matches!(action, Some(DetailAction::OpenExternalEditor(_))));
}
```

**Help screen update:**
```
Detail View:
  e       Edit issue fields inline
  E       Open description in external editor ($EDITOR)
  Ctrl+S  Save changes to JIRA
  ...
```

## Testing Requirements

- [ ] All unit tests pass
- [ ] Tests cover edge cases (missing env vars, empty content, etc.)
- [ ] Integration test with mock editor verifies workflow
- [ ] Manual test checklist completed:
  - [ ] `E` opens Vim with description content
  - [ ] Save and exit in Vim returns to TUI with changes
  - [ ] Exit without save (`:q!`) returns with no changes
  - [ ] `Ctrl+S` after external edit saves to JIRA
  - [ ] No visual artifacts after returning
  - [ ] Works when `$EDITOR` is not set (falls back to vi)

## Dependencies

- **Prerequisite Tasks:** Task 1, Task 2, Task 3, Task 4
- **Blocks Tasks:** None
- **External:** None

## Definition of Done

- [ ] All acceptance criteria met
- [ ] All unit tests passing (`cargo test`)
- [ ] Code coverage for new code is reasonable
- [ ] Help screen updated with `E` key binding
- [ ] Manual testing checklist completed
- [ ] Code follows project standards (cargo fmt, cargo clippy)
- [ ] Code reviewed and merged
