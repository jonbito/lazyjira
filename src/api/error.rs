//! API error types for JIRA client.

use thiserror::Error;

/// Errors that can occur when interacting with the JIRA API.
#[derive(Debug, Error)]
pub enum ApiError {
    /// Authentication failed - invalid email or API token.
    #[error("Authentication failed: check your email and API token")]
    Unauthorized,

    /// Permission denied - user lacks access to the resource.
    #[error("Permission denied: you don't have access to this resource")]
    Forbidden,

    /// Resource not found.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Rate limited by the JIRA API.
    #[error("Rate limited: please wait before retrying")]
    RateLimited,

    /// JIRA server error.
    #[error("JIRA server error: {0}")]
    ServerError(String),

    /// Network or HTTP error.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Invalid URL.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Keyring error when storing/retrieving tokens.
    #[error("Keyring error: {0}")]
    Keyring(String),

    /// Invalid response from the API.
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    /// Connection validation failed.
    #[error("Connection validation failed: {0}")]
    ConnectionFailed(String),

    /// Failed to update an issue.
    #[error("Failed to update issue: {0}")]
    UpdateFailed(String),

    /// Failed to transition an issue.
    #[error("Failed to transition issue: {0}")]
    TransitionFailed(String),

    /// Conflict error - issue was modified by another user.
    #[error("Conflict: issue was modified by another user. Please refresh and try again")]
    Conflict,

    /// Permission denied for modifying this issue.
    #[error("You don't have permission to modify this issue")]
    PermissionDenied,
}

/// Result type for API operations.
pub type Result<T> = std::result::Result<T, ApiError>;

impl ApiError {
    /// Create an error from an HTTP status code.
    pub fn from_status(status: reqwest::StatusCode, context: &str) -> Self {
        match status.as_u16() {
            401 => ApiError::Unauthorized,
            403 => ApiError::Forbidden,
            404 => ApiError::NotFound(context.to_string()),
            409 => ApiError::Conflict,
            429 => ApiError::RateLimited,
            500..=599 => ApiError::ServerError(format!("HTTP {}: {}", status, context)),
            _ => ApiError::ServerError(format!("Unexpected HTTP {}: {}", status, context)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    #[test]
    fn test_error_from_status_401() {
        let err = ApiError::from_status(StatusCode::UNAUTHORIZED, "test");
        assert!(matches!(err, ApiError::Unauthorized));
    }

    #[test]
    fn test_error_from_status_403() {
        let err = ApiError::from_status(StatusCode::FORBIDDEN, "test");
        assert!(matches!(err, ApiError::Forbidden));
    }

    #[test]
    fn test_error_from_status_404() {
        let err = ApiError::from_status(StatusCode::NOT_FOUND, "issue PROJ-123");
        match err {
            ApiError::NotFound(msg) => assert_eq!(msg, "issue PROJ-123"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_error_from_status_429() {
        let err = ApiError::from_status(StatusCode::TOO_MANY_REQUESTS, "test");
        assert!(matches!(err, ApiError::RateLimited));
    }

    #[test]
    fn test_error_from_status_500() {
        let err = ApiError::from_status(StatusCode::INTERNAL_SERVER_ERROR, "test");
        assert!(matches!(err, ApiError::ServerError(_)));
    }

    #[test]
    fn test_error_from_status_409() {
        let err = ApiError::from_status(StatusCode::CONFLICT, "test");
        assert!(matches!(err, ApiError::Conflict));
    }

    #[test]
    fn test_error_display() {
        let err = ApiError::Unauthorized;
        assert_eq!(
            err.to_string(),
            "Authentication failed: check your email and API token"
        );

        let err = ApiError::NotFound("PROJ-123".to_string());
        assert_eq!(err.to_string(), "Resource not found: PROJ-123");
    }
}
