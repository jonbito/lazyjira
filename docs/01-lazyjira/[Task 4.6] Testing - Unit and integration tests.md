# Task 4.6: Unit and Integration Tests

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 4.6
**Area:** Testing
**Estimated Effort:** L (10-15 hours)

## Description

Implement comprehensive unit and integration tests for all major components including configuration, API client, state management, and UI components.

## Acceptance Criteria

- [ ] Unit tests for configuration loading/saving
- [ ] Unit tests for API types parsing
- [ ] Integration tests for JIRA API client (with mock server)
- [ ] Tests for state management logic
- [ ] Tests for filter/JQL generation
- [ ] Tests for cache operations
- [ ] Test coverage > 70%
- [ ] CI-friendly test execution

## Implementation Details

### Approach

1. Set up test infrastructure
2. Create mock JIRA API responses
3. Write unit tests for core modules
4. Write integration tests with mock server
5. Add test utilities for common patterns

### Files to Modify/Create

- `tests/` directory structure
- `src/*/tests.rs` inline test modules
- `tests/fixtures/` mock API responses
- `tests/common/mod.rs` test utilities

### Technical Specifications

**Test Directory Structure:**
```
tests/
├── common/
│   ├── mod.rs           # Test utilities
│   ├── fixtures.rs      # Mock data
│   └── mock_server.rs   # Mock JIRA server
├── integration/
│   ├── api_client.rs    # API client tests
│   ├── config.rs        # Configuration tests
│   └── cache.rs         # Cache tests
└── fixtures/
    ├── issue.json       # Sample issue response
    ├── search.json      # Sample search response
    └── transitions.json # Sample transitions
```

**Configuration Tests:**
```rust
// src/config/mod.rs or tests/integration/config.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.profiles.is_empty());
        assert_eq!(config.settings.theme, "dark");
        assert_eq!(config.settings.cache_ttl_minutes, 30);
    }

    #[test]
    fn test_config_load_valid() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");

        std::fs::write(&config_path, r#"
            [settings]
            default_profile = "work"
            theme = "light"

            [[profiles]]
            name = "work"
            url = "https://jira.example.com"
            email = "user@example.com"
        "#).unwrap();

        let config = Config::load_from_path(&config_path).unwrap();
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].name, "work");
        assert_eq!(config.settings.theme, "light");
    }

    #[test]
    fn test_config_validation_duplicate_profiles() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile { name: "work".into(), url: "https://a.com".into(), email: "a@a.com".into() },
                Profile { name: "work".into(), url: "https://b.com".into(), email: "b@b.com".into() },
            ],
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("duplicate"));
    }

    #[test]
    fn test_config_save_and_reload() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");

        let config = Config {
            settings: Settings {
                default_profile: Some("test".into()),
                theme: "dark".into(),
                ..Default::default()
            },
            profiles: vec![
                Profile {
                    name: "test".into(),
                    url: "https://test.atlassian.net".into(),
                    email: "test@test.com".into(),
                },
            ],
        };

        config.save_to_path(&config_path).unwrap();
        let reloaded = Config::load_from_path(&config_path).unwrap();

        assert_eq!(reloaded.profiles.len(), 1);
        assert_eq!(reloaded.settings.default_profile, Some("test".into()));
    }
}
```

**API Types Tests:**
```rust
// src/api/types.rs
#[cfg(test)]
mod tests {
    use super::*;

    const ISSUE_JSON: &str = r#"{
        "key": "PROJ-123",
        "fields": {
            "summary": "Test issue",
            "status": {
                "name": "In Progress",
                "id": "3",
                "statusCategory": {
                    "key": "indeterminate",
                    "name": "In Progress"
                }
            },
            "assignee": {
                "displayName": "John Doe",
                "emailAddress": "john@example.com",
                "accountId": "123"
            },
            "priority": {
                "name": "High",
                "id": "2"
            },
            "issuetype": {
                "name": "Bug",
                "id": "1"
            },
            "labels": ["backend", "urgent"],
            "created": "2024-01-15T10:30:00.000+0000",
            "updated": "2024-01-16T14:20:00.000+0000"
        }
    }"#;

    #[test]
    fn test_parse_issue() {
        let issue: Issue = serde_json::from_str(ISSUE_JSON).unwrap();

        assert_eq!(issue.key, "PROJ-123");
        assert_eq!(issue.fields.summary, "Test issue");
        assert_eq!(issue.fields.status.name, "In Progress");
        assert!(issue.fields.assignee.is_some());
        assert_eq!(issue.fields.labels, vec!["backend", "urgent"]);
    }

    #[test]
    fn test_parse_issue_null_assignee() {
        let json = r#"{
            "key": "PROJ-456",
            "fields": {
                "summary": "Unassigned issue",
                "status": { "name": "Open", "id": "1" },
                "assignee": null,
                "issuetype": { "name": "Task", "id": "2" },
                "created": "2024-01-15T10:30:00.000+0000",
                "updated": "2024-01-15T10:30:00.000+0000"
            }
        }"#;

        let issue: Issue = serde_json::from_str(json).unwrap();
        assert!(issue.fields.assignee.is_none());
        assert_eq!(issue.assignee_name(), "Unassigned");
    }

    #[test]
    fn test_parse_search_results() {
        let json = r#"{
            "issues": [],
            "startAt": 0,
            "maxResults": 50,
            "total": 0
        }"#;

        let results: SearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(results.total, 0);
        assert!(results.issues.is_empty());
    }
}
```

**Mock JIRA Server:**
```rust
// tests/common/mock_server.rs
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};

pub async fn setup_mock_server() -> MockServer {
    MockServer::start().await
}

pub async fn mock_search_issues(server: &MockServer, issues: Vec<serde_json::Value>) {
    let response = serde_json::json!({
        "issues": issues,
        "startAt": 0,
        "maxResults": 50,
        "total": issues.len()
    });

    Mock::given(method("GET"))
        .and(path("/rest/api/3/search"))
        .and(header("Authorization", "Basic dGVzdEB0ZXN0LmNvbTp0ZXN0dG9rZW4="))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(server)
        .await;
}

pub async fn mock_get_issue(server: &MockServer, key: &str, issue: serde_json::Value) {
    Mock::given(method("GET"))
        .and(path(format!("/rest/api/3/issue/{}", key)))
        .respond_with(ResponseTemplate::new(200).set_body_json(issue))
        .mount(server)
        .await;
}

pub async fn mock_unauthorized(server: &MockServer) {
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "errorMessages": ["Unauthorized"],
            "errors": {}
        })))
        .mount(server)
        .await;
}
```

**API Client Integration Tests:**
```rust
// tests/integration/api_client.rs
use lazyjira::api::{JiraClient, ApiError};
use lazyjira::config::Profile;

mod common;
use common::mock_server::*;

#[tokio::test]
async fn test_search_issues_success() {
    let server = setup_mock_server().await;

    mock_search_issues(&server, vec![
        serde_json::json!({
            "key": "TEST-1",
            "fields": {
                "summary": "Test Issue",
                "status": { "name": "Open", "id": "1" },
                "issuetype": { "name": "Bug", "id": "1" },
                "created": "2024-01-15T10:30:00.000+0000",
                "updated": "2024-01-15T10:30:00.000+0000"
            }
        })
    ]).await;

    let profile = Profile {
        name: "test".into(),
        url: server.uri(),
        email: "test@test.com".into(),
    };

    // Note: Need to mock keyring or use test credentials
    let client = JiraClient::new_with_token(&profile, "testtoken").await.unwrap();
    let results = client.search_issues("project = TEST", 0, 50).await.unwrap();

    assert_eq!(results.issues.len(), 1);
    assert_eq!(results.issues[0].key, "TEST-1");
}

#[tokio::test]
async fn test_unauthorized_error() {
    let server = setup_mock_server().await;
    mock_unauthorized(&server).await;

    let profile = Profile {
        name: "test".into(),
        url: server.uri(),
        email: "test@test.com".into(),
    };

    let result = JiraClient::new_with_token(&profile, "badtoken").await;
    assert!(matches!(result, Err(ApiError::Unauthorized)));
}
```

**Filter State Tests:**
```rust
// src/ui/views/filter.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_filter_generates_empty_jql() {
        let filter = FilterState::default();
        assert!(filter.to_jql("currentUser").is_empty());
    }

    #[test]
    fn test_status_filter_jql() {
        let filter = FilterState {
            statuses: vec!["Open".into(), "In Progress".into()],
            ..Default::default()
        };

        let jql = filter.to_jql("currentUser");
        assert!(jql.contains("status IN"));
        assert!(jql.contains("\"Open\""));
        assert!(jql.contains("\"In Progress\""));
    }

    #[test]
    fn test_assignee_me_filter() {
        let filter = FilterState {
            assignee_is_me: true,
            ..Default::default()
        };

        let jql = filter.to_jql("user123");
        assert!(jql.contains("assignee = currentUser()"));
    }

    #[test]
    fn test_combined_filters() {
        let filter = FilterState {
            statuses: vec!["Open".into()],
            project: Some("PROJ".into()),
            labels: vec!["urgent".into()],
            ..Default::default()
        };

        let jql = filter.to_jql("user");
        assert!(jql.contains(" AND "));
        assert!(jql.contains("status IN"));
        assert!(jql.contains("project = \"PROJ\""));
        assert!(jql.contains("labels IN"));
    }
}
```

**Cache Tests:**
```rust
// tests/integration/cache.rs
use lazyjira::cache::CacheManager;
use tempfile::TempDir;

#[test]
fn test_cache_write_read() {
    let dir = TempDir::new().unwrap();
    let cache = CacheManager::new_with_dir(dir.path(), "test", 30).unwrap();

    let issue = create_test_issue("TEST-1", "Test Issue");
    cache.set_issue(&issue).unwrap();

    let cached = cache.get_issue("TEST-1");
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().key, "TEST-1");
}

#[test]
fn test_cache_expiration() {
    let dir = TempDir::new().unwrap();
    let cache = CacheManager::new_with_dir(dir.path(), "test", 0).unwrap(); // 0 minute TTL

    let issue = create_test_issue("TEST-1", "Test Issue");
    cache.set_issue(&issue).unwrap();

    // Wait a moment for expiration
    std::thread::sleep(std::time::Duration::from_millis(100));

    let cached = cache.get_issue("TEST-1");
    assert!(cached.is_none()); // Should be expired
}

#[test]
fn test_cache_invalidation() {
    let dir = TempDir::new().unwrap();
    let cache = CacheManager::new_with_dir(dir.path(), "test", 30).unwrap();

    let issue = create_test_issue("TEST-1", "Test Issue");
    cache.set_issue(&issue).unwrap();
    cache.invalidate_issue("TEST-1").unwrap();

    let cached = cache.get_issue("TEST-1");
    assert!(cached.is_none());
}
```

## Testing Requirements

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Test coverage measured
- [ ] Tests run in CI
- [ ] Mock server works correctly
- [ ] No flaky tests

## Dependencies

- **Prerequisite Tasks:** All implementation tasks
- **Blocks Tasks:** None
- **External:** wiremock, tempfile, tokio-test

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Test coverage > 70%
- [ ] CI pipeline configured
- [ ] Tests are reliable
- [ ] Test documentation complete
