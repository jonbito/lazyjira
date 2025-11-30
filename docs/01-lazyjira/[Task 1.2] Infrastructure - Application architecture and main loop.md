# Task 1.2: Application Architecture and Main Loop

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 1.2
**Area:** Infrastructure
**Estimated Effort:** M (4-8 hours)

## Description

Implement the core application architecture using The Elm Architecture (TEA) pattern as recommended in the PRD. Set up the main event loop, terminal initialization, and basic state management structure.

## Acceptance Criteria

- [x] Main application struct (`App`) with state management
- [x] Terminal initialization and cleanup (raw mode, alternate screen)
- [x] Main event loop with proper frame rendering
- [x] Graceful shutdown handling (Ctrl+C, panics)
- [x] Basic TEA pattern: Model, Update, View separation
- [x] Async runtime (tokio) integration with event loop
- [x] Application launches and shows blank TUI
- [x] Clean exit restores terminal state

## Implementation Details

### Approach

1. Implement `main.rs` with tokio runtime and CLI parsing
2. Create `App` struct in `app.rs` with:
   - State enum for current view
   - Model holding application data
   - Methods for update (handle events) and view (render)
3. Set up terminal with crossterm:
   - Enable raw mode
   - Enter alternate screen
   - Enable mouse capture (optional)
4. Implement event loop:
   - Poll for crossterm events
   - Handle keyboard/resize events
   - Trigger state updates
   - Render frames at reasonable rate
5. Add panic hook to restore terminal on crash
6. Implement graceful shutdown

### Files to Modify/Create

- `src/main.rs`: Entry point, tokio runtime, CLI args, terminal setup
- `src/app.rs`: App struct, state management, TEA implementation
- `src/events/handler.rs`: Event polling and dispatch
- `src/events/mod.rs`: Event types and exports

### Technical Specifications

**App State Enum:**
```rust
pub enum AppState {
    Loading,
    IssueList,
    IssueDetail,
    ProfileSelect,
    FilterPanel,
    Help,
    Exiting,
}
```

**App Struct:**
```rust
pub struct App {
    state: AppState,
    should_quit: bool,
    // Future: issues, profiles, etc.
}

impl App {
    pub fn new() -> Self { ... }
    pub fn update(&mut self, event: Event) -> Result<()> { ... }
    pub fn view(&self, frame: &mut Frame) { ... }
}
```

**Main Loop Pattern:**
```rust
loop {
    terminal.draw(|f| app.view(f))?;

    if crossterm::event::poll(Duration::from_millis(100))? {
        let event = crossterm::event::read()?;
        app.update(event)?;
    }

    if app.should_quit {
        break;
    }
}
```

**Startup time target:** < 500ms (per NFR)

## Testing Requirements

- [x] Application starts and displays empty TUI
- [x] Pressing 'q' exits cleanly
- [x] Ctrl+C exits cleanly
- [x] Terminal state restored after exit
- [x] Terminal state restored after panic
- [x] Window resize is handled

## Dependencies

- **Prerequisite Tasks:** Task 1.1
- **Blocks Tasks:** Task 1.3, Task 1.4, Task 1.5, Task 1.6, Task 1.7
- **External:** ratatui, crossterm, tokio

## Definition of Done

- [x] All acceptance criteria met
- [x] TEA architecture properly implemented
- [x] No terminal corruption on any exit path
- [x] Startup time under 500ms (measured ~5ms)
- [x] Code reviewed and follows project conventions

## Completion Notes

### Implementation Summary

The core application architecture has been implemented following The Elm Architecture (TEA) pattern:

**Files Modified/Created:**
- `src/main.rs` - Entry point with tokio runtime, terminal setup, panic hook, and main event loop
- `src/app.rs` - App struct with Model (AppState), Update (event handling), and View (rendering) separation
- `src/events/mod.rs` - Event enum for keyboard, resize, tick, and quit events
- `src/events/handler.rs` - EventHandler for polling crossterm events with configurable tick rate

**Key Implementation Decisions:**
1. **TEA Pattern**: State changes flow through `App::update()` for predictable behavior
2. **Event Polling**: 100ms tick rate provides responsive UI while remaining efficient
3. **Panic Hook**: Custom panic hook restores terminal state before displaying panic message
4. **Graceful Shutdown**: Both 'q' key and Ctrl+C trigger clean exit with terminal restoration

**Test Coverage:**
- 12 unit tests covering state transitions, key handling, and event handler configuration
- All tests pass

**Performance:**
- Startup time: ~5ms (well under 500ms NFR target)
- Binary size (release): Optimized with all dependencies
