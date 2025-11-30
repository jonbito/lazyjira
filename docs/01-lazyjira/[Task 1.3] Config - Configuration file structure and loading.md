# Task 1.3: Configuration File Structure and Loading

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 1.3
**Area:** Configuration
**Estimated Effort:** M (4-8 hours)

## Description

Implement the configuration system for LazyJira, including XDG-compliant directory structure, TOML configuration file parsing, and settings management. This provides the foundation for profile and application settings.

## Acceptance Criteria

- [x] XDG-compliant config directory (`~/.config/lazyjira/`)
- [x] Config file created if not exists with defaults
- [x] TOML configuration file parsing and serialization
- [x] Settings struct with application preferences
- [x] Profile struct with JIRA connection details
- [x] Validation of configuration values
- [x] Clear error messages for invalid configuration
- [ ] Configuration hot-reload capability (nice-to-have)

## Implementation Details

### Approach

1. Use `dirs` crate to get XDG config directory
2. Define configuration structs with serde:
   - `Config`: Root configuration
   - `Settings`: Application preferences
   - `Profile`: JIRA connection details
3. Implement load/save functions
4. Add validation for required fields
5. Provide sensible defaults
6. Handle missing/corrupted config gracefully

### Files to Modify/Create

- `src/config/mod.rs`: Module exports, Config struct, load/save functions
- `src/config/settings.rs`: Settings struct and defaults
- `src/config/profile.rs`: Profile struct and validation

### Technical Specifications

**Config Directory Structure:**
```
~/.config/lazyjira/
├── config.toml      # Main configuration
└── cache/           # Issue cache (future)
```

**Configuration File Format (from PRD):**
```toml
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

**Rust Structs:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub settings: Settings,
    #[serde(default)]
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub default_profile: Option<String>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub vim_mode: bool,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub url: String,
    pub email: String,
    // Token stored in keyring, not in config file
}
```

**Config Loading Logic:**
```rust
pub fn load_config() -> Result<Config> {
    let config_dir = dirs::config_dir()
        .ok_or(ConfigError::NoConfigDir)?
        .join("lazyjira");

    let config_path = config_dir.join("config.toml");

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;
    config.validate()?;
    Ok(config)
}
```

## Testing Requirements

- [x] Default config created when none exists
- [x] Valid TOML config loads correctly
- [x] Invalid TOML produces clear error
- [x] Missing required fields produce validation errors
- [x] Profile without name rejected
- [x] Profile with invalid URL format produces warning
- [x] Settings have sensible defaults

## Dependencies

- **Prerequisite Tasks:** Task 1.1
- **Blocks Tasks:** Task 1.4 (needs Profile struct), Task 2.1, Task 2.2
- **External:** dirs, serde, toml, thiserror

## Definition of Done

- [x] All acceptance criteria met
- [x] Config structs properly documented
- [x] Unit tests for load/save/validate
- [x] Error types defined with thiserror
- [x] Works on Linux, macOS, Windows paths

## Completion Notes

### Implementation Summary

The configuration system was implemented with the following components:

**Files Modified:**
- `src/config/mod.rs` - Added `Config` struct, `ConfigError` enum with thiserror, load/save functions, and validation logic
- `src/config/profile.rs` - Added validation methods for Profile (name, URL, email validation)
- `src/config/settings.rs` - Added serde default functions for all settings fields

**Key Implementation Details:**
- Uses `dirs` crate for XDG-compliant config directory resolution
- Full TOML serialization/deserialization with serde
- Comprehensive validation including:
  - Profile name uniqueness and whitespace checks
  - URL scheme validation (http/https)
  - Email format validation
  - Default profile reference validation
- Graceful handling of missing config (returns defaults)
- Clear error messages via thiserror derive macros

**Test Coverage (22 tests for config module):**
- Default configuration values
- Serialization roundtrip
- TOML parsing (full, minimal, empty configs)
- Validation error cases (duplicate names, missing default, invalid profiles)
- Profile CRUD operations (add/remove/get)
- Settings defaults with partial config

**Cross-Platform Support:**
The implementation uses platform-agnostic path handling via the `dirs` crate which handles:
- Linux: `~/.config/lazyjira/`
- macOS: `~/Library/Application Support/lazyjira/`
- Windows: `%APPDATA%\lazyjira\`
