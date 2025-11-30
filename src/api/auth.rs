//! Authentication handling for JIRA API.

/// Authentication credentials for JIRA.
pub struct Auth {
    /// The user's email address.
    pub email: String,
    /// The API token.
    pub token: String,
}

impl Auth {
    /// Create new authentication credentials.
    pub fn new(email: String, token: String) -> Self {
        Self { email, token }
    }
}
