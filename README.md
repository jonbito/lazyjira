# LazyJira

A terminal-based user interface (TUI) for JIRA that enables developers and project managers to efficiently manage JIRA work items without leaving their terminal environment.

## Features

- **Terminal-native**: Manage JIRA issues directly from your terminal
- **Multiple profiles**: Support for multiple JIRA instances (personal, work, clients)
- **Keyboard-first**: Vim-style navigation and keyboard shortcuts
- **Offline viewing**: Cache issues for offline access
- **Fast navigation**: Quick filtering and searching of issues

## Installation

### From source

```bash
cargo install --path .
```

### Requirements

- Rust 1.70 or later
- A JIRA Cloud or Data Center instance
- JIRA API token

## Usage

```bash
lazyjira
```

## Configuration

Configuration is stored in `~/.config/lazyjira/config.toml`.

## License

MIT
