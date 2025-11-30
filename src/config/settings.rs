//! Application settings configuration.

use serde::{Deserialize, Serialize};

/// Application-wide settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// The name of the default profile to use.
    pub default_profile: Option<String>,
    /// The UI theme to use.
    pub theme: String,
    /// Whether to use vim-style keybindings.
    pub vim_mode: bool,
    /// Cache time-to-live in minutes.
    pub cache_ttl_minutes: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_profile: None,
            theme: "dark".to_string(),
            vim_mode: true,
            cache_ttl_minutes: 30,
        }
    }
}
