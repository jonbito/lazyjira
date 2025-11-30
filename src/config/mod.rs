//! Configuration management for LazyJira.
//!
//! This module handles loading, saving, and managing user configuration
//! including profiles and application settings.
//!
//! # Configuration Directory Structure
//!
//! LazyJira uses XDG-compliant configuration directories:
//!
//! ```text
//! ~/.config/lazyjira/
//! ├── config.toml      # Main configuration file
//! └── cache/           # Issue cache (future)
//! ```
//!
//! # Configuration File Format
//!
//! ```toml
//! [settings]
//! default_profile = "work"
//! theme = "dark"
//! vim_mode = true
//! cache_ttl_minutes = 30
//!
//! [[profiles]]
//! name = "work"
//! url = "https://company.atlassian.net"
//! email = "user@company.com"
//! ```

mod profile;
mod settings;

pub use profile::Profile;
pub use settings::Settings;

// Re-export Config and ConfigError at the module level
pub use self::ConfigError as Error;

use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when working with configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Could not determine the configuration directory.
    #[error("could not determine configuration directory")]
    NoConfigDir,

    /// Failed to create the configuration directory.
    #[error("failed to create configuration directory: {0}")]
    CreateDirError(#[source] std::io::Error),

    /// Failed to read the configuration file.
    #[error("failed to read configuration file: {0}")]
    ReadError(#[source] std::io::Error),

    /// Failed to write the configuration file.
    #[error("failed to write configuration file: {0}")]
    WriteError(#[source] std::io::Error),

    /// Failed to parse the configuration file.
    #[error("failed to parse configuration file: {0}")]
    ParseError(#[source] toml::de::Error),

    /// Failed to serialize the configuration.
    #[error("failed to serialize configuration: {0}")]
    SerializeError(#[source] toml::ser::Error),

    /// Configuration validation failed.
    #[error("configuration validation failed: {0}")]
    ValidationError(String),
}

/// Result type for configuration operations.
pub type Result<T> = std::result::Result<T, ConfigError>;

/// The root configuration structure for LazyJira.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Application-wide settings.
    #[serde(default)]
    pub settings: Settings,

    /// List of JIRA profiles.
    #[serde(default)]
    pub profiles: Vec<Profile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            profiles: Vec::new(),
        }
    }
}

impl Config {
    /// Get the configuration directory path.
    ///
    /// Returns the XDG-compliant configuration directory for LazyJira,
    /// typically `~/.config/lazyjira/` on Linux/macOS.
    pub fn config_dir() -> Result<PathBuf> {
        dirs::config_dir()
            .ok_or(ConfigError::NoConfigDir)
            .map(|p| p.join("lazyjira"))
    }

    /// Get the configuration file path.
    ///
    /// Returns the path to the main configuration file,
    /// typically `~/.config/lazyjira/config.toml`.
    pub fn config_path() -> Result<PathBuf> {
        Self::config_dir().map(|p| p.join("config.toml"))
    }

    /// Load configuration from the default location.
    ///
    /// If the configuration file does not exist, returns a default configuration.
    /// If the file exists but is invalid, returns an error with details.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration directory cannot be determined
    /// - The configuration file exists but cannot be read
    /// - The configuration file contains invalid TOML
    /// - The configuration fails validation
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path).map_err(ConfigError::ReadError)?;

        let config: Config = toml::from_str(&content).map_err(ConfigError::ParseError)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to the default location.
    ///
    /// Creates the configuration directory if it does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration directory cannot be determined or created
    /// - The configuration cannot be serialized
    /// - The configuration file cannot be written
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        let config_path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(ConfigError::CreateDirError)?;
        }

        let content = toml::to_string_pretty(self).map_err(ConfigError::SerializeError)?;

        fs::write(&config_path, content).map_err(ConfigError::WriteError)?;

        Ok(())
    }

    /// Validate the configuration.
    ///
    /// Checks that:
    /// - All profiles have non-empty names
    /// - All profiles have valid URLs
    /// - All profiles have valid email addresses
    /// - Profile names are unique
    /// - The default profile (if set) exists
    ///
    /// # Errors
    ///
    /// Returns a `ConfigError::ValidationError` with details if validation fails.
    pub fn validate(&self) -> Result<()> {
        // Check for duplicate profile names
        let mut seen_names = std::collections::HashSet::new();
        for profile in &self.profiles {
            profile.validate()?;

            if !seen_names.insert(&profile.name) {
                return Err(ConfigError::ValidationError(format!(
                    "duplicate profile name: '{}'",
                    profile.name
                )));
            }
        }

        // Validate default_profile references an existing profile
        if let Some(ref default_profile) = self.settings.default_profile {
            if !self.profiles.iter().any(|p| &p.name == default_profile) {
                return Err(ConfigError::ValidationError(format!(
                    "default profile '{}' does not exist",
                    default_profile
                )));
            }
        }

        Ok(())
    }

    /// Get a profile by name.
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// Get the default profile.
    ///
    /// Returns the profile specified by `settings.default_profile`,
    /// or the first profile if no default is set.
    pub fn get_default_profile(&self) -> Option<&Profile> {
        if let Some(ref default_name) = self.settings.default_profile {
            self.get_profile(default_name)
        } else {
            self.profiles.first()
        }
    }

    /// Add a profile to the configuration.
    ///
    /// Returns an error if a profile with the same name already exists.
    pub fn add_profile(&mut self, profile: Profile) -> Result<()> {
        profile.validate()?;

        if self.profiles.iter().any(|p| p.name == profile.name) {
            return Err(ConfigError::ValidationError(format!(
                "profile '{}' already exists",
                profile.name
            )));
        }

        self.profiles.push(profile);
        Ok(())
    }

    /// Remove a profile by name.
    ///
    /// Returns `true` if the profile was removed, `false` if it didn't exist.
    /// Also clears the default profile if it was the removed profile.
    pub fn remove_profile(&mut self, name: &str) -> bool {
        let initial_len = self.profiles.len();
        self.profiles.retain(|p| p.name != name);

        // Clear default profile if it was the removed one
        if self.settings.default_profile.as_deref() == Some(name) {
            self.settings.default_profile = None;
        }

        self.profiles.len() < initial_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.profiles.is_empty());
        assert!(config.settings.default_profile.is_none());
        assert_eq!(config.settings.theme, "dark");
        assert!(config.settings.vim_mode);
        assert_eq!(config.settings.cache_ttl_minutes, 30);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config {
            settings: Settings {
                default_profile: Some("work".to_string()),
                theme: "light".to_string(),
                vim_mode: false,
                cache_ttl_minutes: 60,
            },
            profiles: vec![
                Profile::new(
                    "work".to_string(),
                    "https://company.atlassian.net".to_string(),
                    "user@company.com".to_string(),
                ),
                Profile::new(
                    "personal".to_string(),
                    "https://personal.atlassian.net".to_string(),
                    "user@personal.com".to_string(),
                ),
            ],
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.settings.default_profile, config.settings.default_profile);
        assert_eq!(parsed.settings.theme, config.settings.theme);
        assert_eq!(parsed.profiles.len(), 2);
        assert_eq!(parsed.profiles[0].name, "work");
        assert_eq!(parsed.profiles[1].name, "personal");
    }

    #[test]
    fn test_duplicate_profile_names_rejected() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile::new(
                    "work".to_string(),
                    "https://company.atlassian.net".to_string(),
                    "user@company.com".to_string(),
                ),
                Profile::new(
                    "work".to_string(),
                    "https://other.atlassian.net".to_string(),
                    "other@company.com".to_string(),
                ),
            ],
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("duplicate profile name"));
    }

    #[test]
    fn test_default_profile_must_exist() {
        let config = Config {
            settings: Settings {
                default_profile: Some("nonexistent".to_string()),
                ..Settings::default()
            },
            profiles: vec![Profile::new(
                "work".to_string(),
                "https://company.atlassian.net".to_string(),
                "user@company.com".to_string(),
            )],
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_get_profile() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile::new(
                    "work".to_string(),
                    "https://company.atlassian.net".to_string(),
                    "user@company.com".to_string(),
                ),
                Profile::new(
                    "personal".to_string(),
                    "https://personal.atlassian.net".to_string(),
                    "user@personal.com".to_string(),
                ),
            ],
        };

        assert!(config.get_profile("work").is_some());
        assert!(config.get_profile("personal").is_some());
        assert!(config.get_profile("nonexistent").is_none());
    }

    #[test]
    fn test_get_default_profile() {
        // With explicit default
        let config = Config {
            settings: Settings {
                default_profile: Some("personal".to_string()),
                ..Settings::default()
            },
            profiles: vec![
                Profile::new(
                    "work".to_string(),
                    "https://company.atlassian.net".to_string(),
                    "user@company.com".to_string(),
                ),
                Profile::new(
                    "personal".to_string(),
                    "https://personal.atlassian.net".to_string(),
                    "user@personal.com".to_string(),
                ),
            ],
        };
        assert_eq!(config.get_default_profile().unwrap().name, "personal");

        // Without explicit default - returns first
        let config_no_default = Config {
            settings: Settings::default(),
            profiles: vec![Profile::new(
                "work".to_string(),
                "https://company.atlassian.net".to_string(),
                "user@company.com".to_string(),
            )],
        };
        assert_eq!(config_no_default.get_default_profile().unwrap().name, "work");

        // Empty profiles
        let empty_config = Config::default();
        assert!(empty_config.get_default_profile().is_none());
    }

    #[test]
    fn test_add_profile() {
        let mut config = Config::default();

        let profile = Profile::new(
            "work".to_string(),
            "https://company.atlassian.net".to_string(),
            "user@company.com".to_string(),
        );

        assert!(config.add_profile(profile.clone()).is_ok());
        assert_eq!(config.profiles.len(), 1);

        // Adding duplicate should fail
        let result = config.add_profile(profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_remove_profile() {
        let mut config = Config {
            settings: Settings {
                default_profile: Some("work".to_string()),
                ..Settings::default()
            },
            profiles: vec![
                Profile::new(
                    "work".to_string(),
                    "https://company.atlassian.net".to_string(),
                    "user@company.com".to_string(),
                ),
                Profile::new(
                    "personal".to_string(),
                    "https://personal.atlassian.net".to_string(),
                    "user@personal.com".to_string(),
                ),
            ],
        };

        assert!(config.remove_profile("work"));
        assert_eq!(config.profiles.len(), 1);
        assert!(config.settings.default_profile.is_none()); // default cleared

        assert!(!config.remove_profile("nonexistent"));
        assert_eq!(config.profiles.len(), 1);
    }

    #[test]
    fn test_parse_toml_config() {
        let toml_content = r#"
[settings]
default_profile = "work"
theme = "dark"
vim_mode = true
cache_ttl_minutes = 30

[[profiles]]
name = "work"
url = "https://company.atlassian.net"
email = "user@company.com"

[[profiles]]
name = "personal"
url = "https://personal.atlassian.net"
email = "user@personal.com"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.settings.default_profile, Some("work".to_string()));
        assert_eq!(config.settings.theme, "dark");
        assert!(config.settings.vim_mode);
        assert_eq!(config.settings.cache_ttl_minutes, 30);
        assert_eq!(config.profiles.len(), 2);
        assert_eq!(config.profiles[0].name, "work");
        assert_eq!(config.profiles[0].url, "https://company.atlassian.net");
        assert_eq!(config.profiles[1].name, "personal");
    }

    #[test]
    fn test_parse_minimal_config() {
        // Test with only required fields - defaults should apply
        let toml_content = r#"
[[profiles]]
name = "work"
url = "https://company.atlassian.net"
email = "user@company.com"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert!(config.settings.default_profile.is_none());
        assert_eq!(config.settings.theme, "dark"); // default
        assert!(config.settings.vim_mode); // default
        assert_eq!(config.settings.cache_ttl_minutes, 30); // default
        assert_eq!(config.profiles.len(), 1);
    }

    #[test]
    fn test_parse_empty_config() {
        let toml_content = "";
        let config: Config = toml::from_str(toml_content).unwrap();
        assert!(config.profiles.is_empty());
    }
}
