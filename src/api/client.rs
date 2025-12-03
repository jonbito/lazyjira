//! JIRA API client implementation.
//!
//! This module provides the main client for interacting with the JIRA REST API v3.
//! It handles authentication, request/response processing, error handling, and retry logic.

use std::time::Duration;

use reqwest::{header, Client, Response, StatusCode};
use tracing::{debug, error, info, instrument, warn};

use super::auth::Auth;
use super::error::{ApiError, Result};
use super::types::{
    AddCommentRequest, BoardsResponse, Changelog, Comment, CommentsResponse,
    CreateIssueLinkRequest, CurrentUser, FieldUpdates, FilterOption, FilterOptions, Issue,
    IssueKeyRef, IssueLinkType, IssueLinkTypeRef, IssueLinkTypesResponse, IssuePickerResponse,
    IssueSuggestion, IssueUpdateRequest, LabelOperation, LabelsResponse, Priority, Project,
    SearchResult, SprintsResponse, Status, Transition, TransitionRef, TransitionRequest,
    TransitionsResponse, UpdateOperations, User,
};
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
#[derive(Debug, Clone)]
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
    #[allow(dead_code)]
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
                ApiError::Network(ref _err) => ApiError::ConnectionFailed(format!(
                    "Cannot connect to {}: {}",
                    self.base_url, e
                )),
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
    /// * `start_at` - The index of the first issue to return (0-based) - NOTE: deprecated, use next_page_token
    /// * `max_results` - Maximum number of issues to return (max 100)
    ///
    /// # Returns
    ///
    /// A `SearchResult` containing the matching issues and pagination info.
    #[instrument(skip(self), fields(jql = %jql))]
    pub async fn search_issues(
        &self,
        jql: &str,
        _start_at: u32,
        max_results: u32,
    ) -> Result<SearchResult> {
        self.search_issues_with_token(jql, max_results, None).await
    }

    /// Search for issues using JQL with pagination token.
    ///
    /// # Arguments
    ///
    /// * `jql` - The JQL query string
    /// * `max_results` - Maximum number of issues to return (max 100)
    /// * `next_page_token` - Optional token for pagination
    ///
    /// # Returns
    ///
    /// A `SearchResult` containing the matching issues and pagination info.
    #[instrument(skip(self), fields(jql = %jql))]
    pub async fn search_issues_with_token(
        &self,
        jql: &str,
        max_results: u32,
        next_page_token: Option<&str>,
    ) -> Result<SearchResult> {
        debug!(
            "Searching issues: maxResults={}, has_token={}",
            max_results,
            next_page_token.is_some()
        );

        let url = format!("{}/rest/api/3/search/jql", self.base_url);

        let mut body = serde_json::json!({
            "jql": jql,
            "maxResults": max_results.min(100),
            "fields": ["*all"]
        });

        if let Some(token) = next_page_token {
            body["nextPageToken"] = serde_json::Value::String(token.to_string());
        }

        let result: SearchResult = self.post(&url, &body).await?;
        debug!(
            "Found {} issues (total: {})",
            result.issues.len(),
            result.total
        );
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

        // Request all fields including issuelinks which isn't returned by default
        let url = format!(
            "{}/rest/api/3/issue/{}?fields=*all,-comment",
            self.base_url, key
        );
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

    /// Perform a POST request with authentication and error handling.
    ///
    /// Includes retry logic for transient failures (rate limiting, server errors).
    #[instrument(skip(self, body), fields(url = %url))]
    async fn post<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<T> {
        let mut attempts = 0;
        let mut last_error: Option<ApiError> = None;

        while attempts < MAX_RETRIES {
            attempts += 1;
            debug!("POST request attempt {}/{}", attempts, MAX_RETRIES);

            match self.execute_post::<T>(url, body).await {
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

    /// Execute a single POST request.
    async fn execute_post<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<T> {
        let response = self
            .client
            .post(url)
            .header(header::AUTHORIZATION, self.auth.header_value())
            .header(header::ACCEPT, "application/json")
            .header(header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response_with_debug(response).await
    }

    /// Handle response with debug logging for the raw body.
    async fn handle_response_with_debug<T: serde::de::DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T> {
        let status = response.status();
        let url = response.url().to_string();

        if status.is_success() {
            let body_text = response.text().await.map_err(|e| {
                ApiError::InvalidResponse(format!("Failed to read response body: {}", e))
            })?;

            debug!(
                "Response body (first 500 chars): {}",
                &body_text.chars().take(500).collect::<String>()
            );

            serde_json::from_str::<T>(&body_text).map_err(|e| {
                error!(
                    "Failed to parse JSON. Error: {}. Body preview: {}",
                    e,
                    &body_text.chars().take(1000).collect::<String>()
                );
                ApiError::InvalidResponse(format!("Failed to parse response: {}", e))
            })
        } else {
            let error_body = response.text().await.unwrap_or_default();
            debug!("Error response body: {}", error_body);
            Err(Self::error_from_response(status, &url, &error_body))
        }
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
                        let error_strings: Vec<String> =
                            obj.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
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

    /// Perform a PUT request with authentication and error handling.
    ///
    /// Includes retry logic for transient failures (rate limiting, server errors).
    #[instrument(skip(self, body), fields(url = %url))]
    async fn put<B: serde::Serialize + std::fmt::Debug>(&self, url: &str, body: &B) -> Result<()> {
        let mut attempts = 0;
        let mut last_error: Option<ApiError> = None;

        while attempts < MAX_RETRIES {
            attempts += 1;
            debug!("PUT request attempt {}/{}", attempts, MAX_RETRIES);

            match self.execute_put(url, body).await {
                Ok(()) => return Ok(()),
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

    /// Execute a single PUT request.
    async fn execute_put<B: serde::Serialize>(&self, url: &str, body: &B) -> Result<()> {
        let response = self
            .client
            .put(url)
            .header(header::AUTHORIZATION, self.auth.header_value())
            .header(header::ACCEPT, "application/json")
            .header(header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_empty_response(response).await
    }

    /// Perform a POST request that returns no body (204 No Content).
    #[instrument(skip(self, body), fields(url = %url))]
    async fn post_no_content<B: serde::Serialize + std::fmt::Debug>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<()> {
        let mut attempts = 0;
        let mut last_error: Option<ApiError> = None;

        while attempts < MAX_RETRIES {
            attempts += 1;
            debug!(
                "POST (no content) request attempt {}/{}",
                attempts, MAX_RETRIES
            );

            match self.execute_post_no_content(url, body).await {
                Ok(()) => return Ok(()),
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

    /// Execute a single POST request that returns no body.
    async fn execute_post_no_content<B: serde::Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<()> {
        let response = self
            .client
            .post(url)
            .header(header::AUTHORIZATION, self.auth.header_value())
            .header(header::ACCEPT, "application/json")
            .header(header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_empty_response(response).await
    }

    /// Handle an HTTP response that should have no body (204 No Content).
    async fn handle_empty_response(&self, response: Response) -> Result<()> {
        let status = response.status();
        let url = response.url().to_string();

        if status.is_success() {
            Ok(())
        } else {
            let error_body = response.text().await.unwrap_or_default();
            debug!("Error response body: {}", error_body);
            Err(Self::error_from_response(status, &url, &error_body))
        }
    }

    /// Perform a DELETE request with authentication and error handling.
    ///
    /// Includes retry logic for transient failures (rate limiting, server errors).
    #[instrument(skip(self), fields(url = %url))]
    async fn delete(&self, url: &str) -> Result<()> {
        let mut attempts = 0;
        let mut last_error: Option<ApiError> = None;

        while attempts < MAX_RETRIES {
            attempts += 1;
            debug!("DELETE request attempt {}/{}", attempts, MAX_RETRIES);

            match self.execute_delete(url).await {
                Ok(()) => return Ok(()),
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

    /// Execute a single DELETE request.
    async fn execute_delete(&self, url: &str) -> Result<()> {
        let response = self
            .client
            .delete(url)
            .header(header::AUTHORIZATION, self.auth.header_value())
            .header(header::ACCEPT, "application/json")
            .send()
            .await?;

        self.handle_empty_response(response).await
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
    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    // ========================================================================
    // Filter Options API Methods
    // ========================================================================

    /// Get all available statuses for the JIRA instance.
    #[instrument(skip(self))]
    pub async fn get_statuses(&self) -> Result<Vec<Status>> {
        debug!("Fetching statuses");
        let url = format!("{}/rest/api/3/status", self.base_url);
        let statuses: Vec<Status> = self.get(&url).await?;
        debug!("Found {} statuses", statuses.len());
        Ok(statuses)
    }

    /// Get all projects the user has access to.
    #[instrument(skip(self))]
    pub async fn get_projects(&self) -> Result<Vec<Project>> {
        debug!("Fetching projects");
        let url = format!("{}/rest/api/3/project", self.base_url);
        let projects: Vec<Project> = self.get(&url).await?;
        debug!("Found {} projects", projects.len());
        Ok(projects)
    }

    /// Search for users by query string.
    ///
    /// # Arguments
    ///
    /// * `query` - Search query for username or display name
    /// * `max_results` - Maximum number of results to return (default 50)
    #[allow(dead_code)]
    #[instrument(skip(self), fields(query = %query))]
    pub async fn search_users(&self, query: &str, max_results: u32) -> Result<Vec<User>> {
        debug!("Searching users");
        let url = format!(
            "{}/rest/api/3/user/search?query={}&maxResults={}",
            self.base_url,
            urlencoding::encode(query),
            max_results.min(100)
        );
        let users: Vec<User> = self.get(&url).await?;
        debug!("Found {} users", users.len());
        Ok(users)
    }

    /// Get assignable users for a project.
    ///
    /// # Arguments
    ///
    /// * `project_key` - The project key to get assignable users for
    #[instrument(skip(self), fields(project = %project_key))]
    pub async fn get_assignable_users(&self, project_key: &str) -> Result<Vec<User>> {
        debug!("Fetching assignable users for project");
        let url = format!(
            "{}/rest/api/3/user/assignable/search?project={}",
            self.base_url,
            urlencoding::encode(project_key)
        );
        let users: Vec<User> = self.get(&url).await?;
        debug!("Found {} assignable users", users.len());
        Ok(users)
    }

    /// Get all labels used in the JIRA instance.
    #[instrument(skip(self))]
    pub async fn get_labels(&self) -> Result<Vec<String>> {
        debug!("Fetching labels");
        let url = format!("{}/rest/api/3/label", self.base_url);
        let response: LabelsResponse = self.get(&url).await?;
        debug!("Found {} labels", response.values.len());
        Ok(response.values)
    }

    /// Get all components for a project.
    ///
    /// # Arguments
    ///
    /// * `project_key` - The project key (e.g., "PROJ")
    #[instrument(skip(self), fields(project = %project_key))]
    pub async fn get_project_components(
        &self,
        project_key: &str,
    ) -> Result<Vec<super::types::Component>> {
        debug!("Fetching components for project");
        let url = format!(
            "{}/rest/api/3/project/{}/components",
            self.base_url,
            urlencoding::encode(project_key)
        );
        let components: Vec<super::types::Component> = self.get(&url).await?;
        debug!("Found {} components", components.len());
        Ok(components)
    }

    /// Get all boards the user has access to.
    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub async fn get_boards(&self) -> Result<Vec<super::types::Board>> {
        debug!("Fetching boards");
        let url = format!("{}/rest/agile/1.0/board", self.base_url);
        let response: BoardsResponse = self.get(&url).await?;
        debug!("Found {} boards", response.values.len());
        Ok(response.values)
    }

    /// Get sprints for a board.
    ///
    /// # Arguments
    ///
    /// * `board_id` - The board ID to get sprints for
    /// * `state` - Optional filter by sprint state (active, future, closed)
    #[allow(dead_code)]
    #[instrument(skip(self), fields(board_id = board_id))]
    pub async fn get_sprints(
        &self,
        board_id: u64,
        state: Option<&str>,
    ) -> Result<Vec<super::types::Sprint>> {
        debug!("Fetching sprints for board");
        let mut url = format!("{}/rest/agile/1.0/board/{}/sprint", self.base_url, board_id);
        if let Some(state) = state {
            url.push_str(&format!("?state={}", state));
        }
        let response: SprintsResponse = self.get(&url).await?;
        debug!("Found {} sprints", response.values.len());
        Ok(response.values)
    }

    /// Fetch all epics.
    ///
    /// Returns a list of all epic issues (issue type = Epic).
    #[instrument(skip(self))]
    pub async fn get_epics(&self) -> Result<Vec<Issue>> {
        debug!("Fetching all epics");
        let result = self
            .search_issues("issuetype = Epic ORDER BY key ASC", 0, 100)
            .await?;
        debug!("Found {} epics", result.issues.len());
        Ok(result.issues)
    }

    /// Fetch all filter options in one call.
    ///
    /// This method fetches statuses, projects, labels, and epics.
    /// Users and sprints need to be fetched separately with project/board context.
    #[instrument(skip(self))]
    pub async fn get_filter_options(&self) -> Result<FilterOptions> {
        debug!("Fetching all filter options");

        // Fetch statuses, projects, labels, and epics sequentially
        let statuses = self.get_statuses().await.unwrap_or_default();
        let projects = self.get_projects().await.unwrap_or_default();
        let labels = self.get_labels().await.unwrap_or_default();
        let epics = self.get_epics().await.unwrap_or_default();

        let mut options = FilterOptions::new();

        // Convert statuses
        for status in statuses {
            options
                .statuses
                .push(FilterOption::new(&status.id, &status.name));
        }

        // Convert projects
        for project in projects {
            options
                .projects
                .push(FilterOption::new(&project.key, &project.name));
        }

        // Convert labels (ID and label are the same for labels)
        for label in labels {
            options.labels.push(FilterOption::new(&label, &label));
        }

        // Convert epics (key as ID, "key - summary" as label)
        for epic in epics {
            options.epics.push(FilterOption::new(
                &epic.key,
                format!("{} - {}", epic.key, epic.fields.summary),
            ));
        }

        debug!(
            "Loaded filter options: {} statuses, {} projects, {} labels, {} epics",
            options.statuses.len(),
            options.projects.len(),
            options.labels.len(),
            options.epics.len()
        );

        Ok(options)
    }

    // ========================================================================
    // Issue Update Operations
    // ========================================================================

    /// Update issue fields.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `update` - The update request containing field changes
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The issue doesn't exist
    /// - Permission is denied
    /// - A conflict occurs (issue was modified by another user)
    #[instrument(skip(self, update), fields(issue_key = %key))]
    pub async fn update_issue(&self, key: &str, update: IssueUpdateRequest) -> Result<()> {
        let url = format!("{}/rest/api/3/issue/{}", self.base_url, key);
        info!("Updating issue {}", key);

        self.put(&url, &update).await.map_err(|e| {
            error!("Failed to update issue {}: {}", key, e);
            match e {
                ApiError::NotFound(_) => ApiError::NotFound(format!("Issue '{}' not found", key)),
                ApiError::Forbidden => ApiError::PermissionDenied,
                other => ApiError::UpdateFailed(other.to_string()),
            }
        })?;

        info!("Successfully updated issue {}", key);
        Ok(())
    }

    /// Get available transitions for an issue.
    ///
    /// Returns the list of workflow transitions that can be performed on the issue
    /// based on its current status and the user's permissions.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    #[instrument(skip(self), fields(issue_key = %key))]
    pub async fn get_transitions(&self, key: &str) -> Result<Vec<Transition>> {
        debug!("Fetching transitions for issue {}", key);
        let url = format!("{}/rest/api/3/issue/{}/transitions", self.base_url, key);
        let response: TransitionsResponse = self.get(&url).await?;
        debug!("Found {} available transitions", response.transitions.len());
        Ok(response.transitions)
    }

    /// Perform a status transition on an issue.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `transition_id` - The ID of the transition to perform
    /// * `fields` - Optional fields to set during the transition
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transition is not valid for the issue's current status
    /// - Required fields are missing
    /// - Permission is denied
    #[instrument(skip(self, fields), fields(issue_key = %key, transition_id = %transition_id))]
    pub async fn transition_issue(
        &self,
        key: &str,
        transition_id: &str,
        fields: Option<FieldUpdates>,
    ) -> Result<()> {
        let url = format!("{}/rest/api/3/issue/{}/transitions", self.base_url, key);
        info!(
            "Transitioning issue {} via transition {}",
            key, transition_id
        );

        let request = TransitionRequest {
            transition: TransitionRef::new(transition_id),
            fields,
        };

        self.post_no_content(&url, &request).await.map_err(|e| {
            error!("Failed to transition issue {}: {}", key, e);
            match e {
                ApiError::NotFound(_) => ApiError::NotFound(format!("Issue '{}' not found", key)),
                ApiError::Forbidden => ApiError::PermissionDenied,
                other => ApiError::TransitionFailed(other.to_string()),
            }
        })?;

        info!("Successfully transitioned issue {}", key);
        Ok(())
    }

    /// Get all available priorities.
    ///
    /// Returns the list of priorities configured for the JIRA instance.
    #[instrument(skip(self))]
    pub async fn get_priorities(&self) -> Result<Vec<Priority>> {
        debug!("Fetching priorities");
        let url = format!("{}/rest/api/3/priority", self.base_url);
        let priorities: Vec<Priority> = self.get(&url).await?;
        debug!("Found {} priorities", priorities.len());
        Ok(priorities)
    }

    // ========================================================================
    // Convenience Update Methods
    // ========================================================================

    /// Update the issue summary.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `summary` - The new summary text
    #[allow(dead_code)]
    #[instrument(skip(self), fields(issue_key = %key))]
    pub async fn update_summary(&self, key: &str, summary: &str) -> Result<()> {
        let update = IssueUpdateRequest {
            fields: Some(FieldUpdates {
                summary: Some(summary.to_string()),
                ..Default::default()
            }),
            update: None,
        };
        self.update_issue(key, update).await
    }

    /// Update the issue assignee.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `account_id` - The account ID of the new assignee, or None to unassign
    #[instrument(skip(self), fields(issue_key = %key))]
    pub async fn update_assignee(&self, key: &str, account_id: Option<&str>) -> Result<()> {
        use super::types::NullableUserRef;

        let assignee = match account_id {
            Some(id) => NullableUserRef::assign(id),
            None => NullableUserRef::unassign(), // Explicitly serializes as null
        };

        let update = IssueUpdateRequest {
            fields: Some(FieldUpdates {
                assignee: Some(assignee),
                ..Default::default()
            }),
            update: None,
        };
        self.update_issue(key, update).await
    }

    /// Update the issue priority.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `priority_id` - The ID of the new priority
    #[instrument(skip(self), fields(issue_key = %key, priority_id = %priority_id))]
    pub async fn update_priority(&self, key: &str, priority_id: &str) -> Result<()> {
        let update = IssueUpdateRequest {
            fields: Some(FieldUpdates {
                priority: Some(super::types::PriorityRef::new(priority_id)),
                ..Default::default()
            }),
            update: None,
        };
        self.update_issue(key, update).await
    }

    /// Add labels to an issue.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `labels` - The labels to add
    #[instrument(skip(self, labels), fields(issue_key = %key))]
    pub async fn add_labels(&self, key: &str, labels: Vec<String>) -> Result<()> {
        let operations: Vec<LabelOperation> = labels.into_iter().map(LabelOperation::Add).collect();

        let update = IssueUpdateRequest {
            fields: None,
            update: Some(UpdateOperations {
                labels: Some(operations),
                ..Default::default()
            }),
        };
        self.update_issue(key, update).await
    }

    /// Remove labels from an issue.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `labels` - The labels to remove
    #[instrument(skip(self, labels), fields(issue_key = %key))]
    pub async fn remove_labels(&self, key: &str, labels: Vec<String>) -> Result<()> {
        let operations: Vec<LabelOperation> =
            labels.into_iter().map(LabelOperation::Remove).collect();

        let update = IssueUpdateRequest {
            fields: None,
            update: Some(UpdateOperations {
                labels: Some(operations),
                ..Default::default()
            }),
        };
        self.update_issue(key, update).await
    }

    /// Add components to an issue.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `components` - The component names to add
    #[instrument(skip(self, components), fields(issue_key = %key))]
    pub async fn add_components(&self, key: &str, components: Vec<String>) -> Result<()> {
        let operations: Vec<super::types::ComponentOperation> = components
            .into_iter()
            .map(|name| super::types::ComponentOperation::Add { name })
            .collect();

        let update = IssueUpdateRequest {
            fields: None,
            update: Some(UpdateOperations {
                components: Some(operations),
                ..Default::default()
            }),
        };
        self.update_issue(key, update).await
    }

    /// Remove components from an issue.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `components` - The component names to remove
    #[instrument(skip(self, components), fields(issue_key = %key))]
    pub async fn remove_components(&self, key: &str, components: Vec<String>) -> Result<()> {
        let operations: Vec<super::types::ComponentOperation> = components
            .into_iter()
            .map(|name| super::types::ComponentOperation::Remove { name })
            .collect();

        let update = IssueUpdateRequest {
            fields: None,
            update: Some(UpdateOperations {
                components: Some(operations),
                ..Default::default()
            }),
        };
        self.update_issue(key, update).await
    }

    /// Update story points for an issue.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `points` - The story points value
    #[allow(dead_code)]
    #[instrument(skip(self), fields(issue_key = %key, points = %points))]
    pub async fn update_story_points(&self, key: &str, points: f32) -> Result<()> {
        let update = IssueUpdateRequest {
            fields: Some(FieldUpdates {
                story_points: Some(points),
                ..Default::default()
            }),
            update: None,
        };
        self.update_issue(key, update).await
    }

    /// Update sprint assignment for an issue.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `sprint_id` - The sprint ID to assign, or None to remove from sprint
    #[allow(dead_code)]
    #[instrument(skip(self), fields(issue_key = %key))]
    pub async fn update_sprint(&self, key: &str, sprint_id: Option<i64>) -> Result<()> {
        let update = IssueUpdateRequest {
            fields: Some(FieldUpdates {
                sprint: sprint_id,
                ..Default::default()
            }),
            update: None,
        };
        self.update_issue(key, update).await
    }

    // ========================================================================
    // Comment Operations
    // ========================================================================

    /// Get comments for an issue.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `start_at` - Starting index for pagination (0-based)
    /// * `max_results` - Maximum number of comments to return (default 50, max 100)
    #[instrument(skip(self), fields(issue_key = %key))]
    pub async fn get_comments(
        &self,
        key: &str,
        start_at: u32,
        max_results: u32,
    ) -> Result<CommentsResponse> {
        debug!("Fetching comments for issue {}", key);
        let url = format!(
            "{}/rest/api/3/issue/{}/comment?startAt={}&maxResults={}&orderBy=-created",
            self.base_url,
            key,
            start_at,
            max_results.min(100)
        );
        let response: CommentsResponse = self.get(&url).await?;
        debug!(
            "Found {} comments (total: {})",
            response.comments.len(),
            response.total
        );
        Ok(response)
    }

    /// Add a comment to an issue.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `body` - The comment text (will be converted to ADF)
    ///
    /// # Returns
    ///
    /// The created comment.
    #[instrument(skip(self, body), fields(issue_key = %key))]
    pub async fn add_comment(&self, key: &str, body: &str) -> Result<Comment> {
        info!("Adding comment to issue {}", key);
        let url = format!("{}/rest/api/3/issue/{}/comment", self.base_url, key);
        let request = AddCommentRequest::from_text(body);
        let json_value = serde_json::to_value(request).map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to serialize comment: {}", e))
        })?;
        let comment: Comment = self.post(&url, &json_value).await?;
        info!("Successfully added comment {} to issue {}", comment.id, key);
        Ok(comment)
    }

    // ========================================================================
    // Changelog Operations
    // ========================================================================

    /// Get the changelog (history) for an issue.
    ///
    /// Returns a paginated list of changes made to the issue, including field
    /// changes, status transitions, and user actions.
    ///
    /// # Arguments
    ///
    /// * `key` - The issue key (e.g., "PROJ-123")
    /// * `start_at` - Starting index for pagination (0-based)
    /// * `max_results` - Maximum number of history entries to return (default 50, max 100)
    #[instrument(skip(self), fields(issue_key = %key))]
    pub async fn get_changelog(
        &self,
        key: &str,
        start_at: u32,
        max_results: u32,
    ) -> Result<Changelog> {
        debug!("Fetching changelog for issue {}", key);
        let url = format!(
            "{}/rest/api/3/issue/{}/changelog?startAt={}&maxResults={}",
            self.base_url,
            key,
            start_at,
            max_results.min(100)
        );
        let response: Changelog = self.get(&url).await?;
        debug!(
            "Found {} history entries (total: {})",
            response.histories.len(),
            response.total
        );
        Ok(response)
    }

    // ========================================================================
    // Issue Link Operations
    // ========================================================================

    /// Get available issue link types.
    ///
    /// Returns all link types available in the JIRA instance (e.g., "Blocks",
    /// "Relates", "Duplicates").
    #[instrument(skip(self))]
    pub async fn get_issue_link_types(&self) -> Result<Vec<IssueLinkType>> {
        debug!("Fetching issue link types");
        let url = format!("{}/rest/api/3/issueLinkType", self.base_url);
        let response: IssueLinkTypesResponse = self.get(&url).await?;
        debug!("Found {} issue link types", response.issue_link_types.len());
        Ok(response.issue_link_types)
    }

    /// Create a link between two issues.
    ///
    /// # Arguments
    ///
    /// * `link_type_name` - The name of the link type (e.g., "Blocks", "Relates")
    /// * `outward_issue_key` - The issue key that is the source (e.g., "PROJ-123 blocks PROJ-456")
    /// * `inward_issue_key` - The issue key that is the target (e.g., "PROJ-456 is blocked by PROJ-123")
    ///
    /// # Note
    ///
    /// For a "Blocks" link type where A blocks B:
    /// - `outward_issue_key` should be A (the blocker)
    /// - `inward_issue_key` should be B (the blocked issue)
    #[instrument(skip(self), fields(link_type = %link_type_name, outward = %outward_issue_key, inward = %inward_issue_key))]
    pub async fn create_issue_link(
        &self,
        link_type_name: &str,
        outward_issue_key: &str,
        inward_issue_key: &str,
    ) -> Result<()> {
        info!(
            "Creating issue link: {} {} {}",
            outward_issue_key, link_type_name, inward_issue_key
        );
        let url = format!("{}/rest/api/3/issueLink", self.base_url);
        let request = CreateIssueLinkRequest {
            link_type: IssueLinkTypeRef {
                name: link_type_name.to_string(),
            },
            outward_issue: IssueKeyRef {
                key: outward_issue_key.to_string(),
            },
            inward_issue: IssueKeyRef {
                key: inward_issue_key.to_string(),
            },
        };
        self.post_no_content(&url, &request).await?;
        info!("Successfully created issue link");
        Ok(())
    }

    /// Delete an issue link.
    ///
    /// # Arguments
    ///
    /// * `link_id` - The ID of the link to delete
    #[instrument(skip(self), fields(link_id = %link_id))]
    pub async fn delete_issue_link(&self, link_id: &str) -> Result<()> {
        info!("Deleting issue link {}", link_id);
        let url = format!("{}/rest/api/3/issueLink/{}", self.base_url, link_id);
        self.delete(&url).await?;
        info!("Successfully deleted issue link {}", link_id);
        Ok(())
    }

    /// Search for issues using the issue picker endpoint.
    ///
    /// This endpoint is optimized for autocomplete/typeahead scenarios and returns
    /// suggestions based on the query.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query (can be issue key or text)
    /// * `current_issue_key` - Optional key of the current issue (to exclude from results)
    #[instrument(skip(self), fields(query = %query))]
    pub async fn search_issues_for_picker(
        &self,
        query: &str,
        current_issue_key: Option<&str>,
    ) -> Result<Vec<IssueSuggestion>> {
        debug!("Searching issues for picker with query: {}", query);
        let mut url = format!(
            "{}/rest/api/3/issue/picker?query={}",
            self.base_url,
            urlencoding::encode(query)
        );
        if let Some(key) = current_issue_key {
            url.push_str(&format!("&currentIssueKey={}", key));
        }

        // Get raw response for debugging
        let raw_response: serde_json::Value = self.get(&url).await?;
        debug!("Issue picker raw response: {:?}", raw_response);

        // Parse the response
        let response: IssuePickerResponse = serde_json::from_value(raw_response).map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to parse picker response: {}", e))
        })?;

        // Flatten all sections into a single list
        let suggestions: Vec<IssueSuggestion> = response
            .sections
            .into_iter()
            .flat_map(|s| s.issues)
            .collect();

        debug!("Found {} issue suggestions", suggestions.len());
        Ok(suggestions)
    }
}

/// Normalize the base URL by removing trailing slashes and ensuring HTTPS.
fn normalize_base_url(url: &str) -> String {
    let url = url.trim_end_matches('/');

    // Warn if not HTTPS (but don't enforce for localhost/testing)
    if !url.starts_with("https://") && !url.contains("localhost") {
        warn!(
            "URL does not use HTTPS: {}. This is insecure for production use.",
            url
        );
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
