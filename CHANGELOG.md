# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-12-06

### Added

- Create issue form with project and issue type dropdowns
- Assignee and priority pickers for new issue creation
- Epic parent field with dropdown selector
- Parent issue field for creating subtasks
- Vim-style keybindings in link picker (`j`/`k` navigation, `/` search, `Esc` close)
- Recent issues list in link picker for quick linking
- Alternative `n` key for creating new links
- Configurable `page_size` setting for issue list pagination
- Scroll-to-bottom auto-load for seamless pagination
- Manual load-more with `Ctrl+L` keybinding
- Loading indicator and pagination error display
- Context-aware keybinding prioritization in help system
- Help panel available in Issue Detail view (`?`)
- Dynamic help panel title showing current context

### Changed

- Improved dropdown component for project and issue type selection

## [0.1.0] - 2025-12-03

Initial release of LazyJira - a terminal-based JIRA TUI application built with Rust.

### Added

#### Core Features

- Issue list view with table display and navigation
- Issue detail view with full issue information display
- JQL query input with history support
- Filter panel with common filters (status, priority, assignee, type, epic)
- Saved filters functionality for quick access to frequent queries
- Quick search filter for loaded issues
- Issue sorting and pagination in list view
- Disk-based issue caching for offline viewing

#### Issue Management

- Edit issue summary and description (inline and modal editing)
- External editor integration for description editing
- Status transitions with picker interface
- Assignee and priority pickers
- Labels and components editing with tag editor
- Comments viewing and creation
- Issue history and changelog view
- Create and manage issue links
- Linked issues and subtasks display
- Open issue in browser functionality

#### Configuration & Profiles

- Configuration file structure and loading
- Multi-profile support with runtime switching
- Profile management TUI with CRUD operations
- Secure token storage via OS keyring

#### User Interface

- Comprehensive theme support (dark, light, high-contrast)
- Help panel and keyboard shortcuts system
- Command palette for quick command access
- Confirmation dialogs for destructive actions
- Vim-style keybindings in filter panel
- Persistent error display and notification system

#### Infrastructure

- Application architecture following The Elm Architecture (TEA)
- Structured logging with tracing and file output
- Error handling and user feedback system
- JIRA REST API client with async HTTP
- GitHub Actions CI workflow
- cargo-dist configuration for cross-platform releases and Homebrew formula

### Fixed

- List view sort order now applies to JQL queries
- Status labels correctly converted to IDs when restoring filter state
- Migrated to new JIRA search endpoint
- Improved first-run experience and notification UX
- Improved confirmation dialog button readability
- Improved tag editor selection readability
- Profile dialog height adjusted for content

### Changed

- Standardized input modes across picker components
- Use Ctrl+S instead of Ctrl+Enter for comment submission

[0.2.0]: https://github.com/jonbito/lazyjira/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jonbito/lazyjira/releases/tag/v0.1.0
