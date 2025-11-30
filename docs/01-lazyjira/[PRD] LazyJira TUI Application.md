# PRD: LazyJira - JIRA TUI Application

## Executive Summary

LazyJira is a terminal-based user interface (TUI) application written in Rust that enables developers and project managers to efficiently manage JIRA work items without leaving their terminal environment. The application leverages the ratatui framework for rendering, supports multiple JIRA profiles, and provides comprehensive filtering and editing capabilities for JIRA issues.

## Problem Statement

Developers and technical users frequently context-switch between their terminal workflow and web-based JIRA interfaces, disrupting productivity and flow. The current web-based JIRA experience is often slow, requires mouse navigation, and doesn't integrate well with terminal-centric workflows. Additionally, users managing multiple JIRA instances (personal projects, different clients, multiple organizations) lack a unified tool that can seamlessly switch between these contexts.

Key pain points include:
- **Context switching overhead**: Moving between terminal and browser disrupts focus
- **Slow web interface**: JIRA's web UI can be resource-intensive and slow to navigate
- **Multiple account management**: No unified interface for managing multiple JIRA instances
- **Keyboard-driven workflow**: Developers prefer keyboard navigation over mouse interactions
- **Offline visibility**: Limited ability to quickly view/filter issues when connectivity is intermittent

## User Stories

### Primary User Story

As a **developer working in a terminal environment**, I want to **view, filter, and update JIRA issues directly from my terminal** so that **I can maintain my workflow without context-switching to a web browser**.

### Additional User Stories

- As a **consultant working with multiple clients**, I want to **manage multiple JIRA profiles** so that **I can quickly switch between different client projects without re-authenticating**.
- As a **team lead triaging issues**, I want to **filter work items by status, assignee, and labels** so that **I can quickly find and prioritize relevant issues**.
- As a **developer updating issue status**, I want to **modify all fields of a JIRA issue** so that **I can update progress, add comments, and change assignments without leaving my terminal**.
- As a **remote worker with intermittent connectivity**, I want to **quickly view cached issue data** so that **I can reference issue details even when offline**.
- As a **power user**, I want to **use keyboard shortcuts for all actions** so that **I can navigate and manage issues efficiently**.

## Market Research

### Existing TUI JIRA Tools

The current landscape of terminal-based JIRA tools is limited:

1. **jira-cli (go-jira)**: A Go-based CLI tool that provides command-line access to JIRA but lacks a true TUI interface
2. **jiracli**: Basic CLI tool without interactive features
3. **Various scripts/wrappers**: Custom solutions that lack polish and maintainability

### Rust TUI Ecosystem

Based on research from the [Ratatui ecosystem](https://github.com/ratatui/ratatui):

- **Ratatui** is the actively maintained successor to tui-rs, with strong community support
- The framework supports multiple backends (crossterm, termion, termwiz)
- [Best practices](https://github.com/ratatui/ratatui/discussions/220) recommend:
  - Using The Elm Architecture (TEA) for state management
  - Leveraging async runtimes (tokio) for API calls
  - Using the `dirs` crate for XDG-compliant configuration storage

### JIRA API Authentication

According to [Atlassian's documentation](https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/):

- **API Token + Basic Auth**: Recommended for personal scripts and tools
- Format: Base64 encoded `email:api_token`
- OAuth 2.0 available but more complex for CLI tools
- REST API v3 provides comprehensive issue management capabilities

### Competitive Advantages

| Feature | LazyJira | jira-cli | Web JIRA |
|---------|----------|----------|----------|
| TUI Interface | ✅ | ❌ | ❌ |
| Multiple Profiles | ✅ | ⚠️ Limited | ❌ |
| Keyboard-First | ✅ | ✅ | ❌ |
| Offline Viewing | ✅ | ❌ | ❌ |
| Fast Navigation | ✅ | ✅ | ❌ |
| Full Issue Editing | ✅ | ⚠️ Limited | ✅ |

## Functional Requirements

### Profile Management

- FR-1.1: Support multiple JIRA profiles with unique configurations
- FR-1.2: Store profiles in XDG-compliant configuration directory (`~/.config/lazyjira/`)
- FR-1.3: Each profile must contain: name, JIRA URL, email, and API token
- FR-1.4: Encrypt/obfuscate API tokens in configuration file
- FR-1.5: Provide TUI interface for adding, editing, and removing profiles
- FR-1.6: Support profile switching within the application

### Issue List View

- FR-2.1: Display paginated list of JIRA issues with key columns (Key, Summary, Status, Assignee, Priority)
- FR-2.2: Support column sorting (ascending/descending)
- FR-2.3: Filter issues by:
  - Status (Open, In Progress, Done, etc.)
  - Assignee (including "Assigned to me")
  - Project
  - Labels/Components
  - Sprint
  - Custom JQL queries
- FR-2.4: Quick search/filter within loaded issues
- FR-2.5: Keyboard navigation (vim-style: j/k, gg/G, /, etc.)
- FR-2.6: Visual indicators for issue priority and type

### Issue Detail View

- FR-3.1: Display all issue fields in a readable format
- FR-3.2: Support editing of:
  - Summary
  - Description (with markdown preview)
  - Status (workflow transitions)
  - Assignee
  - Priority
  - Labels
  - Components
  - Sprint
  - Story points/estimates
  - Custom fields
- FR-3.3: View and add comments
- FR-3.4: View issue history/changelog
- FR-3.5: Display linked issues and subtasks
- FR-3.6: Support attachments viewing (file list with metadata)

### Navigation & UX

- FR-4.1: Global keyboard shortcuts help panel (?)
- FR-4.2: Command palette for quick actions
- FR-4.3: Breadcrumb navigation showing current context
- FR-4.4: Status bar showing connection status and current profile
- FR-4.5: Confirmation dialogs for destructive actions
- FR-4.6: Toast/notification system for operation feedback

## Non-Functional Requirements

### Performance

- **Startup time**: Application must launch in under 500ms
- **API response handling**: Async operations with loading indicators
- **Pagination**: Load issues in batches of 50 to maintain responsiveness
- **Caching**: Cache issue data for offline viewing and reduced API calls
- **Memory usage**: Target under 50MB RAM for typical usage

### Security

- **Token storage**: API tokens must be stored encrypted or use OS keychain
- **No plaintext secrets**: Never log or display API tokens
- **Secure transport**: Enforce HTTPS for all API communications
- **Token validation**: Validate tokens on profile creation/edit
- **Session handling**: Clear sensitive data from memory when switching profiles

### Scalability

- **Large issue sets**: Handle projects with 10,000+ issues via pagination
- **Multiple profiles**: Support at least 20 configured profiles
- **Concurrent operations**: Support background refresh while user navigates

### Usability

- **Accessibility**: Support terminal color themes and high contrast modes
- **Responsiveness**: Adapt layout to terminal size (minimum 80x24)
- **Error handling**: Clear, actionable error messages
- **Discoverability**: Contextual hints for available actions
- **Consistency**: Follow established TUI conventions (vim-like bindings optional)

### Compatibility

- **Platforms**: Linux, macOS, Windows (via crossterm)
- **JIRA versions**: JIRA Cloud REST API v3, Data Center compatibility
- **Terminal emulators**: Support major terminals (iTerm2, Alacritty, Windows Terminal, etc.)
- **Shell integration**: Work correctly within tmux/screen sessions

## Success Metrics

### Primary Metrics

- **Adoption**: 500+ GitHub stars within 6 months of release
- **Active users**: 100+ weekly active users (via opt-in telemetry)
- **Issue operations**: Average 10+ issue views/updates per session

### Secondary Metrics

- **User satisfaction**: 4+ star average rating on crates.io
- **Contribution**: 10+ external contributors within first year
- **Documentation quality**: Less than 5 GitHub issues tagged "documentation" per month
- **Reliability**: Crash rate below 0.1% of sessions

## Technical Considerations

### Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Language | Rust | Performance, safety, cross-platform |
| TUI Framework | ratatui | Active development, rich widget library |
| Terminal Backend | crossterm | Cross-platform, pure Rust |
| Input Handling | terminput | Flexible input parsing |
| HTTP Client | reqwest | Async, feature-rich HTTP client |
| Async Runtime | tokio | Industry standard async runtime |
| Logging | tracing | Structured logging, spans |
| Config Directory | dirs | XDG-compliant path resolution |
| Serialization | serde + toml | Human-readable config files |
| Keyring | keyring | Secure credential storage |

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        LazyJira TUI                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Views     │  │   State     │  │   Event Handler     │  │
│  │             │  │  (Model)    │  │                     │  │
│  │ - List      │◄─┤             │◄─┤ - Keyboard Input    │  │
│  │ - Detail    │  │ - Issues    │  │ - API Responses     │  │
│  │ - Profile   │  │ - Profiles  │  │ - Signals           │  │
│  │ - Filter    │  │ - UI State  │  │                     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                      Service Layer                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐  │
│  │  JIRA Client    │  │  Config Manager │  │  Cache      │  │
│  │                 │  │                 │  │             │  │
│  │ - Auth          │  │ - Load/Save     │  │ - Issues    │  │
│  │ - Issues CRUD   │  │ - Profiles      │  │ - Metadata  │  │
│  │ - Search/JQL    │  │ - Encryption    │  │             │  │
│  └─────────────────┘  └─────────────────┘  └─────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    External Services                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐  │
│  │  JIRA REST API  │  │  OS Keychain    │  │  Filesystem │  │
│  └─────────────────┘  └─────────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Design Patterns

1. **The Elm Architecture (TEA)**: Unidirectional data flow for predictable state management
2. **Command Pattern**: Encapsulate user actions for undo/redo capability
3. **Repository Pattern**: Abstract JIRA API access for testability
4. **Observer Pattern**: React to async API responses and state changes

### Project Structure

```
lazyjira/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, app initialization
│   ├── app.rs               # Main application state and loop
│   ├── config/
│   │   ├── mod.rs
│   │   ├── profile.rs       # Profile management
│   │   └── settings.rs      # App settings
│   ├── api/
│   │   ├── mod.rs
│   │   ├── client.rs        # JIRA API client
│   │   ├── types.rs         # API response types
│   │   └── auth.rs          # Authentication handling
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── views/
│   │   │   ├── list.rs      # Issue list view
│   │   │   ├── detail.rs    # Issue detail view
│   │   │   ├── profile.rs   # Profile management view
│   │   │   └── filter.rs    # Filter/search view
│   │   ├── components/
│   │   │   ├── table.rs     # Reusable table widget
│   │   │   ├── input.rs     # Text input widget
│   │   │   └── modal.rs     # Modal dialog widget
│   │   └── theme.rs         # Color schemes and styling
│   ├── events/
│   │   ├── mod.rs
│   │   ├── handler.rs       # Event processing
│   │   └── keys.rs          # Key binding definitions
│   └── cache/
│       └── mod.rs           # Issue caching logic
└── tests/
    └── integration/
```

## Implementation Phases

### Phase 1: Foundation (MVP)

**Core functionality enabling basic issue viewing and profile management**

- [ ] Project scaffolding with ratatui/crossterm
- [ ] Configuration file structure and loading (TOML format)
- [ ] Single profile support with token storage
- [ ] JIRA API client with basic authentication
- [ ] Issue list view with basic columns (Key, Summary, Status)
- [ ] Keyboard navigation (j/k, Enter to select)
- [ ] Issue detail view (read-only)
- [ ] Error handling and user feedback
- [ ] Basic logging with tracing

**Exit Criteria**: User can configure a profile, view issues, and read issue details

### Phase 2: Multi-Profile & Filtering

**Enhanced profile management and issue discovery**

- [ ] Multiple profile support with switching
- [ ] Profile management TUI (add/edit/delete)
- [ ] Secure token storage via OS keychain
- [ ] Filter panel with common filters (status, assignee, project)
- [ ] JQL query input support
- [ ] Issue sorting by column
- [ ] Pagination with lazy loading
- [ ] Quick search within loaded issues
- [ ] Cache layer for offline viewing

**Exit Criteria**: User can manage multiple profiles and filter issues effectively

### Phase 3: Full Issue Editing

**Complete issue lifecycle management**

- [ ] Edit issue summary and description
- [ ] Status transitions (workflow-aware)
- [ ] Assignee changes with user search
- [ ] Priority and label updates
- [ ] Comment viewing and creation
- [ ] Sprint and story point editing
- [ ] Custom field support
- [ ] Confirmation dialogs for changes
- [ ] Undo/redo for local changes

**Exit Criteria**: User can fully manage issue lifecycle from the TUI

### Phase 4: Polish & Advanced Features

**Enhanced UX and power-user features**

- [ ] Vim-style keybindings (optional)
- [ ] Command palette (Ctrl+P style)
- [ ] Theme support (light/dark/custom)
- [ ] Issue history/changelog view
- [ ] Linked issues and subtasks navigation
- [ ] Bulk operations (multi-select)
- [ ] Export issues to markdown/JSON
- [ ] Watch/unwatch issues
- [ ] Notifications integration
- [ ] Plugin/extension system

**Exit Criteria**: Feature-complete TUI with power-user capabilities

## Risk Assessment

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| JIRA API rate limiting | Medium | High | Implement request throttling, caching, and graceful degradation |
| Cross-platform terminal inconsistencies | Medium | Medium | Use crossterm abstraction, extensive testing on all platforms |
| API version compatibility | Low | High | Abstract API layer, version detection, graceful feature degradation |
| Complex async state management | Medium | Medium | Use established patterns (TEA), comprehensive testing |
| Token security vulnerabilities | Low | High | Use OS keychain, security audit, no plaintext storage |

### Business Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Limited adoption due to niche use case | Medium | Medium | Focus on developer communities, integrate with popular workflows |
| Atlassian API changes/deprecations | Low | High | Monitor API changelog, maintain compatibility layer |
| Competition from official Atlassian CLI | Low | Medium | Differentiate with TUI experience and multi-profile support |
| Maintenance burden as sole maintainer | Medium | Medium | Build community, good documentation, modular architecture |

## Dependencies

### Internal Dependencies

- Rust toolchain (stable, MSRV 1.70+)
- Cargo build system
- Unit and integration test framework

### External Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| ratatui | ^0.28 | TUI framework |
| crossterm | ^0.28 | Terminal backend |
| terminput | ^0.2 | Input handling |
| tokio | ^1.0 | Async runtime |
| reqwest | ^0.12 | HTTP client |
| serde | ^1.0 | Serialization |
| toml | ^0.8 | Config file format |
| tracing | ^0.1 | Logging framework |
| dirs | ^5.0 | XDG directories |
| keyring | ^2.0 | Secure credential storage |
| thiserror | ^1.0 | Error handling |
| clap | ^4.0 | CLI argument parsing |

### External Services

- JIRA Cloud REST API v3 or JIRA Data Center
- Operating system keychain (optional, for secure storage)

## Appendix

### A. Key Binding Reference (Proposed)

| Key | Context | Action |
|-----|---------|--------|
| `j` / `↓` | List | Move down |
| `k` / `↑` | List | Move up |
| `Enter` | List | Open issue detail |
| `q` | Any | Go back / Quit |
| `/` | List | Open search |
| `f` | List | Open filter panel |
| `p` | Any | Switch profile |
| `r` | List | Refresh issues |
| `e` | Detail | Edit issue |
| `c` | Detail | Add comment |
| `?` | Any | Show help |
| `Esc` | Any | Cancel/Close |

### B. Configuration File Format

```toml
# ~/.config/lazyjira/config.toml

[settings]
default_profile = "work"
theme = "dark"
vim_mode = true
cache_ttl_minutes = 30

[[profiles]]
name = "work"
url = "https://company.atlassian.net"
email = "user@company.com"
# Token stored in OS keychain, referenced by profile name

[[profiles]]
name = "personal"
url = "https://personal.atlassian.net"
email = "user@personal.com"
```

### C. References

- [Ratatui Documentation](https://ratatui.rs/)
- [Ratatui GitHub Repository](https://github.com/ratatui/ratatui)
- [JIRA REST API v3 Documentation](https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/)
- [JIRA Basic Authentication](https://developer.atlassian.com/cloud/jira/software/basic-auth-for-rest-apis/)
- [The Elm Architecture in Ratatui](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/)
- [Awesome Ratatui - Example Applications](https://github.com/ratatui/awesome-ratatui)

---

## Task Breakdown

This PRD has been broken down into the following implementation tasks:

### Phase 1: Foundation (MVP)

- [ ] [Task 1.1] Infrastructure: Project scaffolding and dependencies
- [ ] [Task 1.2] Infrastructure: Application architecture and main loop
- [ ] [Task 1.3] Config: Configuration file structure and loading
- [ ] [Task 1.4] API: JIRA client with basic authentication
- [ ] [Task 1.5] API: Issue data types and parsing
- [ ] [Task 1.6] UI: Issue list view with basic columns
- [ ] [Task 1.7] UI: Issue detail view (read-only)
- [ ] [Task 1.8] Infrastructure: Error handling and user feedback
- [ ] [Task 1.9] Infrastructure: Logging with tracing

### Phase 2: Multi-Profile & Filtering

- [ ] [Task 2.1] Config: Multiple profile support with switching
- [ ] [Task 2.2] UI: Profile management TUI
- [ ] [Task 2.3] UI: Filter panel with common filters
- [ ] [Task 2.4] UI: JQL query input support
- [ ] [Task 2.5] UI: Issue sorting and pagination
- [ ] [Task 2.6] UI: Quick search within loaded issues
- [ ] [Task 2.7] Cache: Issue caching for offline viewing

### Phase 3: Full Issue Editing

- [ ] [Task 3.1] API: Issue update operations
- [ ] [Task 3.2] UI: Edit issue summary and description
- [ ] [Task 3.3] UI: Status transitions
- [ ] [Task 3.4] UI: Assignee and priority changes
- [ ] [Task 3.5] UI: Comments viewing and creation
- [ ] [Task 3.6] UI: Labels and components editing
- [ ] [Task 3.7] UI: Confirmation dialogs for changes

### Phase 4: Polish & Advanced Features

- [ ] [Task 4.1] UI: Help panel and keyboard shortcuts
- [ ] [Task 4.2] UI: Theme support
- [ ] [Task 4.3] UI: Command palette
- [ ] [Task 4.4] UI: Issue history and changelog
- [ ] [Task 4.5] UI: Linked issues and subtasks
- [ ] [Task 4.6] Testing: Unit and integration tests
- [ ] [Task 4.7] Documentation: User guide and API docs

**Total Tasks:** 30
**Critical Path:** Task 1.1 → Task 1.2 → Task 1.3 → Task 1.4 → Task 1.6 → Task 1.7 → Task 3.1 → Task 3.2
**Implementation Order:** Follow task numbering sequence for optimal dependency flow
