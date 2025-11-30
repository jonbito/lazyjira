# Task 1.2: Application Architecture and Main Loop

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 1.2
**Area:** Infrastructure
**Estimated Effort:** M (4-8 hours)

## Description

Implement the core application architecture using The Elm Architecture (TEA) pattern as recommended in the PRD. Set up the main event loop, terminal initialization, and basic state management structure.

## Acceptance Criteria

- [ ] Main application struct (`App`) with state management
- [ ] Terminal initialization and cleanup (raw mode, alternate screen)
- [ ] Main event loop with proper frame rendering
- [ ] Graceful shutdown handling (Ctrl+C, panics)
- [ ] Basic TEA pattern: Model, Update, View separation
- [ ] Async runtime (tokio) integration with event loop
- [ ] Application launches and shows blank TUI
- [ ] Clean exit restores terminal state

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

- [ ] Application starts and displays empty TUI
- [ ] Pressing 'q' exits cleanly
- [ ] Ctrl+C exits cleanly
- [ ] Terminal state restored after exit
- [ ] Terminal state restored after panic
- [ ] Window resize is handled

## Dependencies

- **Prerequisite Tasks:** Task 1.1
- **Blocks Tasks:** Task 1.3, Task 1.4, Task 1.5, Task 1.6, Task 1.7
- **External:** ratatui, crossterm, tokio

## Definition of Done

- [ ] All acceptance criteria met
- [ ] TEA architecture properly implemented
- [ ] No terminal corruption on any exit path
- [ ] Startup time under 500ms
- [ ] Code reviewed and follows project conventions
