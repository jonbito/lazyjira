//! JIRA API client implementation.
//!
//! This module provides the main client for interacting with the JIRA REST API v3.
//! It handles authentication, request/response processing, error handling, and retry logic.

use std::time::Duration;

use reqwest::{header, Client, Response, StatusCode};
use tracing::{debug, error, info, instrument, warn};

use super::auth::Auth;
use super::error::{ApiError, Result};
use super::types::{CurrentUser, Issue, SearchResult};
use crate::config::Profile;

/// Default request timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Maximum number of retries for transient failures.
const MAX_RETRIES: u32 = 3;

/// Base delay between retries in milliseconds.
const RETRY_DELAY_MS: u64 = 1000;

/// The JIRA API client.
///
/// Provides async methods for interacting with the JIRA REST API v3.
/// Handles authentication, error handling, and retry logic for transient failures.
#[derive(Debug)]
pub struct JiraClient {
    /// The HTTP client.
    client: Client,
    /// The base URL for the JIRA instance.
    base_url: String,
    /// Authentication credentials.
    auth: Auth,
}

impl JiraClient {
    /// Create a new JIRA client from a profile.
    ///
    /// Retrieves the API token from the OS keyring and validates the connection.
    ///
    /// # Arguments
    ///
    /// * `profile` - The profile configuration containing URL and email
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The token cannot be retrieved from the keyring
    /// - The HTTP client cannot be built
    /// - Connection validation fails
    #[instrument(skip(profile), fields(profile_name = %profile.name))]
    pub async fn new(profile: &Profile) -> Result<Self> {
        info!("Creating JIRA client for profile");

        let auth = Auth::from_keyring(&profile.name, &profile.email)?;

        let client = Self::build_http_client()?;

        let base_url = normalize_base_url(&profile.url);

        let jira = Self {
            client,
            base_url,
            auth,
        };

        // Validate connection
        jira.validate_connection().await?;

        info!("JIRA client created and connection validated");
        Ok(jira)
    }

    /// Create a new JIRA client with explicit credentials.
    ///
    /// Use this for testing or when credentials are provided directly.
    /// Does NOT validate the connection automatically.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The JIRA instance URL
    /// * `email` - The user's email address
    /// * `token` - The API token
    pub fn with_credentials(base_url: &str, email: &str, token: &str) -> Result<Self> {
        let auth = Auth::new(email, token);
        let client = Self::build_http_client()?;
        let base_url = normalize_base_url(base_url);

        Ok(Self {
            client,
            base_url,
            auth,
        })
    }

    /// Build the HTTP client with appropriate settings.
    fn build_http_client() -> Result<Client> {
        Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(ApiError::Network)
    }

    /// Validate the connection by calling the /myself endpoint.
    ///
    /// This verifies that:
    /// - The URL is reachable
    /// - The credentials are valid
    /// - The user has access to the JIRA instance
    #[instrument(skip(self))]
    pub async fn validate_connection(&self) -> Result<CurrentUser> {
        debug!("Validating JIRA connection");

        let user = self.get_current_user().await.map_err(|e| {
            error!("Connection validation failed: {}", e);
            match e {
                ApiError::Unauthorized => e,
                ApiError::Network(ref _err) => {
                    ApiError::ConnectionFailed(format!("Cannot connect to {}: {}", self.base_url, e))
                }
                _ => ApiError::ConnectionFailed(e.to_string()),
            }
        })?;

        info!("Connected as user: {}", user.display_name);
        Ok(user)
    }

    /// Get the current authenticated user.
    ///
    /// Calls `GET /rest/api/3/myself` to retrieve user information.
    #[instrument(skip(self))]
    pub async fn get_current_user(&self) -> Result<CurrentUser> {
        let url = format!("{}/rest/api/3/myself", self.base_url);
        let response: CurrentUser = self.get(&url).await?;
        Ok(response)
    }

    /// Search for issues using JQL.
    ///
    /// # Arguments
    ///
    /// * `jql` - The JQL query string
    /// * `start_at` - The index of the first issue to return (0-based)
    /// * `max_results` - Maximum number of issues to return (max 100)
    ///
    /// # Returns
    ///
    /// A `SearchResult` containing the matching issues and pagination info.
    #[instrument(skip(self), fields(jql = %jql))]
    pub async fn search_issues(
        &self,
        jql: &str,
        start_at: u32,
        max_results: u32,
    ) -> Result<SearchResult> {
        debug!("Searching issues: startAt={}, maxResults={}", start_at, max_results);

        let url = format!(
            "{}/rest/api/3/search?jql={}&startAt={}&maxResults={}",
            self.base_url,
            urlencoding::encode(jql),
            start_at,
            max_results.min(100) // JIRA limits to 100
        );

        let result: SearchResult = self.get(&url).await?;
        debug!("Found {} issues (total: {})", result.issues.len(), result.total);
        Ok(result)
    }

    /// Get a single issue by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    ///
    /// # Returns
    ///
    /// The issue details.
    #[instrument(skip(self), fields(issue_key = %key))]
    pub async fn get_issue(&self, key: &str) -> Result<Issue> {
        debug!("Fetching issue");

        let url = format!("{}/rest/api/3/issue/{}", self.base_url, key);
        let issue: Issue = self.get(&url).await.map_err(|e| {
            if matches!(e, ApiError::NotFound(_)) {
                ApiError::NotFound(format!("Issue '{}' not found", key))
            } else {
                e
            }
        })?;

        debug!("Fetched issue: {}", issue.key);
        Ok(issue)
    }

    /// Perform a GET request with authentication and error handling.
    ///
    /// Includes retry logic for transient failures (rate limiting, server errors).
    #[instrument(skip(self), fields(url = %url))]
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let mut attempts = 0;
        let mut last_error: Option<ApiError> = None;

        while attempts < MAX_RETRIES {
            attempts += 1;
            debug!("Request attempt {}/{}", attempts, MAX_RETRIES);

            match self.execute_get::<T>(url).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if Self::is_retryable(&e) && attempts < MAX_RETRIES {
                        let delay = Self::calculate_retry_delay(attempts);
                        warn!(
                            "Request failed (attempt {}), retrying in {}ms: {}",
                            attempts, delay, e
                        );
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        last_error = Some(e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Err(last_error.unwrap_or(ApiError::ServerError("Max retries exceeded".to_string())))
    }

    /// Execute a single GET request.
    async fn execute_get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self
            .client
            .get(url)
            .header(header::AUTHORIZATION, self.auth.header_value())
            .header(header::ACCEPT, "application/json")
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Handle the HTTP response, checking for errors and parsing JSON.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T> {
        let status = response.status();
        let url = response.url().to_string();

        if status.is_success() {
            response
                .json::<T>()
                .await
                .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse response: {}", e)))
        } else {
            // Try to get error details from response body
            let error_body = response.text().await.unwrap_or_default();
            debug!("Error response body: {}", error_body);

            Err(Self::error_from_response(status, &url, &error_body))
        }
    }

    /// Create an appropriate error from an HTTP response.
    fn error_from_response(status: StatusCode, url: &str, body: &str) -> ApiError {
        // Try to extract JIRA error message from response
        let context = if body.is_empty() {
            url.to_string()
        } else {
            // JIRA often returns JSON with errorMessages
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
                if let Some(messages) = json.get("errorMessages") {
                    if let Some(arr) = messages.as_array() {
                        if !arr.is_empty() {
                            return ApiError::from_status(
                                status,
                                &arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", "),
                            );
                        }
                    }
                }
                if let Some(errors) = json.get("errors") {
                    if let Some(obj) = errors.as_object() {
                        let error_strings: Vec<String> = obj
                            .iter()
                            .map(|(k, v)| format!("{}: {}", k, v))
                            .collect();
                        if !error_strings.is_empty() {
                            return ApiError::from_status(status, &error_strings.join(", "));
                        }
                    }
                }
            }
            url.to_string()
        };

        ApiError::from_status(status, &context)
    }

    /// Check if an error is retryable.
    fn is_retryable(error: &ApiError) -> bool {
        matches!(
            error,
            ApiError::RateLimited | ApiError::ServerError(_) | ApiError::Network(_)
        )
    }

    /// Calculate retry delay with exponential backoff.
    fn calculate_retry_delay(attempt: u32) -> u64 {
        RETRY_DELAY_MS * 2u64.pow(attempt - 1)
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Normalize the base URL by removing trailing slashes and ensuring HTTPS.
fn normalize_base_url(url: &str) -> String {
    let url = url.trim_end_matches('/');

    // Warn if not HTTPS (but don't enforce for localhost/testing)
    if !url.starts_with("https://") && !url.contains("localhost") {
        warn!("URL does not use HTTPS: {}. This is insecure for production use.", url);
    }

    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_base_url_removes_trailing_slash() {
        assert_eq!(
            normalize_base_url("https://company.atlassian.net/"),
            "https://company.atlassian.net"
        );
    }

    #[test]
    fn test_normalize_base_url_handles_multiple_slashes() {
        assert_eq!(
            normalize_base_url("https://company.atlassian.net///"),
            "https://company.atlassian.net"
        );
    }

    #[test]
    fn test_normalize_base_url_preserves_path() {
        assert_eq!(
            normalize_base_url("https://company.atlassian.net/jira/"),
            "https://company.atlassian.net/jira"
        );
    }

    #[test]
    fn test_is_retryable_rate_limited() {
        assert!(JiraClient::is_retryable(&ApiError::RateLimited));
    }

    #[test]
    fn test_is_retryable_server_error() {
        assert!(JiraClient::is_retryable(&ApiError::ServerError(
            "test".to_string()
        )));
    }

    #[test]
    fn test_is_not_retryable_unauthorized() {
        assert!(!JiraClient::is_retryable(&ApiError::Unauthorized));
    }

    #[test]
    fn test_is_not_retryable_not_found() {
        assert!(!JiraClient::is_retryable(&ApiError::NotFound(
            "test".to_string()
        )));
    }

    #[test]
    fn test_retry_delay_exponential() {
        assert_eq!(JiraClient::calculate_retry_delay(1), 1000);
        assert_eq!(JiraClient::calculate_retry_delay(2), 2000);
        assert_eq!(JiraClient::calculate_retry_delay(3), 4000);
    }
}
