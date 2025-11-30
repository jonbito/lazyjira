//! Application settings configuration.

use serde::{Deserialize, Serialize};

/// Default theme value.
fn default_theme() -> String {
    "dark".to_string()
}

/// Default vim mode value.
fn default_vim_mode() -> bool {
    true
}

/// Default cache TTL value in minutes.
fn default_cache_ttl() -> u32 {
    30
}

/// Application-wide settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    /// The name of the default profile to use.
    ///
    /// If not set, the first profile in the list will be used.
    #[serde(default)]
    pub default_profile: Option<String>,

    /// The UI theme to use.
    ///
    /// Defaults to "dark".
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Whether to use vim-style keybindings.
    ///
    /// Defaults to `true`.
    #[serde(default = "default_vim_mode")]
    pub vim_mode: bool,

    /// Cache time-to-live in minutes.
    ///
    /// Defaults to 30 minutes.
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_minutes: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_profile: None,
            theme: default_theme(),
            vim_mode: default_vim_mode(),
            cache_ttl_minutes: default_cache_ttl(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(settings.default_profile.is_none());
        assert_eq!(settings.theme, "dark");
        assert!(settings.vim_mode);
        assert_eq!(settings.cache_ttl_minutes, 30);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = Settings {
            default_profile: Some("work".to_string()),
            theme: "light".to_string(),
            vim_mode: false,
            cache_ttl_minutes: 60,
        };

        let toml_str = toml::to_string(&settings).unwrap();
        let parsed: Settings = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed, settings);
    }

    #[test]
    fn test_partial_settings_with_defaults() {
        // Only some fields specified - others should use defaults
        let toml_content = r#"
theme = "monokai"
"#;

        let settings: Settings = toml::from_str(toml_content).unwrap();
        assert!(settings.default_profile.is_none()); // default
        assert_eq!(settings.theme, "monokai"); // specified
        assert!(settings.vim_mode); // default
        assert_eq!(settings.cache_ttl_minutes, 30); // default
    }

    #[test]
    fn test_empty_settings_uses_defaults() {
        let toml_content = "";
        let settings: Settings = toml::from_str(toml_content).unwrap();

        assert!(settings.default_profile.is_none());
        assert_eq!(settings.theme, "dark");
        assert!(settings.vim_mode);
        assert_eq!(settings.cache_ttl_minutes, 30);
    }
}
