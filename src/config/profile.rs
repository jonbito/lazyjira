//! JIRA profile configuration.

use serde::{Deserialize, Serialize};

/// A JIRA profile configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// The name of this profile.
    pub name: String,
    /// The JIRA instance URL.
    pub url: String,
    /// The user's email address.
    pub email: String,
}

impl Profile {
    /// Create a new profile.
    pub fn new(name: String, url: String, email: String) -> Self {
        Self { name, url, email }
    }
}
