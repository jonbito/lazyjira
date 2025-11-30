# Task 1.4: JIRA Client with Basic Authentication

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 1.4
**Area:** Backend/API
**Estimated Effort:** M (6-10 hours)

## Description

Implement the JIRA REST API client with basic authentication support. The client should handle API requests, authentication, error handling, and response parsing for the JIRA Cloud REST API v3.

## Acceptance Criteria

- [ ] JIRA API client struct with configurable base URL
- [ ] Basic authentication using email + API token
- [ ] Token stored securely via OS keyring (keyring crate)
- [ ] Async HTTP requests with reqwest
- [ ] Proper error handling for API errors (401, 403, 404, 429, 5xx)
- [ ] Rate limiting awareness and retry logic
- [ ] Request/response logging with tracing
- [ ] HTTPS enforced for all requests
- [ ] Connection validation on client creation

## Implementation Details

### Approach

1. Create `JiraClient` struct with reqwest client and auth
2. Implement keyring integration for token storage/retrieval
3. Build authentication header (Base64 of email:token)
4. Add generic request method with error handling
5. Implement specific API endpoints (search, get issue)
6. Add retry logic for transient failures
7. Integrate tracing for debugging

### Files to Modify/Create

- `src/api/mod.rs`: Module exports, JiraClient struct
- `src/api/client.rs`: HTTP client implementation
- `src/api/auth.rs`: Authentication handling, keyring integration
- `src/api/types.rs`: API request/response types
- `src/api/error.rs`: API error types

### Technical Specifications

**Authentication (from Atlassian docs):**
```rust
// Header: Authorization: Basic base64(email:api_token)
fn build_auth_header(email: &str, token: &str) -> String {
    let credentials = format!("{}:{}", email, token);
    let encoded = base64::encode(credentials);
    format!("Basic {}", encoded)
}
```

**JiraClient Struct:**
```rust
pub struct JiraClient {
    client: reqwest::Client,
    base_url: String,
    auth_header: String,
}

impl JiraClient {
    pub async fn new(profile: &Profile) -> Result<Self> {
        let token = keyring::Entry::new("lazyjira", &profile.name)?
            .get_password()?;

        let auth_header = build_auth_header(&profile.email, &token);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let jira = Self { client, base_url: profile.url.clone(), auth_header };
        jira.validate_connection().await?;
        Ok(jira)
    }

    pub async fn search_issues(&self, jql: &str, start: u32, max: u32) -> Result<SearchResult> { ... }
    pub async fn get_issue(&self, key: &str) -> Result<Issue> { ... }
}
```

**API Endpoints (REST API v3):**
- Search: `GET /rest/api/3/search?jql={jql}&startAt={start}&maxResults={max}`
- Get Issue: `GET /rest/api/3/issue/{issueKey}`
- Myself: `GET /rest/api/3/myself` (for connection validation)

**Error Handling:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Authentication failed: check your email and API token")]
    Unauthorized,

    #[error("Permission denied: you don't have access to this resource")]
    Forbidden,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Rate limited: please wait before retrying")]
    RateLimited,

    #[error("JIRA server error: {0}")]
    ServerError(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}
```

**Keyring Integration:**
```rust
pub fn store_token(profile_name: &str, token: &str) -> Result<()> {
    let entry = keyring::Entry::new("lazyjira", profile_name)?;
    entry.set_password(token)?;
    Ok(())
}

pub fn get_token(profile_name: &str) -> Result<String> {
    let entry = keyring::Entry::new("lazyjira", profile_name)?;
    Ok(entry.get_password()?)
}

pub fn delete_token(profile_name: &str) -> Result<()> {
    let entry = keyring::Entry::new("lazyjira", profile_name)?;
    entry.delete_password()?;
    Ok(())
}
```

## Testing Requirements

- [ ] Client creation with valid credentials succeeds
- [ ] Client creation with invalid credentials returns Unauthorized
- [ ] Invalid URL produces clear error
- [ ] 404 response handled correctly
- [ ] Rate limiting detected and logged
- [ ] Token stored and retrieved from keyring
- [ ] Token never logged or displayed

## Dependencies

- **Prerequisite Tasks:** Task 1.1, Task 1.3 (Profile struct)
- **Blocks Tasks:** Task 1.5, Task 1.6
- **External:** reqwest, keyring, base64, tokio

## Definition of Done

- [ ] All acceptance criteria met
- [ ] No plaintext tokens in logs or errors
- [ ] Async operations work correctly
- [ ] Error messages are user-friendly
- [ ] Integration test with mock server (optional)
- [ ] Works on Linux, macOS, Windows keyrings
