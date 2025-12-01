# Task 2: Terminal State Management

**Documentation:** [Feature: External Editor for JIRA Description]([Feature]%20External%20Editor%20for%20JIRA%20Description.md)
**Task Number:** 2
**Area:** CLI
**Estimated Effort:** S (1-2 hours)

## Description

Implement terminal state management functions to properly suspend and resume the TUI when launching an external editor. This ensures the terminal is in the correct mode for the editor and restores the TUI state cleanly afterward without visual artifacts.

## Acceptance Criteria

- [ ] Terminal leaves alternate screen before editor launch
- [ ] Raw mode is disabled before editor launch
- [ ] Terminal re-enters alternate screen after editor exits
- [ ] Raw mode is re-enabled after editor exits
- [ ] Screen is cleared/redrawn after returning to TUI
- [ ] Terminal state is restored even if editor crashes or is killed
- [ ] No visual artifacts after returning from editor
- [ ] Tests written and passing

## Implementation Details

### Approach

1. Create terminal utility functions in `src/main.rs` or a new `src/terminal.rs` module
2. Implement `suspend_tui()` function to leave alternate screen and disable raw mode
3. Implement `resume_tui()` function to re-enter alternate screen and enable raw mode
4. Wrap these in a guard pattern or ensure they're always called in pairs
5. Add error handling to always attempt restoration even on failure

### Files to Modify/Create

- `src/main.rs`: **Modify** - Add terminal state management functions
- `src/terminal.rs`: **Create (optional)** - Separate module if main.rs gets too large

### Technical Specifications

```rust
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

/// Suspends TUI mode to allow external process to use terminal
pub fn suspend_tui(stdout: &mut Stdout) -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}

/// Resumes TUI mode after external process completes
pub fn resume_tui(stdout: &mut Stdout, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    terminal.clear()?;
    Ok(())
}
```

**Guard pattern for safety:**
```rust
struct TuiSuspendGuard<'a> {
    stdout: &'a mut Stdout,
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
}

impl<'a> Drop for TuiSuspendGuard<'a> {
    fn drop(&mut self) {
        let _ = resume_tui(self.stdout, self.terminal);
    }
}
```

## Testing Requirements

- [ ] Test terminal state transitions compile correctly
- [ ] Test guard pattern ensures restoration on panic
- [ ] Manual test: Launch editor, verify no artifacts on return
- [ ] Manual test: Kill editor process, verify terminal recovers

## Dependencies

- **Prerequisite Tasks:** Task 1 (needed to have something to integrate with)
- **Blocks Tasks:** Task 4
- **External:** `crossterm` crate (already in use)

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Code follows project standards (cargo fmt, cargo clippy)
- [ ] Manual testing confirms no visual artifacts
- [ ] Guard pattern or equivalent ensures robustness
- [ ] Code reviewed and merged
