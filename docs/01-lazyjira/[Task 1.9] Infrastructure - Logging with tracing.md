# Task 1.9: Logging with Tracing

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 1.9
**Area:** Infrastructure
**Estimated Effort:** S (2-4 hours)

## Description

Set up structured logging using the tracing crate with appropriate log levels, file output, and span-based context for debugging API calls and application flow.

## Acceptance Criteria

- [ ] Tracing subscriber configured with appropriate defaults
- [ ] Log output to file (not terminal, to avoid TUI corruption)
- [ ] Log levels configurable via environment variable or config
- [ ] API request/response logging (without sensitive data)
- [ ] Span-based context for async operations
- [ ] Log rotation or size limits
- [ ] Debug mode with verbose output
- [ ] No sensitive data (tokens, passwords) in logs

## Implementation Details

### Approach

1. Configure tracing-subscriber with file appender
2. Set up log levels (RUST_LOG environment variable)
3. Add spans for major operations
4. Instrument API client with request logging
5. Ensure token/password scrubbing
6. Add startup/shutdown logging

### Files to Modify/Create

- `src/logging.rs`: Tracing configuration
- `src/main.rs`: Initialize logging
- `src/api/client.rs`: Add tracing instrumentation

### Technical Specifications

**Logging Setup:**
```rust
use tracing_subscriber::{
    fmt,
    prelude::*,
    filter::EnvFilter,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

pub fn init_logging() -> Result<()> {
    let log_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow!("Could not determine log directory"))?
        .join("lazyjira")
        .join("logs");

    std::fs::create_dir_all(&log_dir)?;

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &log_dir,
        "lazyjira.log",
    );

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("lazyjira=info,warn"));

    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
        )
        .with(filter);

    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("LazyJira starting up");
    Ok(())
}
```

**Log Directory:**
```
~/.local/share/lazyjira/logs/
├── lazyjira.log           # Current log
├── lazyjira.log.2024-01-15 # Rotated log
└── lazyjira.log.2024-01-14 # Older log
```

**API Request Logging:**
```rust
impl JiraClient {
    #[tracing::instrument(skip(self), fields(url = %url))]
    async fn request<T: DeserializeOwned>(&self, method: Method, url: &str) -> Result<T> {
        tracing::debug!("Sending {} request", method);

        let response = self.client
            .request(method.clone(), url)
            .header("Authorization", &self.auth_header) // Don't log this!
            .send()
            .await?;

        let status = response.status();
        tracing::debug!(status = %status, "Received response");

        if !status.is_success() {
            tracing::warn!(status = %status, "Request failed");
        }

        // ... handle response
    }

    #[tracing::instrument(skip(self), fields(jql = %jql, start = start, max = max))]
    pub async fn search_issues(&self, jql: &str, start: u32, max: u32) -> Result<SearchResult> {
        tracing::info!("Searching issues");
        // ...
    }
}
```

**Sensitive Data Protection:**
```rust
// NEVER log tokens or passwords
// Use skip in instrument macro
#[tracing::instrument(skip(token))]
pub fn store_token(profile_name: &str, token: &str) -> Result<()> {
    tracing::info!(profile = profile_name, "Storing token in keyring");
    // ...
}

// Redact sensitive fields in structs
impl std::fmt::Debug for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Profile")
            .field("name", &self.name)
            .field("url", &self.url)
            .field("email", &self.email)
            // Token is NOT included
            .finish()
    }
}
```

**Span Context for Async:**
```rust
pub async fn fetch_and_display_issues(app: &mut App) -> Result<()> {
    let span = tracing::info_span!("fetch_issues", profile = %app.current_profile);
    async {
        tracing::info!("Starting issue fetch");

        let issues = app.client.search_issues("", 0, 50).await?;
        tracing::info!(count = issues.issues.len(), "Fetched issues");

        app.issue_list.set_issues(issues.issues);
        Ok(())
    }
    .instrument(span)
    .await
}
```

**Log Levels:**
- `error`: Unrecoverable errors, crashes
- `warn`: Recoverable errors, degraded functionality
- `info`: Major operations (startup, API calls, user actions)
- `debug`: Detailed operation flow
- `trace`: Very verbose, frame-by-frame details

## Testing Requirements

- [ ] Log file created in correct location
- [ ] Log levels filter correctly (RUST_LOG)
- [ ] API requests logged without tokens
- [ ] Spans provide useful context
- [ ] Log rotation works
- [ ] No terminal output from logging

## Dependencies

- **Prerequisite Tasks:** Task 1.1
- **Blocks Tasks:** None (all tasks should use logging)
- **External:** tracing, tracing-subscriber, tracing-appender

## Definition of Done

- [ ] All acceptance criteria met
- [ ] No sensitive data in any log output
- [ ] Logs are useful for debugging issues
- [ ] Log file location documented
- [ ] Log level configuration documented
