# Task 1: External Editor Module

**Documentation:** [Feature: External Editor for JIRA Description]([Feature]%20External%20Editor%20for%20JIRA%20Description.md)
**Task Number:** 1
**Area:** CLI
**Estimated Effort:** M (2-3 hours)

## Description

Create the core external editor module that handles launching the user's preferred text editor with a temporary file containing the JIRA description content. This is the foundational component that all other tasks depend on.

## Acceptance Criteria

- [x] Editor is detected from `$EDITOR` environment variable
- [x] Falls back to `$VISUAL` if `$EDITOR` is not set
- [x] Falls back to `vi` if neither environment variable is set
- [x] Temporary file is created with pattern `lazyjira-{issue_key}-{pid}.md`
- [x] Content is written to temp file before editor launch
- [x] Content is read back after editor exits
- [x] Temp file is cleaned up after reading
- [x] Editor exit status is properly captured
- [x] Tests written and passing

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

- [x] Test editor detection with `$EDITOR` set
- [x] Test editor detection with only `$VISUAL` set
- [x] Test fallback to `vi` when neither is set
- [x] Test temp file creation and naming pattern
- [x] Test content writing and reading
- [x] Test cleanup after read
- [x] Test error handling for spawn failures

## Dependencies

- **Prerequisite Tasks:** None (foundation task)
- **Blocks Tasks:** Task 2, Task 3, Task 4
- **External:** `std::process::Command`, `std::env`, `std::fs`

## Definition of Done

- [x] All acceptance criteria met
- [x] Code follows project standards (cargo fmt, cargo clippy)
- [x] Unit tests passing
- [x] Documentation comments on public API
- [ ] Code reviewed and merged

## Implementation Completion

**Completed:** 2025-12-01
**Branch:** `external-editor-module`
**Commit:** d38e5c9

### Files Created/Modified

- `src/ui/components/external_editor.rs` - **Created** (466 lines)
  - `ExternalEditor` struct with `new()`, `with_editor()`, and `open()` methods
  - `ExternalEditResult` struct for returning edit results
  - `ExternalEditorError` enum with comprehensive error types
  - `get_editor()` public function for editor detection
  - Internal `create_temp_file()` and `read_and_cleanup()` helper functions
- `src/ui/components/mod.rs` - **Modified**
  - Added `mod external_editor;` declaration
  - Added public exports for `ExternalEditor`, `ExternalEditResult`, `ExternalEditorError`

### Test Coverage

18 unit tests covering:
- Editor detection from `$EDITOR` environment variable
- Editor detection from `$VISUAL` environment variable
- Fallback to `vi` when no environment variable set
- `$EDITOR` priority over `$VISUAL`
- Temporary file creation with correct naming pattern
- Content read-back and cleanup
- Change detection (`was_modified` flag)
- Error type display messages for all error variants
- Nonexistent editor spawn error handling

### Key Implementation Decisions

1. **Error handling**: Used `thiserror` for ergonomic error definitions, matching existing project patterns
2. **Cleanup strategy**: Temp file cleanup failure is logged as warning but not propagated as error (non-fatal)
3. **Change detection**: Simple string comparison between original and new content
4. **Process ID in filename**: Uses `std::process::id()` to avoid temp file collisions
