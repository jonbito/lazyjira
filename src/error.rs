//! Centralized error types for LazyJira.
//!
//! This module provides a unified error hierarchy for the application with
//! user-friendly error messages. All error types use `thiserror` for
//! ergonomic error handling.

use thiserror::Error;

use crate::api::error::ApiError;
use crate::config::ConfigError;

/// The main application error type.
///
/// This enum aggregates all error types that can occur in LazyJira,
/// providing user-friendly error messages while preserving the underlying
/// error context for debugging.
#[derive(Debug, Error)]
pub enum AppError {
    /// Configuration-related errors.
    #[error("{0}")]
    Config(#[from] ConfigError),

    /// API-related errors.
    #[error("{0}")]
    Api(#[from] ApiError),

    /// IO errors (file system, etc.).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Terminal-related errors.
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Generic errors with a message.
    #[error("{0}")]
    Other(String),
}

impl AppError {
    /// Create a terminal error.
    pub fn terminal(msg: impl Into<String>) -> Self {
        AppError::Terminal(msg.into())
    }

    /// Create a generic error.
    pub fn other(msg: impl Into<String>) -> Self {
        AppError::Other(msg.into())
    }

    /// Get a user-friendly message for display.
    ///
    /// This returns a message suitable for showing to users in the UI,
    /// without technical jargon or stack traces.
    pub fn user_message(&self) -> String {
        match self {
            AppError::Config(e) => match e {
                ConfigError::NoConfigDir => {
                    "Could not find configuration directory. Please check your system settings."
                        .to_string()
                }
                ConfigError::CreateDirError(_) => {
                    "Could not create configuration directory. Check file permissions.".to_string()
                }
                ConfigError::ReadError(_) => {
                    "Could not read configuration file. Please check the file exists and is readable.".to_string()
                }
                ConfigError::WriteError(_) => {
                    "Could not save configuration. Please check file permissions.".to_string()
                }
                ConfigError::ParseError(_) => {
                    "Configuration file is invalid. Please check the file format.".to_string()
                }
                ConfigError::SerializeError(_) => {
                    "Could not save configuration. Internal error.".to_string()
                }
                ConfigError::ValidationError(msg) => format!("Configuration error: {}", msg),
                ConfigError::ProfileNotFound(name) => {
                    format!("Profile '{}' not found.", name)
                }
            },
            AppError::Api(e) => match e {
                ApiError::Unauthorized => {
                    "Authentication failed. Please check your email and API token.".to_string()
                }
                ApiError::Forbidden => {
                    "Access denied. You don't have permission to access this resource.".to_string()
                }
                ApiError::NotFound(resource) => format!("'{}' was not found.", resource),
                ApiError::RateLimited => {
                    "Too many requests. Please wait a moment and try again.".to_string()
                }
                ApiError::ServerError(_) => {
                    "JIRA server error. Please try again later.".to_string()
                }
                ApiError::Network(_) => {
                    "Connection failed. Please check your internet connection.".to_string()
                }
                ApiError::InvalidUrl(_) => "Invalid JIRA URL in configuration.".to_string(),
                ApiError::Keyring(_) => {
                    "Could not access secure storage. Please reconfigure your profile.".to_string()
                }
                ApiError::InvalidResponse(_) => {
                    "Unexpected response from JIRA. Please try again.".to_string()
                }
                ApiError::ConnectionFailed(_) => {
                    "Could not connect to JIRA. Please check your URL and network.".to_string()
                }
                ApiError::UpdateFailed(msg) => format!("Failed to update issue: {}", msg),
                ApiError::TransitionFailed(msg) => format!("Failed to change issue status: {}", msg),
                ApiError::Conflict => {
                    "This issue was modified by someone else. Please refresh and try again."
                        .to_string()
                }
                ApiError::PermissionDenied => {
                    "You don't have permission to modify this issue.".to_string()
                }
            },
            AppError::Io(_) => "A file operation failed. Please check file permissions.".to_string(),
            AppError::Terminal(msg) => format!("Terminal error: {}", msg),
            AppError::Other(msg) => msg.clone(),
        }
    }

    /// Check if this error is critical and requires user acknowledgment.
    ///
    /// Critical errors typically indicate issues that prevent the application
    /// from functioning correctly, such as configuration or authentication problems.
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            AppError::Config(_)
                | AppError::Api(ApiError::Unauthorized)
                | AppError::Api(ApiError::Forbidden)
                | AppError::Terminal(_)
        )
    }

    /// Check if this error is recoverable.
    ///
    /// Recoverable errors can be retried or the user can continue working.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            AppError::Api(ApiError::RateLimited)
                | AppError::Api(ApiError::ServerError(_))
                | AppError::Api(ApiError::Network(_))
                | AppError::Api(ApiError::NotFound(_))
                | AppError::Api(ApiError::UpdateFailed(_))
                | AppError::Api(ApiError::TransitionFailed(_))
                | AppError::Api(ApiError::Conflict)
        )
    }

    /// Get a suggested action for the user.
    pub fn suggested_action(&self) -> Option<&'static str> {
        match self {
            AppError::Config(ConfigError::NoConfigDir) | AppError::Config(ConfigError::ReadError(_)) => {
                Some("Run 'lazyjira setup' to create a configuration file.")
            }
            AppError::Api(ApiError::Unauthorized) => {
                Some("Check your API token at https://id.atlassian.com/manage-profile/security/api-tokens")
            }
            AppError::Api(ApiError::RateLimited) => Some("Wait a few seconds and press 'r' to refresh."),
            AppError::Api(ApiError::Network(_)) | AppError::Api(ApiError::ConnectionFailed(_)) => {
                Some("Check your internet connection and JIRA URL.")
            }
            _ => None,
        }
    }
}

/// Result type for application operations.
pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_from_config_error() {
        let config_err = ConfigError::NoConfigDir;
        let app_err: AppError = config_err.into();
        assert!(matches!(
            app_err,
            AppError::Config(ConfigError::NoConfigDir)
        ));
    }

    #[test]
    fn test_app_error_from_api_error() {
        let api_err = ApiError::Unauthorized;
        let app_err: AppError = api_err.into();
        assert!(matches!(app_err, AppError::Api(ApiError::Unauthorized)));
    }

    #[test]
    fn test_user_message_unauthorized() {
        let err = AppError::Api(ApiError::Unauthorized);
        let msg = err.user_message();
        assert!(msg.contains("Authentication failed"));
        assert!(msg.contains("email"));
        assert!(msg.contains("API token"));
    }

    #[test]
    fn test_user_message_not_found() {
        let err = AppError::Api(ApiError::NotFound("PROJ-123".to_string()));
        let msg = err.user_message();
        assert!(msg.contains("PROJ-123"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_user_message_connection_failed() {
        let err = AppError::Api(ApiError::ConnectionFailed("network error".to_string()));
        let msg = err.user_message();
        assert!(msg.contains("Could not connect to JIRA"));
    }

    #[test]
    fn test_user_message_config_validation() {
        let err = AppError::Config(ConfigError::ValidationError(
            "duplicate profile".to_string(),
        ));
        let msg = err.user_message();
        assert!(msg.contains("duplicate profile"));
    }

    #[test]
    fn test_is_critical_unauthorized() {
        let err = AppError::Api(ApiError::Unauthorized);
        assert!(err.is_critical());
    }

    #[test]
    fn test_is_critical_forbidden() {
        let err = AppError::Api(ApiError::Forbidden);
        assert!(err.is_critical());
    }

    #[test]
    fn test_is_critical_config() {
        let err = AppError::Config(ConfigError::NoConfigDir);
        assert!(err.is_critical());
    }

    #[test]
    fn test_is_not_critical_rate_limited() {
        let err = AppError::Api(ApiError::RateLimited);
        assert!(!err.is_critical());
    }

    #[test]
    fn test_is_recoverable_rate_limited() {
        let err = AppError::Api(ApiError::RateLimited);
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_is_recoverable_not_found() {
        let err = AppError::Api(ApiError::NotFound("TEST".to_string()));
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_is_not_recoverable_unauthorized() {
        let err = AppError::Api(ApiError::Unauthorized);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_suggested_action_unauthorized() {
        let err = AppError::Api(ApiError::Unauthorized);
        let action = err.suggested_action();
        assert!(action.is_some());
        assert!(action.unwrap().contains("api-tokens"));
    }

    #[test]
    fn test_suggested_action_rate_limited() {
        let err = AppError::Api(ApiError::RateLimited);
        let action = err.suggested_action();
        assert!(action.is_some());
        assert!(action.unwrap().contains("refresh"));
    }

    #[test]
    fn test_terminal_error() {
        let err = AppError::terminal("test error");
        assert!(matches!(err, AppError::Terminal(_)));
        assert_eq!(err.user_message(), "Terminal error: test error");
    }

    #[test]
    fn test_other_error() {
        let err = AppError::other("something went wrong");
        assert!(matches!(err, AppError::Other(_)));
        assert_eq!(err.user_message(), "something went wrong");
    }
}
