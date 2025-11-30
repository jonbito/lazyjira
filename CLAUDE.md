# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Run the application
cargo run

# Run tests
cargo test

# Run a single test
cargo test test_name

# Run tests in a specific module
cargo test module_name::

# Check code without building
cargo check

# Format code
cargo fmt

# Lint with clippy
cargo clippy
```

## Architecture Overview

LazyJira is a terminal-based JIRA TUI application built with Rust, using The Elm Architecture (TEA) pattern for predictable state management.

### Core Pattern: The Elm Architecture

The application follows TEA with three main phases:
1. **Model** (`App` struct in `src/app.rs`) - Holds all application state
2. **Update** (`App::update()`) - Processes events and mutates state
3. **View** (`App::view()`) - Renders UI based on current state

### Module Structure

- **`src/main.rs`** - Entry point, terminal setup/teardown, panic hook, main event loop
- **`src/app.rs`** - Central state machine with `AppState` enum and `App` struct containing all state
- **`src/api/`** - JIRA REST API client
  - `client.rs` - Async HTTP client with reqwest
  - `types.rs` - JIRA data structures (Issue, IssueFields, etc.)
  - `auth.rs` - Token management via OS keyring
- **`src/config/`** - Configuration management
  - `profile.rs` - JIRA profile (URL, email, token reference)
  - `settings.rs` - App settings (default profile, theme, etc.)
- **`src/ui/`** - TUI components using ratatui
  - `views/` - Full-screen views (list, detail, profile, filter)
  - `components/` - Reusable widgets (table, input, modal, notification)
- **`src/events/`** - Event handling
  - `handler.rs` - Crossterm event polling
  - `keys.rs` - Key binding definitions
- **`src/cache/`** - Issue caching for offline viewing
- **`src/error.rs`** - Application error types
- **`src/logging.rs`** - Tracing-based logging to file

### State Machine

The `AppState` enum controls which view is active:
- `Loading` - Initial data fetch
- `IssueList` - Main issue list view
- `IssueDetail` - Single issue detail view
- `ProfileManagement` - Profile CRUD operations
- `FilterPanel` - Filter overlay on list
- `JqlInput` - JQL query input overlay
- `Help` - Help screen

### Key Dependencies

- `ratatui` - TUI framework
- `crossterm` - Terminal backend (cross-platform)
- `tokio` - Async runtime
- `reqwest` - HTTP client for JIRA API
- `keyring` - Secure credential storage
- `serde` + `toml` - Configuration serialization

### Configuration

Config stored at `~/.config/lazyjira/config.toml`. Supports multiple JIRA profiles with tokens stored in OS keychain.
