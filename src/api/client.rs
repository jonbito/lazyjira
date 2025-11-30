//! JIRA API client implementation.

use super::auth::Auth;

/// The JIRA API client.
pub struct JiraClient {
    /// The base URL for the JIRA instance.
    pub base_url: String,
    /// Authentication credentials.
    pub auth: Auth,
}

impl JiraClient {
    /// Create a new JIRA client.
    pub fn new(base_url: String, auth: Auth) -> Self {
        Self { base_url, auth }
    }
}
