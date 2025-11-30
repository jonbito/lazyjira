//! JIRA API request and response types.
//!
//! These types model the JIRA REST API v3 responses for issues and search results.

use serde::{Deserialize, Serialize};

/// The current authenticated user.
///
/// Returned by `GET /rest/api/3/myself`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUser {
    /// The user's account ID.
    pub account_id: String,
    /// The user's display name.
    pub display_name: String,
    /// The user's email address (may be empty if hidden).
    #[serde(default)]
    pub email_address: String,
    /// Whether the user is active.
    #[serde(default = "default_true")]
    pub active: bool,
    /// The user's timezone.
    #[serde(default)]
    pub time_zone: Option<String>,
    /// URLs for the user's avatar images.
    #[serde(default)]
    pub avatar_urls: Option<AvatarUrls>,
}

fn default_true() -> bool {
    true
}

/// Avatar URLs for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarUrls {
    /// 48x48 pixel avatar.
    #[serde(rename = "48x48")]
    pub size_48: Option<String>,
    /// 24x24 pixel avatar.
    #[serde(rename = "24x24")]
    pub size_24: Option<String>,
    /// 16x16 pixel avatar.
    #[serde(rename = "16x16")]
    pub size_16: Option<String>,
    /// 32x32 pixel avatar.
    #[serde(rename = "32x32")]
    pub size_32: Option<String>,
}

/// Search result from JQL query.
///
/// Returned by `GET /rest/api/3/search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// The index of the first result.
    pub start_at: u32,
    /// Maximum results requested.
    pub max_results: u32,
    /// Total number of matching issues.
    pub total: u32,
    /// The list of issues.
    #[serde(default)]
    pub issues: Vec<Issue>,
}

impl SearchResult {
    /// Check if there are more pages of results.
    pub fn has_more(&self) -> bool {
        self.start_at + (self.issues.len() as u32) < self.total
    }

    /// Get the starting index for the next page.
    pub fn next_start(&self) -> u32 {
        self.start_at + self.issues.len() as u32
    }
}

/// A JIRA issue.
///
/// Returned by `GET /rest/api/3/issue/{issueKey}` or as part of search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// The issue ID.
    pub id: String,
    /// The issue key (e.g., "PROJ-123").
    pub key: String,
    /// URL to view the issue in JIRA.
    #[serde(rename = "self")]
    pub self_url: String,
    /// The issue fields.
    pub fields: IssueFields,
}

impl Issue {
    /// Get the issue summary.
    pub fn summary(&self) -> &str {
        &self.fields.summary
    }

    /// Get the issue status name.
    pub fn status(&self) -> &str {
        &self.fields.status.name
    }

    /// Get the issue type name.
    pub fn issue_type(&self) -> &str {
        &self.fields.issuetype.name
    }

    /// Get the issue priority name, if set.
    pub fn priority(&self) -> Option<&str> {
        self.fields.priority.as_ref().map(|p| p.name.as_str())
    }

    /// Get the assignee display name, if assigned.
    pub fn assignee(&self) -> Option<&str> {
        self.fields.assignee.as_ref().map(|a| a.display_name.as_str())
    }

    /// Get the reporter display name, if set.
    pub fn reporter(&self) -> Option<&str> {
        self.fields.reporter.as_ref().map(|r| r.display_name.as_str())
    }
}

/// Issue fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueFields {
    /// The issue summary/title.
    pub summary: String,
    /// The issue description (may be in Atlassian Document Format).
    #[serde(default)]
    pub description: Option<serde_json::Value>,
    /// The issue status.
    pub status: Status,
    /// The issue type (Bug, Story, Task, etc.).
    pub issuetype: IssueType,
    /// The issue priority.
    #[serde(default)]
    pub priority: Option<Priority>,
    /// The issue assignee.
    #[serde(default)]
    pub assignee: Option<User>,
    /// The issue reporter.
    #[serde(default)]
    pub reporter: Option<User>,
    /// The project this issue belongs to.
    #[serde(default)]
    pub project: Option<Project>,
    /// Labels attached to the issue.
    #[serde(default)]
    pub labels: Vec<String>,
    /// Components the issue is associated with.
    #[serde(default)]
    pub components: Vec<Component>,
    /// When the issue was created.
    #[serde(default)]
    pub created: Option<String>,
    /// When the issue was last updated.
    #[serde(default)]
    pub updated: Option<String>,
    /// When the issue is due.
    #[serde(default)]
    pub duedate: Option<String>,
    /// Story points or other estimate.
    #[serde(default, rename = "customfield_10016")]
    pub story_points: Option<f64>,
}

/// Issue status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// The status ID.
    pub id: String,
    /// The status name (e.g., "To Do", "In Progress", "Done").
    pub name: String,
    /// The status category.
    #[serde(default)]
    pub status_category: Option<StatusCategory>,
}

/// Status category (groups statuses into to-do, in-progress, done).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCategory {
    /// The category ID.
    pub id: u32,
    /// The category key.
    pub key: String,
    /// The category name.
    pub name: String,
    /// The category color.
    #[serde(default)]
    pub color_name: Option<String>,
}

/// Issue type (Bug, Story, Task, Epic, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueType {
    /// The issue type ID.
    pub id: String,
    /// The issue type name.
    pub name: String,
    /// Whether this is a subtask type.
    #[serde(default)]
    pub subtask: bool,
    /// The issue type description.
    #[serde(default)]
    pub description: Option<String>,
    /// URL to the issue type icon.
    #[serde(default)]
    pub icon_url: Option<String>,
}

/// Issue priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Priority {
    /// The priority ID.
    pub id: String,
    /// The priority name (e.g., "Highest", "High", "Medium", "Low", "Lowest").
    pub name: String,
    /// URL to the priority icon.
    #[serde(default)]
    pub icon_url: Option<String>,
}

/// A JIRA user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// The user's account ID.
    pub account_id: String,
    /// The user's display name.
    pub display_name: String,
    /// The user's email address (may be empty).
    #[serde(default)]
    pub email_address: Option<String>,
    /// Whether the user is active.
    #[serde(default = "default_true")]
    pub active: bool,
    /// URLs for the user's avatar images.
    #[serde(default)]
    pub avatar_urls: Option<AvatarUrls>,
}

/// A JIRA project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// The project ID.
    pub id: String,
    /// The project key (e.g., "PROJ").
    pub key: String,
    /// The project name.
    pub name: String,
    /// URLs for the project's avatar images.
    #[serde(default, rename = "avatarUrls")]
    pub avatar_urls: Option<AvatarUrls>,
}

/// A project component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// The component ID.
    pub id: String,
    /// The component name.
    pub name: String,
    /// The component description.
    #[serde(default)]
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_has_more() {
        // First page: start=0, got 50 issues, total is 100 -> has more
        let result = SearchResult {
            start_at: 0,
            max_results: 50,
            total: 100,
            issues: (0..50).map(|_| create_test_issue()).collect(),
        };
        assert!(result.has_more());

        // Last page: start=50, got 50 issues, total is 100 -> no more
        let result = SearchResult {
            start_at: 50,
            max_results: 50,
            total: 100,
            issues: (0..50).map(|_| create_test_issue()).collect(),
        };
        assert!(!result.has_more());

        // Partial last page: start=90, got 10 issues, total is 100 -> no more
        let result = SearchResult {
            start_at: 90,
            max_results: 50,
            total: 100,
            issues: (0..10).map(|_| create_test_issue()).collect(),
        };
        assert!(!result.has_more());
    }

    fn create_test_issue() -> Issue {
        Issue {
            id: "1".to_string(),
            key: "TEST-1".to_string(),
            self_url: "https://example.com".to_string(),
            fields: IssueFields {
                summary: "Test".to_string(),
                description: None,
                status: Status {
                    id: "1".to_string(),
                    name: "Open".to_string(),
                    status_category: None,
                },
                issuetype: IssueType {
                    id: "1".to_string(),
                    name: "Bug".to_string(),
                    subtask: false,
                    description: None,
                    icon_url: None,
                },
                priority: None,
                assignee: None,
                reporter: None,
                project: None,
                labels: vec![],
                components: vec![],
                created: None,
                updated: None,
                duedate: None,
                story_points: None,
            },
        }
    }

    #[test]
    fn test_search_result_next_start() {
        let result = SearchResult {
            start_at: 0,
            max_results: 50,
            total: 100,
            issues: vec![],
        };
        assert_eq!(result.next_start(), 0);
    }

    #[test]
    fn test_parse_minimal_issue() {
        let json = r#"{
            "id": "10001",
            "key": "PROJ-123",
            "self": "https://company.atlassian.net/rest/api/3/issue/10001",
            "fields": {
                "summary": "Test issue",
                "status": {
                    "id": "1",
                    "name": "To Do"
                },
                "issuetype": {
                    "id": "10001",
                    "name": "Bug"
                }
            }
        }"#;

        let issue: Issue = serde_json::from_str(json).unwrap();
        assert_eq!(issue.key, "PROJ-123");
        assert_eq!(issue.summary(), "Test issue");
        assert_eq!(issue.status(), "To Do");
        assert_eq!(issue.issue_type(), "Bug");
        assert!(issue.priority().is_none());
        assert!(issue.assignee().is_none());
    }

    #[test]
    fn test_parse_full_issue() {
        let json = r#"{
            "id": "10001",
            "key": "PROJ-123",
            "self": "https://company.atlassian.net/rest/api/3/issue/10001",
            "fields": {
                "summary": "Test issue with full fields",
                "status": {
                    "id": "1",
                    "name": "In Progress",
                    "statusCategory": {
                        "id": 4,
                        "key": "indeterminate",
                        "name": "In Progress",
                        "colorName": "yellow"
                    }
                },
                "issuetype": {
                    "id": "10001",
                    "name": "Story",
                    "subtask": false
                },
                "priority": {
                    "id": "2",
                    "name": "High"
                },
                "assignee": {
                    "accountId": "abc123",
                    "displayName": "John Doe",
                    "active": true
                },
                "reporter": {
                    "accountId": "def456",
                    "displayName": "Jane Smith",
                    "active": true
                },
                "project": {
                    "id": "10000",
                    "key": "PROJ",
                    "name": "My Project"
                },
                "labels": ["frontend", "urgent"],
                "components": [
                    {"id": "10001", "name": "UI"}
                ],
                "created": "2024-01-15T10:00:00.000+0000",
                "updated": "2024-01-16T14:30:00.000+0000"
            }
        }"#;

        let issue: Issue = serde_json::from_str(json).unwrap();
        assert_eq!(issue.key, "PROJ-123");
        assert_eq!(issue.summary(), "Test issue with full fields");
        assert_eq!(issue.status(), "In Progress");
        assert_eq!(issue.issue_type(), "Story");
        assert_eq!(issue.priority(), Some("High"));
        assert_eq!(issue.assignee(), Some("John Doe"));
        assert_eq!(issue.reporter(), Some("Jane Smith"));
        assert_eq!(issue.fields.labels, vec!["frontend", "urgent"]);
        assert_eq!(issue.fields.components.len(), 1);
        assert_eq!(issue.fields.project.as_ref().unwrap().key, "PROJ");
    }

    #[test]
    fn test_parse_current_user() {
        let json = r#"{
            "accountId": "abc123",
            "displayName": "Test User",
            "emailAddress": "test@example.com",
            "active": true,
            "timeZone": "America/New_York"
        }"#;

        let user: CurrentUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.account_id, "abc123");
        assert_eq!(user.display_name, "Test User");
        assert_eq!(user.email_address, "test@example.com");
        assert!(user.active);
    }

    #[test]
    fn test_parse_search_result() {
        let json = r#"{
            "startAt": 0,
            "maxResults": 50,
            "total": 2,
            "issues": [
                {
                    "id": "10001",
                    "key": "PROJ-1",
                    "self": "https://company.atlassian.net/rest/api/3/issue/10001",
                    "fields": {
                        "summary": "First issue",
                        "status": {"id": "1", "name": "Open"},
                        "issuetype": {"id": "1", "name": "Bug"}
                    }
                },
                {
                    "id": "10002",
                    "key": "PROJ-2",
                    "self": "https://company.atlassian.net/rest/api/3/issue/10002",
                    "fields": {
                        "summary": "Second issue",
                        "status": {"id": "2", "name": "Done"},
                        "issuetype": {"id": "2", "name": "Task"}
                    }
                }
            ]
        }"#;

        let result: SearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.start_at, 0);
        assert_eq!(result.max_results, 50);
        assert_eq!(result.total, 2);
        assert_eq!(result.issues.len(), 2);
        assert_eq!(result.issues[0].key, "PROJ-1");
        assert_eq!(result.issues[1].key, "PROJ-2");
        assert!(!result.has_more());
    }
}
