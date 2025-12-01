//! Application settings configuration.

use serde::{Deserialize, Serialize};

use crate::ui::theme::CustomThemeConfig;

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

/// Default cache max size in MB.
fn default_cache_max_size() -> u64 {
    100
}

/// Default for confirm_discard_changes setting.
fn default_confirm_discard() -> bool {
    true
}

/// Maximum number of JQL queries to keep in history.
const MAX_JQL_HISTORY: usize = 10;

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

    /// Maximum cache size in megabytes.
    ///
    /// Defaults to 100 MB.
    #[serde(default = "default_cache_max_size")]
    pub cache_max_size_mb: u64,

    /// JQL query history (most recent first).
    ///
    /// Limited to 10 entries.
    #[serde(default)]
    pub jql_history: Vec<String>,

    /// Whether to show confirmation dialog for status transitions.
    ///
    /// Defaults to `false` (transitions execute immediately).
    #[serde(default)]
    pub confirm_transitions: bool,

    /// Whether to show confirmation dialog for discarding unsaved changes.
    ///
    /// Defaults to `true`.
    #[serde(default = "default_confirm_discard")]
    pub confirm_discard_changes: bool,

    /// Custom theme color overrides.
    ///
    /// Allows customizing individual colors of the selected theme.
    #[serde(default)]
    pub custom_theme: Option<CustomThemeConfig>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_profile: None,
            theme: default_theme(),
            vim_mode: default_vim_mode(),
            cache_ttl_minutes: default_cache_ttl(),
            cache_max_size_mb: default_cache_max_size(),
            jql_history: Vec::new(),
            confirm_transitions: false,
            confirm_discard_changes: default_confirm_discard(),
            custom_theme: None,
        }
    }
}

impl Settings {
    /// Add a JQL query to the history.
    ///
    /// The query is added to the front of the history. If the query already
    /// exists in the history, it is moved to the front. The history is
    /// limited to 10 entries.
    pub fn add_jql_to_history(&mut self, query: String) {
        // Remove duplicate if exists
        self.jql_history.retain(|q| q != &query);

        // Add to front
        self.jql_history.insert(0, query);

        // Trim to max size
        self.jql_history.truncate(MAX_JQL_HISTORY);
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
        assert_eq!(settings.cache_max_size_mb, 100);
        assert!(settings.jql_history.is_empty());
        assert!(!settings.confirm_transitions);
        assert!(settings.confirm_discard_changes);
        assert!(settings.custom_theme.is_none());
    }

    #[test]
    fn test_settings_serialization() {
        let settings = Settings {
            default_profile: Some("work".to_string()),
            theme: "light".to_string(),
            vim_mode: false,
            cache_ttl_minutes: 60,
            cache_max_size_mb: 200,
            jql_history: vec!["project = TEST".to_string()],
            confirm_transitions: true,
            confirm_discard_changes: false,
            custom_theme: None,
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
        assert_eq!(settings.cache_max_size_mb, 100); // default
        assert!(settings.jql_history.is_empty()); // default
        assert!(!settings.confirm_transitions); // default
        assert!(settings.confirm_discard_changes); // default
    }

    #[test]
    fn test_empty_settings_uses_defaults() {
        let toml_content = "";
        let settings: Settings = toml::from_str(toml_content).unwrap();

        assert!(settings.default_profile.is_none());
        assert_eq!(settings.theme, "dark");
        assert!(settings.vim_mode);
        assert_eq!(settings.cache_ttl_minutes, 30);
        assert_eq!(settings.cache_max_size_mb, 100);
        assert!(settings.jql_history.is_empty());
        assert!(!settings.confirm_transitions);
        assert!(settings.confirm_discard_changes);
    }

    #[test]
    fn test_confirmation_settings() {
        let toml_content = r#"
confirm_transitions = true
confirm_discard_changes = false
"#;

        let settings: Settings = toml::from_str(toml_content).unwrap();
        assert!(settings.confirm_transitions);
        assert!(!settings.confirm_discard_changes);
    }

    #[test]
    fn test_add_jql_to_history() {
        let mut settings = Settings::default();

        settings.add_jql_to_history("query1".to_string());
        assert_eq!(settings.jql_history, vec!["query1"]);

        settings.add_jql_to_history("query2".to_string());
        assert_eq!(settings.jql_history, vec!["query2", "query1"]);
    }

    #[test]
    fn test_add_jql_to_history_deduplication() {
        let mut settings = Settings::default();

        settings.add_jql_to_history("query1".to_string());
        settings.add_jql_to_history("query2".to_string());
        settings.add_jql_to_history("query1".to_string());

        // query1 should be moved to front
        assert_eq!(settings.jql_history, vec!["query1", "query2"]);
    }

    #[test]
    fn test_add_jql_to_history_max_size() {
        let mut settings = Settings::default();

        // Add more than MAX_JQL_HISTORY items
        for i in 0..15 {
            settings.add_jql_to_history(format!("query{}", i));
        }

        // Should be limited to MAX_JQL_HISTORY (10)
        assert_eq!(settings.jql_history.len(), 10);
        // Most recent should be first
        assert_eq!(settings.jql_history[0], "query14");
    }

    #[test]
    fn test_custom_theme_config() {
        let toml_content = r##"
theme = "dark"

[custom_theme]
accent = "#ff00ff"
success = "lightgreen"
"##;

        let settings: Settings = toml::from_str(toml_content).unwrap();
        assert_eq!(settings.theme, "dark");
        assert!(settings.custom_theme.is_some());
        let custom = settings.custom_theme.unwrap();
        assert_eq!(custom.accent, Some("#ff00ff".to_string()));
        assert_eq!(custom.success, Some("lightgreen".to_string()));
        assert!(custom.error.is_none());
    }
}
