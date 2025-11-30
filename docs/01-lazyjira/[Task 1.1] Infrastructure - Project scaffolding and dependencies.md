# Task 1.1: Project Scaffolding and Dependencies

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 1.1
**Area:** Infrastructure
**Estimated Effort:** S (2-4 hours)

## Description

Initialize the Rust project with Cargo, set up the project structure as defined in the PRD, and configure all required dependencies. This establishes the foundation for all subsequent development.

## Acceptance Criteria

- [ ] Cargo.toml created with all specified dependencies and correct versions
- [ ] Project directory structure matches PRD specification
- [ ] All module files created with proper `mod.rs` declarations
- [ ] Project compiles successfully with `cargo build`
- [ ] Basic `cargo test` runs without errors
- [ ] `.gitignore` configured for Rust projects
- [ ] README.md with basic project description

## Implementation Details

### Approach

1. Run `cargo new lazyjira` to initialize project
2. Configure Cargo.toml with dependencies from PRD:
   - ratatui ^0.28
   - crossterm ^0.28
   - terminput ^0.2
   - tokio ^1.0 (with full features)
   - reqwest ^0.12 (with json feature)
   - serde ^1.0 (with derive feature)
   - toml ^0.8
   - tracing ^0.1
   - tracing-subscriber ^0.3
   - dirs ^5.0
   - keyring ^2.0
   - thiserror ^1.0
   - clap ^4.0 (with derive feature)
3. Create directory structure:
   ```
   src/
   ├── main.rs
   ├── app.rs
   ├── config/
   │   ├── mod.rs
   │   ├── profile.rs
   │   └── settings.rs
   ├── api/
   │   ├── mod.rs
   │   ├── client.rs
   │   ├── types.rs
   │   └── auth.rs
   ├── ui/
   │   ├── mod.rs
   │   ├── views/
   │   │   ├── mod.rs
   │   │   ├── list.rs
   │   │   ├── detail.rs
   │   │   ├── profile.rs
   │   │   └── filter.rs
   │   ├── components/
   │   │   ├── mod.rs
   │   │   ├── table.rs
   │   │   ├── input.rs
   │   │   └── modal.rs
   │   └── theme.rs
   ├── events/
   │   ├── mod.rs
   │   ├── handler.rs
   │   └── keys.rs
   └── cache/
       └── mod.rs
   tests/
   └── integration/
       └── mod.rs
   ```
4. Add placeholder code in each module
5. Verify compilation

### Files to Modify/Create

- `Cargo.toml`: Project manifest with all dependencies
- `src/main.rs`: Entry point stub
- `src/app.rs`: App struct placeholder
- `src/config/mod.rs`: Config module exports
- `src/api/mod.rs`: API module exports
- `src/ui/mod.rs`: UI module exports
- `src/events/mod.rs`: Events module exports
- `src/cache/mod.rs`: Cache module placeholder
- `.gitignore`: Rust-specific ignores
- `README.md`: Basic project description

### Technical Specifications

**Cargo.toml structure:**
```toml
[package]
name = "lazyjira"
version = "0.1.0"
edition = "2021"
rust-version = "1.70"

[dependencies]
ratatui = "0.28"
crossterm = "0.28"
terminput = "0.2"
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = "0.3"
dirs = "5.0"
keyring = "2.0"
thiserror = "1.0"
clap = { version = "4.0", features = ["derive"] }

[dev-dependencies]
tokio-test = "0.4"
```

## Testing Requirements

- [ ] `cargo build` completes without errors
- [ ] `cargo test` runs (even with no tests yet)
- [ ] `cargo clippy` passes without warnings
- [ ] `cargo fmt --check` passes

## Dependencies

- **Prerequisite Tasks:** None (this is the first task)
- **Blocks Tasks:** Task 1.2, Task 1.3, Task 1.4, Task 1.5, Task 1.6, Task 1.7
- **External:** Rust toolchain (stable, 1.70+)

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Code follows Rust conventions and clippy recommendations
- [ ] Project compiles on stable Rust 1.70+
- [ ] Directory structure matches PRD specification
- [ ] Ready for subsequent implementation tasks
