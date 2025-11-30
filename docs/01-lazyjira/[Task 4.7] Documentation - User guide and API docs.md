# Task 4.7: User Guide and API Documentation

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 4.7
**Area:** Documentation
**Estimated Effort:** M (4-6 hours)

## Description

Create comprehensive user documentation including installation guide, configuration reference, keyboard shortcuts, and API documentation for contributors.

## Acceptance Criteria

- [ ] README with installation instructions
- [ ] Configuration file reference
- [ ] Complete keyboard shortcuts reference
- [ ] Troubleshooting guide
- [ ] Contributing guidelines
- [ ] API documentation (rustdoc)
- [ ] Changelog

## Implementation Details

### Approach

1. Write comprehensive README
2. Create docs/ folder structure
3. Generate rustdoc documentation
4. Add inline documentation to code
5. Create CONTRIBUTING.md

### Files to Modify/Create

- `README.md`: Main documentation
- `docs/configuration.md`: Config reference
- `docs/shortcuts.md`: Keyboard shortcuts
- `docs/troubleshooting.md`: Common issues
- `CONTRIBUTING.md`: Contributor guide
- `CHANGELOG.md`: Version history

### Technical Specifications

**README.md Structure:**
```markdown
# LazyJira

A terminal-based user interface (TUI) for JIRA, built with Rust and ratatui.

## Features

- 📋 View and manage JIRA issues from your terminal
- 🔀 Multiple profile support for different JIRA instances
- ⌨️ Vim-style keyboard navigation
- 🔍 Powerful filtering with JQL support
- 📴 Offline viewing with issue caching
- 🎨 Customizable themes

## Installation

### From crates.io
```bash
cargo install lazyjira
```

### From source
```bash
git clone https://github.com/username/lazyjira
cd lazyjira
cargo install --path .
```

### Homebrew (macOS)
```bash
brew tap username/lazyjira
brew install lazyjira
```

## Quick Start

1. Create your first profile:
```bash
lazyjira setup
```

2. Enter your JIRA details:
   - URL: `https://yourcompany.atlassian.net`
   - Email: Your JIRA account email
   - API Token: Generate at https://id.atlassian.com/manage-profile/security/api-tokens

3. Launch the application:
```bash
lazyjira
```

## Configuration

Configuration is stored in `~/.config/lazyjira/config.toml`.

See [Configuration Reference](docs/configuration.md) for details.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Open/select |
| `q` | Go back / Quit |
| `?` | Show help |

See [Full Shortcuts Reference](docs/shortcuts.md).

## Screenshots

<!-- Add screenshots here -->

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT License - see [LICENSE](LICENSE).
```

**Configuration Reference (docs/configuration.md):**
```markdown
# Configuration Reference

LazyJira stores configuration in `~/.config/lazyjira/config.toml`.

## Settings

```toml
[settings]
# Default profile to use on startup
default_profile = "work"

# Color theme: "dark", "light", or "high-contrast"
theme = "dark"

# Enable vim-style keybindings
vim_mode = true

# Cache time-to-live in minutes
cache_ttl_minutes = 30

# Confirm before status transitions
confirm_transitions = false

# Confirm before discarding unsaved changes
confirm_discard_changes = true
```

## Profiles

Each profile represents a connection to a JIRA instance:

```toml
[[profiles]]
name = "work"
url = "https://company.atlassian.net"
email = "user@company.com"
# API token is stored securely in your OS keychain

[[profiles]]
name = "personal"
url = "https://personal.atlassian.net"
email = "user@personal.com"
```

### Profile Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Unique identifier for the profile |
| `url` | Yes | JIRA instance URL (must be HTTPS) |
| `email` | Yes | Your JIRA account email |

### API Token Storage

API tokens are stored securely using your operating system's keychain:
- **macOS**: Keychain Access
- **Linux**: Secret Service (GNOME Keyring, KWallet)
- **Windows**: Credential Manager

## Custom Themes

You can customize colors in your config:

```toml
[custom_theme]
accent = "#00ff00"
success = "green"
warning = "#ffaa00"
error = "red"
```

## Data Directories

- Config: `~/.config/lazyjira/`
- Cache: `~/.cache/lazyjira/`
- Logs: `~/.local/share/lazyjira/logs/`
```

**Keyboard Shortcuts (docs/shortcuts.md):**
```markdown
# Keyboard Shortcuts Reference

## Global

| Key | Action |
|-----|--------|
| `?` | Show help panel |
| `q` | Go back / Quit application |
| `p` | Switch profile |
| `r` | Refresh current view |
| `Ctrl+P` | Open command palette |

## Issue List

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `gg` | Go to first issue |
| `G` | Go to last issue |
| `Enter` | Open issue detail |
| `f` | Open filter panel |
| `/` | Quick search (in loaded issues) |
| `:` | Enter JQL query |
| `n` | Next search match |
| `N` | Previous search match |
| `s` | Sort by column |

## Issue Detail

| Key | Action |
|-----|--------|
| `e` | Edit issue |
| `c` | Add comment |
| `t` | Change status (transition) |
| `a` | Change assignee |
| `P` | Change priority |
| `l` | Edit labels |
| `L` | Edit components |
| `h` | View history |
| `j` / `k` | Scroll content |

## Editor Mode

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save changes |
| `Esc` | Cancel editing |
| Arrow keys | Navigate text |
| `Tab` | Next field |

## Filter Panel

| Key | Action |
|-----|--------|
| `Space` | Toggle selection |
| `Tab` | Next filter section |
| `Enter` | Apply filters |
| `c` | Clear all filters |
| `Esc` | Close panel |

## Dialogs

| Key | Action |
|-----|--------|
| `y` | Confirm |
| `n` / `Esc` | Cancel |
| `j` / `k` | Navigate options |
| `Enter` | Select option |
```

**Troubleshooting (docs/troubleshooting.md):**
```markdown
# Troubleshooting

## Authentication Issues

### "Authentication failed" error

1. Verify your API token is correct
2. Ensure you're using the email associated with your JIRA account
3. Check that the token hasn't expired
4. Generate a new token at https://id.atlassian.com/manage-profile/security/api-tokens

### Token not found in keyring

On Linux, ensure you have a Secret Service provider running:
- GNOME: `gnome-keyring-daemon`
- KDE: KWallet

## Connection Issues

### "Connection failed" error

1. Check your internet connection
2. Verify the JIRA URL is correct and includes `https://`
3. Check if your organization uses a VPN

### Slow response times

1. Check your network connection
2. Try reducing `cache_ttl_minutes` to refresh more often
3. Use filters to reduce the number of issues loaded

## Display Issues

### Terminal colors look wrong

1. Ensure your terminal supports 256 colors
2. Try the `high-contrast` theme
3. Check your terminal's color scheme

### UI is corrupted after resize

Press `Ctrl+L` to force a redraw (if implemented).

## Cache Issues

### Stale data displayed

1. Press `r` to refresh
2. Clear the cache: `lazyjira cache clear`

### Cache taking too much disk space

```bash
lazyjira cache clear
```

## Reporting Bugs

If you encounter a bug:

1. Check existing issues on GitHub
2. Collect log files from `~/.local/share/lazyjira/logs/`
3. Create a new issue with:
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Log excerpts
   - OS and terminal information
```

**CONTRIBUTING.md:**
```markdown
# Contributing to LazyJira

Thank you for your interest in contributing!

## Development Setup

1. Install Rust (1.70+): https://rustup.rs/
2. Clone the repository
3. Build: `cargo build`
4. Run tests: `cargo test`
5. Run linter: `cargo clippy`
6. Format code: `cargo fmt`

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## Code Style

- Follow Rust conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Add documentation for public APIs

## Commit Messages

Use conventional commits:
- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `refactor:` Code refactoring
- `test:` Test additions/changes
- `chore:` Maintenance tasks

## Testing

- Add unit tests for new functions
- Add integration tests for new features
- Ensure existing tests pass

## Documentation

- Update README if adding features
- Add rustdoc comments for public APIs
- Update keyboard shortcuts if adding bindings

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
```

## Testing Requirements

- [ ] README renders correctly on GitHub
- [ ] All links work
- [ ] Code examples are correct
- [ ] rustdoc generates without errors
- [ ] Documentation covers all features

## Dependencies

- **Prerequisite Tasks:** All feature tasks
- **Blocks Tasks:** None
- **External:** None

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Documentation reviewed
- [ ] No broken links
- [ ] Examples tested
- [ ] Changelog started
