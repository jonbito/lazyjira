//! JIRA profile configuration.

use serde::{Deserialize, Serialize};

use super::{ConfigError, Result};

/// A JIRA profile configuration.
///
/// Profiles store connection details for a JIRA instance.
/// API tokens are stored separately in the OS keychain for security.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    /// The name of this profile.
    ///
    /// Must be non-empty and unique across all profiles.
    pub name: String,

    /// The JIRA instance URL.
    ///
    /// Should be a valid HTTPS URL (e.g., "https://company.atlassian.net").
    pub url: String,

    /// The user's email address.
    ///
    /// Used for JIRA API authentication along with the API token.
    pub email: String,
}

impl Profile {
    /// Create a new profile.
    pub fn new(name: String, url: String, email: String) -> Self {
        Self { name, url, email }
    }

    /// Validate this profile.
    ///
    /// Checks that:
    /// - The name is non-empty
    /// - The URL is non-empty and has a valid format
    /// - The email is non-empty and has a valid format
    ///
    /// # Errors
    ///
    /// Returns a `ConfigError::ValidationError` with details if validation fails.
    pub fn validate(&self) -> Result<()> {
        // Validate name
        if self.name.is_empty() {
            return Err(ConfigError::ValidationError(
                "profile name cannot be empty".to_string(),
            ));
        }

        if self.name.contains(char::is_whitespace) {
            return Err(ConfigError::ValidationError(format!(
                "profile name '{}' cannot contain whitespace",
                self.name
            )));
        }

        // Validate URL
        if self.url.is_empty() {
            return Err(ConfigError::ValidationError(format!(
                "profile '{}': URL cannot be empty",
                self.name
            )));
        }

        if !self.url.starts_with("https://") && !self.url.starts_with("http://") {
            return Err(ConfigError::ValidationError(format!(
                "profile '{}': URL must start with http:// or https://",
                self.name
            )));
        }

        // Validate email
        if self.email.is_empty() {
            return Err(ConfigError::ValidationError(format!(
                "profile '{}': email cannot be empty",
                self.name
            )));
        }

        if !self.email.contains('@') {
            return Err(ConfigError::ValidationError(format!(
                "profile '{}': '{}' does not appear to be a valid email address",
                self.name, self.email
            )));
        }

        Ok(())
    }

    /// Get the keyring service name for this profile's token.
    ///
    /// Used to store and retrieve API tokens from the OS keychain.
    pub fn keyring_service(&self) -> String {
        format!("lazyjira:{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new(
            "work".to_string(),
            "https://company.atlassian.net".to_string(),
            "user@company.com".to_string(),
        );

        assert_eq!(profile.name, "work");
        assert_eq!(profile.url, "https://company.atlassian.net");
        assert_eq!(profile.email, "user@company.com");
    }

    #[test]
    fn test_valid_profile() {
        let profile = Profile::new(
            "work".to_string(),
            "https://company.atlassian.net".to_string(),
            "user@company.com".to_string(),
        );

        assert!(profile.validate().is_ok());
    }

    #[test]
    fn test_empty_name_rejected() {
        let profile = Profile::new(
            "".to_string(),
            "https://company.atlassian.net".to_string(),
            "user@company.com".to_string(),
        );

        let result = profile.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("name cannot be empty"));
    }

    #[test]
    fn test_whitespace_name_rejected() {
        let profile = Profile::new(
            "my work".to_string(),
            "https://company.atlassian.net".to_string(),
            "user@company.com".to_string(),
        );

        let result = profile.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot contain whitespace"));
    }

    #[test]
    fn test_empty_url_rejected() {
        let profile = Profile::new(
            "work".to_string(),
            "".to_string(),
            "user@company.com".to_string(),
        );

        let result = profile.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("URL cannot be empty"));
    }

    #[test]
    fn test_invalid_url_scheme_rejected() {
        let profile = Profile::new(
            "work".to_string(),
            "company.atlassian.net".to_string(),
            "user@company.com".to_string(),
        );

        let result = profile.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must start with http"));
    }

    #[test]
    fn test_http_url_accepted() {
        let profile = Profile::new(
            "local".to_string(),
            "http://localhost:8080".to_string(),
            "user@company.com".to_string(),
        );

        assert!(profile.validate().is_ok());
    }

    #[test]
    fn test_empty_email_rejected() {
        let profile = Profile::new(
            "work".to_string(),
            "https://company.atlassian.net".to_string(),
            "".to_string(),
        );

        let result = profile.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("email cannot be empty"));
    }

    #[test]
    fn test_invalid_email_rejected() {
        let profile = Profile::new(
            "work".to_string(),
            "https://company.atlassian.net".to_string(),
            "not-an-email".to_string(),
        );

        let result = profile.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("valid email"));
    }

    #[test]
    fn test_keyring_service() {
        let profile = Profile::new(
            "work".to_string(),
            "https://company.atlassian.net".to_string(),
            "user@company.com".to_string(),
        );

        assert_eq!(profile.keyring_service(), "lazyjira:work");
    }

    #[test]
    fn test_profile_serialization() {
        let profile = Profile::new(
            "work".to_string(),
            "https://company.atlassian.net".to_string(),
            "user@company.com".to_string(),
        );

        let toml_str = toml::to_string(&profile).unwrap();
        let parsed: Profile = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed, profile);
    }
}
