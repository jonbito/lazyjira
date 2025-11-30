//! Authentication handling for JIRA API.
//!
//! This module handles authentication with JIRA using Basic Auth
//! (email + API token) and secure token storage via the OS keyring.
//!
//! # Security
//!
//! This module is designed to prevent sensitive data leakage:
//! - Tokens are never stored in plain text after being encoded
//! - The `Auth` struct implements `Debug` manually to redact sensitive fields
//! - Logging functions use `#[instrument(skip(token))]` to avoid logging tokens

use std::fmt;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use tracing::{debug, info, instrument, warn};

use super::error::{ApiError, Result};

/// The keyring service name for LazyJira tokens.
const KEYRING_SERVICE: &str = "lazyjira";

/// Authentication credentials for JIRA.
///
/// This struct stores the email and encoded authorization header.
/// The raw token is never stored to minimize exposure.
#[derive(Clone)]
pub struct Auth {
    /// The user's email address.
    email: String,
    /// The Base64-encoded authorization header value.
    /// This is intentionally not exposed in Debug output.
    auth_header: String,
}

// Custom Debug implementation to prevent leaking the auth header
impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Auth")
            .field("email", &self.email)
            .field("auth_header", &"[REDACTED]")
            .finish()
    }
}

impl Auth {
    /// Create new authentication credentials from email and token.
    ///
    /// The token is immediately encoded and the raw token is not stored.
    pub fn new(email: &str, token: &str) -> Self {
        let auth_header = build_auth_header(email, token);
        Self {
            email: email.to_string(),
            auth_header,
        }
    }

    /// Create authentication from a profile using the OS keyring.
    ///
    /// Retrieves the API token from the OS keyring using the profile name.
    ///
    /// # Errors
    ///
    /// Returns an error if the token cannot be retrieved from the keyring.
    #[instrument(skip(email), fields(profile = %profile_name))]
    pub fn from_keyring(profile_name: &str, email: &str) -> Result<Self> {
        debug!("Creating auth from keyring");
        let token = get_token(profile_name)?;
        Ok(Self::new(email, &token))
    }

    /// Get the authorization header value for HTTP requests.
    ///
    /// Returns the complete "Basic ..." header value.
    pub fn header_value(&self) -> &str {
        &self.auth_header
    }

    /// Get the email address.
    pub fn email(&self) -> &str {
        &self.email
    }
}

/// Build the Basic Auth header value.
///
/// Encodes "email:token" in Base64 and prepends "Basic ".
fn build_auth_header(email: &str, token: &str) -> String {
    let credentials = format!("{}:{}", email, token);
    let encoded = BASE64.encode(credentials.as_bytes());
    format!("Basic {}", encoded)
}

/// Store an API token in the OS keyring.
///
/// # Arguments
///
/// * `profile_name` - The profile name to use as the keyring username
/// * `token` - The API token to store
///
/// # Errors
///
/// Returns an error if the token cannot be stored in the keyring.
#[instrument(skip(token), fields(profile = %profile_name))]
pub fn store_token(profile_name: &str, token: &str) -> Result<()> {
    debug!("Storing token in keyring");

    let entry = keyring::Entry::new(KEYRING_SERVICE, profile_name)
        .map_err(|e| ApiError::Keyring(format!("failed to create keyring entry: {}", e)))?;

    entry
        .set_password(token)
        .map_err(|e| {
            warn!("Failed to store token in keyring");
            ApiError::Keyring(format!("failed to store token: {}", e))
        })?;

    info!("Token stored successfully");
    Ok(())
}

/// Retrieve an API token from the OS keyring.
///
/// # Arguments
///
/// * `profile_name` - The profile name to use as the keyring username
///
/// # Errors
///
/// Returns an error if the token cannot be retrieved from the keyring.
#[instrument(fields(profile = %profile_name))]
pub fn get_token(profile_name: &str) -> Result<String> {
    debug!("Retrieving token from keyring");

    let entry = keyring::Entry::new(KEYRING_SERVICE, profile_name)
        .map_err(|e| ApiError::Keyring(format!("failed to access keyring: {}", e)))?;

    entry
        .get_password()
        .map_err(|e| {
            warn!("Failed to retrieve token from keyring");
            ApiError::Keyring(format!("failed to retrieve token: {}", e))
        })
}

/// Delete an API token from the OS keyring.
///
/// # Arguments
///
/// * `profile_name` - The profile name to use as the keyring username
///
/// # Errors
///
/// Returns an error if the token cannot be deleted from the keyring.
#[instrument(fields(profile = %profile_name))]
pub fn delete_token(profile_name: &str) -> Result<()> {
    debug!("Deleting token from keyring");

    let entry = keyring::Entry::new(KEYRING_SERVICE, profile_name)
        .map_err(|e| ApiError::Keyring(format!("failed to access keyring: {}", e)))?;

    entry
        .delete_password()
        .map_err(|e| {
            warn!("Failed to delete token from keyring");
            ApiError::Keyring(format!("failed to delete token: {}", e))
        })?;

    info!("Token deleted successfully");
    Ok(())
}

/// Check if a token exists in the OS keyring for a profile.
///
/// # Arguments
///
/// * `profile_name` - The profile name to check
///
/// # Returns
///
/// `true` if a token exists, `false` otherwise.
pub fn has_token(profile_name: &str) -> bool {
    get_token(profile_name).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_auth_header() {
        // Test case from Atlassian docs
        let header = build_auth_header("user@example.com", "api_token_here");
        assert!(header.starts_with("Basic "));

        // Decode and verify
        let encoded = header.strip_prefix("Basic ").unwrap();
        let decoded = BASE64.decode(encoded).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        assert_eq!(decoded_str, "user@example.com:api_token_here");
    }

    #[test]
    fn test_auth_new() {
        let auth = Auth::new("user@example.com", "secret_token");
        assert_eq!(auth.email(), "user@example.com");
        assert!(auth.header_value().starts_with("Basic "));
    }

    #[test]
    fn test_auth_header_value_format() {
        let auth = Auth::new("test@test.com", "token123");
        let header = auth.header_value();

        // Should be valid Base64 after "Basic "
        let encoded = header.strip_prefix("Basic ").unwrap();
        assert!(BASE64.decode(encoded).is_ok());
    }

    #[test]
    fn test_auth_does_not_expose_token() {
        let auth = Auth::new("user@example.com", "secret_token");
        let debug_output = format!("{:?}", auth);

        // Token should not appear in debug output
        assert!(!debug_output.contains("secret_token"));
    }
}
