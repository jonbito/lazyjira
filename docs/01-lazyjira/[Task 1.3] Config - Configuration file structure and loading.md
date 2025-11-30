# Task 1.3: Configuration File Structure and Loading

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 1.3
**Area:** Configuration
**Estimated Effort:** M (4-8 hours)

## Description

Implement the configuration system for LazyJira, including XDG-compliant directory structure, TOML configuration file parsing, and settings management. This provides the foundation for profile and application settings.

## Acceptance Criteria

- [ ] XDG-compliant config directory (`~/.config/lazyjira/`)
- [ ] Config file created if not exists with defaults
- [ ] TOML configuration file parsing and serialization
- [ ] Settings struct with application preferences
- [ ] Profile struct with JIRA connection details
- [ ] Validation of configuration values
- [ ] Clear error messages for invalid configuration
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

- [ ] Default config created when none exists
- [ ] Valid TOML config loads correctly
- [ ] Invalid TOML produces clear error
- [ ] Missing required fields produce validation errors
- [ ] Profile without name rejected
- [ ] Profile with invalid URL format produces warning
- [ ] Settings have sensible defaults

## Dependencies

- **Prerequisite Tasks:** Task 1.1
- **Blocks Tasks:** Task 1.4 (needs Profile struct), Task 2.1, Task 2.2
- **External:** dirs, serde, toml, thiserror

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Config structs properly documented
- [ ] Unit tests for load/save/validate
- [ ] Error types defined with thiserror
- [ ] Works on Linux, macOS, Windows paths
