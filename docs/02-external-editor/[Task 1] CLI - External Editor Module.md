# Task 1: External Editor Module

**Documentation:** [Feature: External Editor for JIRA Description]([Feature]%20External%20Editor%20for%20JIRA%20Description.md)
**Task Number:** 1
**Area:** CLI
**Estimated Effort:** M (2-3 hours)

## Description

Create the core external editor module that handles launching the user's preferred text editor with a temporary file containing the JIRA description content. This is the foundational component that all other tasks depend on.

## Acceptance Criteria

- [ ] Editor is detected from `$EDITOR` environment variable
- [ ] Falls back to `$VISUAL` if `$EDITOR` is not set
- [ ] Falls back to `vi` if neither environment variable is set
- [ ] Temporary file is created with pattern `lazyjira-{issue_key}-{pid}.md`
- [ ] Content is written to temp file before editor launch
- [ ] Content is read back after editor exits
- [ ] Temp file is cleaned up after reading
- [ ] Editor exit status is properly captured
- [ ] Tests written and passing

## Implementation Details

### Approach

1. Create new module at `src/ui/components/external_editor.rs`
2. Implement `ExternalEditor` struct with configuration
3. Add `get_editor()` function for editor detection
4. Add `create_temp_file()` function for temp file creation
5. Add `read_and_cleanup()` function for post-edit cleanup
6. Add `open()` method that orchestrates the full flow
7. Return a result indicating success/failure and new content

### Files to Modify/Create

- `src/ui/components/external_editor.rs`: **Create** - New module with ExternalEditor struct
- `src/ui/components/mod.rs`: **Modify** - Add `pub mod external_editor;`

### Technical Specifications

```rust
pub struct ExternalEditor {
    editor: String,
}

pub struct ExternalEditResult {
    pub content: String,
    pub was_modified: bool,
}

impl ExternalEditor {
    pub fn new() -> Self;
    pub fn open(&self, issue_key: &str, content: &str) -> Result<ExternalEditResult, ExternalEditorError>;
}

fn get_editor() -> String {
    std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string())
}
```

**Error types to handle:**
- `TempFileCreation`: Failed to create temp file
- `EditorSpawn`: Failed to spawn editor process
- `EditorExecution`: Editor exited with non-zero status
- `ContentRead`: Failed to read content back
- `Cleanup`: Failed to delete temp file (non-fatal warning)

## Testing Requirements

- [ ] Test editor detection with `$EDITOR` set
- [ ] Test editor detection with only `$VISUAL` set
- [ ] Test fallback to `vi` when neither is set
- [ ] Test temp file creation and naming pattern
- [ ] Test content writing and reading
- [ ] Test cleanup after read
- [ ] Test error handling for spawn failures

## Dependencies

- **Prerequisite Tasks:** None (foundation task)
- **Blocks Tasks:** Task 2, Task 3, Task 4
- **External:** `std::process::Command`, `std::env`, `std::fs`

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Code follows project standards (cargo fmt, cargo clippy)
- [ ] Unit tests passing
- [ ] Documentation comments on public API
- [ ] Code reviewed and merged
