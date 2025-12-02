# LazyJira

A fast, keyboard-driven terminal user interface (TUI) for JIRA. Manage issues, track work, and navigate your projects without leaving the command line.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **Keyboard-first navigation** - Vim-style keybindings for efficient workflow
- **Multiple profiles** - Switch between JIRA instances (work, personal, clients)
- **Issue management** - View, edit, comment, and transition issues
- **Powerful filtering** - Quick filters, JQL queries, and saved filter presets
- **Offline viewing** - Cache issues for offline access
- **Theme support** - Dark, light, and high-contrast themes with customization
- **Secure credentials** - API tokens stored in your OS keychain

## Installation

### From source

```bash
git clone https://github.com/lazyjira/lazyjira.git
cd lazyjira
cargo install --path .
```

### Requirements

- Rust 1.70 or later
- A JIRA Cloud or Data Center instance
- [JIRA API token](https://support.atlassian.com/atlassian-account/docs/manage-api-tokens-for-your-atlassian-account/)

## Quick Start

1. Run LazyJira:
   ```bash
   lazyjira
   ```

2. On first run, you'll be prompted to create a profile with:
   - Profile name (e.g., `work`)
   - JIRA instance URL (e.g., `https://company.atlassian.net`)
   - Your email address
   - API token (stored securely in your OS keychain)

3. Start browsing your issues!

## Keyboard Shortcuts

### Global

| Key | Action |
|-----|--------|
| `?` | Show help panel |
| `Ctrl+C` | Quit application |
| `p` | Quick switch profile |
| `P` | Manage profiles (add/edit/delete) |
| `r` | Refresh current view |
| `Ctrl+P` / `Ctrl+K` | Open command palette |

### Issue List

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `gg` | Go to first issue |
| `G` | Go to last issue |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |
| `Enter` | Open issue details |
| `f` | Open filter panel |
| `F` | Open saved filters |
| `:` / `/` | Enter JQL query |
| `o` | Open issue in browser |
| `q` | Quit |

### Issue Detail

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `g` | Go to top |
| `G` | Go to bottom |
| `e` | Edit issue (summary/description) |
| `s` | Change status |
| `c` | Add comment |
| `a` | Change assignee |
| `y` | Change priority |
| `l` | Edit labels |
| `L` | Link issue |
| `o` | Open in browser |
| `q` / `Esc` | Go back to list |

### Filter Panel

| Key | Action |
|-----|--------|
| `Tab` / `←` / `→` | Switch section |
| `↑` / `↓` | Navigate options |
| `Space` | Toggle selection |
| `c` | Clear all filters |
| `Enter` | Apply filters |
| `Esc` | Cancel |

### Editor Mode

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save changes |
| `Esc` | Cancel editing |

## Configuration

Configuration is stored at `~/.config/lazyjira/config.toml`.

### Example Configuration

```toml
# Default profile to use on startup
default_profile = "work"

# UI theme: "dark", "light", or "high-contrast"
theme = "dark"

# Enable vim-style keybindings
vim_mode = true

# Cache settings
cache_ttl_minutes = 30
cache_max_size_mb = 100

# Confirmation dialogs
confirm_transitions = false
confirm_discard_changes = true

# Profiles
[[profiles]]
name = "work"
url = "https://company.atlassian.net"
email = "you@company.com"

[[profiles]]
name = "personal"
url = "https://personal.atlassian.net"
email = "you@personal.com"

# Custom theme colors (optional)
[custom_theme]
accent = "#ff00ff"
success = "lightgreen"
error = "#ff0000"
```

### Theme Customization

You can customize individual colors using:
- Named colors: `red`, `green`, `blue`, `cyan`, `magenta`, `yellow`, `white`, `gray`
- Light variants: `light-red`, `light-green`, etc.
- Hex colors: `#ff0000`, `#f00`
- RGB: `rgb(255, 0, 0)`

Available color overrides:
- `accent` - Primary accent color
- `success` - Success state color
- `warning` - Warning state color
- `error` - Error state color
- `info` - Information state color
- `border` - Border color
- `border_focused` - Focused border color
- `tag_bg` / `tag_fg` - Label/tag colors

## JQL Queries

LazyJira supports full JQL (JIRA Query Language) for powerful issue filtering:

```
# Find your open issues
assignee = currentUser() AND status != Done

# High priority bugs
project = MYPROJ AND type = Bug AND priority >= High

# Recently updated
updated >= -7d ORDER BY updated DESC

# Sprint issues
sprint in openSprints()
```

Press `:` or `/` in the issue list to enter a JQL query. Your query history is saved for quick access with `↑`/`↓`.

## Saved Filters

Create named filter presets for quick access:

1. Set up your filters using the filter panel (`f`)
2. Save the current filter with a name
3. Access saved filters with `F`

Filters are stored in your configuration file and persist across sessions.

## Security

- API tokens are stored in your operating system's secure keychain (macOS Keychain, Windows Credential Manager, or Linux Secret Service)
- Tokens are never written to configuration files
- HTTPS is enforced for all API communications

## Architecture

LazyJira is built with:
- **[ratatui](https://github.com/ratatui-org/ratatui)** - Terminal UI framework
- **[crossterm](https://github.com/crossterm-rs/crossterm)** - Cross-platform terminal handling
- **[tokio](https://tokio.rs/)** - Async runtime
- **[reqwest](https://github.com/seanmonstar/reqwest)** - HTTP client
- **[keyring](https://github.com/hwchen/keyring-rs)** - Secure credential storage

The application follows The Elm Architecture (TEA) pattern for predictable state management.

## Troubleshooting

### "Failed to connect to JIRA"
- Verify your JIRA URL is correct and accessible
- Check your API token is valid
- Ensure you have network connectivity

### "Authentication failed"
- Regenerate your API token at [Atlassian Account Settings](https://id.atlassian.com/manage-profile/security/api-tokens)
- Update the token: delete and re-add your profile

### Logs

LazyJira writes logs to help diagnose issues:
- macOS/Linux: `~/.local/share/lazyjira/logs/`
- Windows: `%APPDATA%\lazyjira\logs\`

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Run lints: `cargo clippy`
6. Format code: `cargo fmt`
7. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) for details.
