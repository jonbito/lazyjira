# Feature: External Editor for JIRA Description

## Overview

Add the ability to edit JIRA issue descriptions using an external text editor (like Vim, Emacs, or any editor specified by the `$EDITOR` environment variable). This provides a more comfortable editing experience for users who prefer their familiar editor over the built-in TUI text editor.

## User Story

As a power user, I want to edit JIRA descriptions in my preferred text editor (Vim) so that I can leverage my editor's features, muscle memory, and productivity tools when writing detailed issue descriptions.

## Problem Statement

The current inline TextEditor component works for simple edits but lacks the full power of a dedicated text editor. Users accustomed to Vim, Emacs, or other editors may find the built-in editor limiting. This feature allows editing in the user's preferred external editor while seamlessly integrating with the TUI workflow.

## Acceptance Criteria

- [ ] Given the user is viewing an issue detail, when they press `E` (Shift+e), then the application suspends and opens the description in their `$EDITOR`
- [ ] Given `$EDITOR` is not set, when the user triggers external edit, then the application falls back to `vi` as default
- [ ] Given the user saves and exits their editor, when they return to LazyJira, then the updated description is shown and ready to be saved to JIRA
- [ ] Given the user exits their editor without saving (or content unchanged), when they return to LazyJira, then no changes are detected and the original description is preserved
- [ ] Given the user has made changes via external editor, when they press `Ctrl+S`, then the changes are submitted to JIRA
- [ ] Given the terminal state, when the editor opens and closes, then the TUI correctly suspends and resumes without visual artifacts

## Technical Requirements

### System Areas Affected

- [x] Frontend
- [ ] Backend
- [x] CLI
- [ ] Database
- [ ] Infrastructure

### Implementation Approach

1. **Create an external editor module** (`src/ui/components/external_editor.rs`):
   - Write description content to a temporary file
   - Detect editor from `$EDITOR` environment variable (fallback to `vi`)
   - Spawn editor process with the temp file
   - Wait for editor to exit
   - Read content back from temp file
   - Clean up temp file

2. **Modify terminal handling in main loop**:
   - Before spawning editor: leave alternate screen, disable raw mode
   - After editor exits: re-enter alternate screen, enable raw mode, force redraw

3. **Integrate with DetailView**:
   - Add new key binding `E` (Shift+e) for external editor
   - Add new `DetailAction::OpenExternalEditor`
   - Track pending external edit state
   - On return from editor, populate the edit state with new content

4. **Handle the async flow in main.rs**:
   - Detect pending external editor request
   - Perform terminal state transition
   - Launch editor synchronously (blocking)
   - Restore terminal state
   - Update view with result

### Key Components

- **`ExternalEditor`**: Utility struct for launching external editor with temp file management
- **`DetailAction::OpenExternalEditor`**: New action variant for triggering external edit
- **`App::take_pending_external_edit()`**: Method to retrieve pending external edit request
- **Terminal state management**: Functions to suspend/resume TUI mode

### Data Requirements

No new data models required. The feature uses the existing `IssueUpdateRequest` and `AtlassianDoc` types for submitting changes.

**Temporary file format:**
- Plain text file containing the description
- Located in system temp directory
- Named with pattern: `lazyjira-{issue_key}-{timestamp}.md`

## Dependencies

### Internal Dependencies

- `src/ui/views/detail.rs`: DetailView for integration
- `src/ui/components/text_editor.rs`: TextEditor for edit state population
- `src/main.rs`: Terminal state management
- `src/app.rs`: Action handling

### External Dependencies

- `std::env::var("EDITOR")`: For detecting user's preferred editor
- `std::process::Command`: For spawning editor process
- `tempfile` crate (or `std::fs` with temp dir): For temporary file management

## Success Criteria

### Definition of Done

- [ ] All acceptance criteria met
- [ ] Tests written and passing
- [ ] Documentation updated (help screen, CLAUDE.md if needed)
- [ ] Code reviewed and merged
- [ ] Feature deployed and verified

### Success Metrics

- External editor launches correctly with `$EDITOR` or fallback
- Terminal state correctly preserved (no visual artifacts on return)
- Changes from editor properly detected and ready for save
- Works correctly with Vim (as specified in testing requirements)

## Risk Assessment

### Technical Risks

- **Terminal state corruption**: If editor crashes or is killed, terminal may be left in bad state
  - *Mitigation*: Wrap editor spawn in try/catch, always attempt terminal restore in finally block

- **Platform differences**: Editor spawning may behave differently on Windows vs Unix
  - *Mitigation*: Focus on Unix/macOS support initially (darwin platform per env), document limitations

- **Long-running editor sessions**: User may leave editor open indefinitely
  - *Mitigation*: This is acceptable behavior - the app blocks until editor exits, similar to `git commit`

### Timeline Risks

- **Scope creep**: Adding too many editor features (syntax highlighting, preview, etc.)
  - *Mitigation*: Keep scope minimal - just spawn editor, read result

## Technical Notes

### Terminal State Transitions

```rust
// Before launching editor:
disable_raw_mode()?;
execute!(stdout(), LeaveAlternateScreen)?;

// Launch editor (blocking)
let status = Command::new(editor)
    .arg(&temp_path)
    .status()?;

// After editor exits:
enable_raw_mode()?;
execute!(stdout(), EnterAlternateScreen)?;
terminal.clear()?;
```

### Editor Detection Logic

```rust
fn get_editor() -> String {
    std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string())
}
```

### Key Binding

- `E` (Shift+e): Open external editor for description
- This complements existing `e` for inline edit mode
- User can still use `e` -> Tab to description -> inline edit if preferred

### Integration Points

1. **DetailView::handle_input()** - Add case for `KeyCode::Char('E')` with `KeyModifiers::SHIFT`
2. **DetailAction** - Add `OpenExternalEditor(String)` variant (issue key)
3. **App::update()** - Handle new action, store pending external edit
4. **main.rs run_app()** - Check for pending external edit, perform terminal transitions
5. **DetailView** - After external edit returns, enter edit mode with updated content

### Temporary File Handling

```rust
use std::fs;
use std::path::PathBuf;

fn create_temp_file(issue_key: &str, content: &str) -> std::io::Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let filename = format!("lazyjira-{}-{}.md", issue_key, std::process::id());
    let path = temp_dir.join(filename);
    fs::write(&path, content)?;
    Ok(path)
}

fn read_and_cleanup(path: &PathBuf) -> std::io::Result<String> {
    let content = fs::read_to_string(path)?;
    fs::remove_file(path)?;
    Ok(content)
}
```

## Task Breakdown

This feature has been broken down into the following implementation tasks:

### 1. Core Infrastructure

- [ ] [Task 1] CLI: External Editor Module - Create the core `ExternalEditor` struct with temp file management and editor detection

### 2. Terminal Management

- [ ] [Task 2] CLI: Terminal State Management - Implement suspend/resume functions for TUI mode transitions

### 3. View Integration

- [ ] [Task 3] Frontend: DetailView Integration - Add `E` key binding and `OpenExternalEditor` action variant

### 4. Main Loop Orchestration

- [ ] [Task 4] CLI: Main Loop Integration - Wire up all components in the main event loop

### 5. Testing & Documentation

- [ ] [Task 5] Testing: Integration Tests - Unit tests, integration tests, and help screen updates

**Total Tasks:** 5
**Estimated Effort:** M-L (8-12 hours total)
**Critical Path:** Task 1 → Task 2 → Task 3 → Task 4 → Task 5
**Implementation Order:** Follow task numbering sequence for optimal dependency flow
