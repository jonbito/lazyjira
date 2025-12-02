//! JIRA API request and response types.
//!
//! These types model the JIRA REST API v3 responses for issues and search results.

use serde::{Deserialize, Serialize};
use std::fmt;

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
/// Returned by `POST /rest/api/3/search/jql`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// The index of the first result (legacy, may not be present in new API).
    #[serde(default)]
    pub start_at: u32,
    /// Maximum results requested (legacy, may not be present in new API).
    #[serde(default)]
    pub max_results: u32,
    /// Total number of matching issues.
    #[serde(default)]
    pub total: u32,
    /// The list of issues.
    #[serde(default)]
    pub issues: Vec<Issue>,
    /// Token for fetching the next page of results (new API).
    #[serde(default)]
    pub next_page_token: Option<String>,
    /// Whether this is the last page of results (new API).
    #[serde(default)]
    pub is_last: bool,
}

impl SearchResult {
    /// Check if there are more pages of results.
    pub fn has_more(&self) -> bool {
        // New API uses nextPageToken/isLast, old API uses total
        if self.next_page_token.is_some() {
            !self.is_last
        } else {
            self.start_at + (self.issues.len() as u32) < self.total
        }
    }

    /// Get the starting index for the next page.
    pub fn next_start(&self) -> u32 {
        self.start_at + self.issues.len() as u32
    }

    /// Get the next page token if available.
    pub fn next_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
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
    #[serde(default)]
    pub key: String,
    /// URL to view the issue in JIRA.
    #[serde(rename = "self", default)]
    pub self_url: String,
    /// The issue fields.
    #[serde(default)]
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
        self.fields
            .assignee
            .as_ref()
            .map(|a| a.display_name.as_str())
    }

    /// Get the reporter display name, if set.
    pub fn reporter(&self) -> Option<&str> {
        self.fields
            .reporter
            .as_ref()
            .map(|r| r.display_name.as_str())
    }

    /// Get the assignee display name, or "Unassigned" if not set.
    pub fn assignee_name(&self) -> &str {
        self.fields
            .assignee
            .as_ref()
            .map(|u| u.display_name.as_str())
            .unwrap_or("Unassigned")
    }

    /// Get the priority name, or "None" if not set.
    pub fn priority_name(&self) -> &str {
        self.fields
            .priority
            .as_ref()
            .map(|p| p.name.as_str())
            .unwrap_or("None")
    }

    /// Get the description as plain text, or empty string if not set.
    pub fn description_text(&self) -> String {
        self.fields
            .description
            .as_ref()
            .map(|d| {
                // Try to parse as AtlassianDoc first
                if let Ok(doc) = serde_json::from_value::<AtlassianDoc>(d.clone()) {
                    doc.to_plain_text()
                } else if let Some(s) = d.as_str() {
                    // Fall back to plain string
                    s.to_string()
                } else {
                    String::new()
                }
            })
            .unwrap_or_default()
    }

    /// Get the project key, if available.
    pub fn project_key(&self) -> Option<&str> {
        self.fields.project.as_ref().map(|p| p.key.as_str())
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.fields.summary)
    }
}

/// Issue fields.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueFields {
    /// The issue summary/title.
    #[serde(default)]
    pub summary: String,
    /// The issue description (may be in Atlassian Document Format).
    #[serde(default)]
    pub description: Option<serde_json::Value>,
    /// The issue status.
    #[serde(default)]
    pub status: Status,
    /// The issue type (Bug, Story, Task, etc.).
    #[serde(default)]
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
    /// Issue links (blocks, is blocked by, relates to, etc.).
    #[serde(default, rename = "issuelinks")]
    pub issue_links: Vec<IssueLink>,
    /// Subtasks of this issue.
    #[serde(default)]
    pub subtasks: Vec<Subtask>,
    /// Parent issue (for subtasks).
    #[serde(default)]
    pub parent: Option<ParentIssue>,
}

// ============================================================================
// Linked Issues and Subtasks Types
// ============================================================================

/// A link between two issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLink {
    /// The link ID.
    pub id: String,
    /// The type of link.
    #[serde(rename = "type")]
    pub link_type: IssueLinkType,
    /// The inward issue (if this issue is the target).
    #[serde(default, rename = "inwardIssue")]
    pub inward_issue: Option<LinkedIssue>,
    /// The outward issue (if this issue is the source).
    #[serde(default, rename = "outwardIssue")]
    pub outward_issue: Option<LinkedIssue>,
}

/// The type of issue link.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IssueLinkType {
    /// The link type ID.
    pub id: String,
    /// The link type name (e.g., "Blocks").
    pub name: String,
    /// The inward description (e.g., "is blocked by").
    pub inward: String,
    /// The outward description (e.g., "blocks").
    pub outward: String,
}

/// A linked issue with minimal fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedIssue {
    /// The issue ID.
    pub id: String,
    /// The issue key (e.g., "PROJ-123").
    pub key: String,
    /// The linked issue fields.
    pub fields: LinkedIssueFields,
}

/// Minimal fields for a linked issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedIssueFields {
    /// The issue summary.
    pub summary: String,
    /// The issue status.
    pub status: Status,
    /// The issue priority.
    #[serde(default)]
    pub priority: Option<Priority>,
    /// The issue type.
    #[serde(default, rename = "issuetype")]
    pub issue_type: Option<IssueType>,
}

/// A subtask of an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    /// The subtask ID.
    pub id: String,
    /// The subtask key (e.g., "PROJ-124").
    pub key: String,
    /// The subtask fields.
    pub fields: SubtaskFields,
}

/// Fields for a subtask.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskFields {
    /// The subtask summary.
    pub summary: String,
    /// The subtask status.
    pub status: Status,
    /// The issue type.
    #[serde(default, rename = "issuetype")]
    pub issue_type: Option<IssueType>,
}

/// The parent issue of a subtask.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentIssue {
    /// The parent issue ID.
    pub id: String,
    /// The parent issue key (e.g., "PROJ-100").
    pub key: String,
    /// The parent issue fields.
    pub fields: ParentIssueFields,
}

/// Minimal fields for a parent issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentIssueFields {
    /// The parent issue summary.
    pub summary: String,
    /// The parent issue status.
    pub status: Status,
    /// The issue type.
    #[serde(default, rename = "issuetype")]
    pub issue_type: Option<IssueType>,
}

// ============================================================================
// Issue Link Management Types
// ============================================================================

/// Response from the issue link types endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct IssueLinkTypesResponse {
    /// The available link types.
    #[serde(rename = "issueLinkTypes")]
    pub issue_link_types: Vec<IssueLinkType>,
}

/// Request to create an issue link.
#[derive(Debug, Clone, Serialize)]
pub struct CreateIssueLinkRequest {
    /// The type of link.
    #[serde(rename = "type")]
    pub link_type: IssueLinkTypeRef,
    /// The inward issue (the issue that is affected).
    #[serde(rename = "inwardIssue")]
    pub inward_issue: IssueKeyRef,
    /// The outward issue (the issue that affects).
    #[serde(rename = "outwardIssue")]
    pub outward_issue: IssueKeyRef,
}

/// Reference to an issue link type by name.
#[derive(Debug, Clone, Serialize)]
pub struct IssueLinkTypeRef {
    /// The link type name.
    pub name: String,
}

/// Reference to an issue by key.
#[derive(Debug, Clone, Serialize)]
pub struct IssueKeyRef {
    /// The issue key.
    pub key: String,
}

/// Issue picker response from JIRA.
#[derive(Debug, Clone, Deserialize)]
pub struct IssuePickerResponse {
    /// Sections containing issue suggestions.
    pub sections: Vec<IssuePickerSection>,
}

/// A section in the issue picker response.
#[derive(Debug, Clone, Deserialize)]
pub struct IssuePickerSection {
    /// The section label.
    pub label: String,
    /// The issues in this section.
    pub issues: Vec<IssueSuggestion>,
}

/// An issue suggestion from the picker.
#[derive(Debug, Clone, Deserialize)]
pub struct IssueSuggestion {
    /// The issue key.
    pub key: String,
    /// The issue summary (may contain HTML).
    #[serde(default, rename = "summaryText")]
    pub summary_text: Option<String>,
    /// The issue summary (raw).
    #[serde(default)]
    pub summary: Option<String>,
    /// The issue ID.
    #[serde(default)]
    pub id: Option<i64>,
}

impl IssueSuggestion {
    /// Get the display summary, preferring summary_text over summary.
    pub fn display_summary(&self) -> &str {
        self.summary_text
            .as_deref()
            .or(self.summary.as_deref())
            .unwrap_or("")
    }
}

/// Issue status.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// The status ID.
    #[serde(default)]
    pub id: String,
    /// The status name (e.g., "To Do", "In Progress", "Done").
    #[serde(default)]
    pub name: String,
    /// The status category.
    #[serde(default)]
    pub status_category: Option<StatusCategory>,
}

impl Status {
    /// Check if this status is in the "done" category.
    pub fn is_done(&self) -> bool {
        self.status_category
            .as_ref()
            .map(|c| c.key == "done")
            .unwrap_or(false)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IssueType {
    /// The issue type ID.
    #[serde(default)]
    pub id: String,
    /// The issue type name.
    #[serde(default)]
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

impl fmt::Display for IssueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
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

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
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

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name)
    }
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

/// A comment on a JIRA issue.
///
/// Returned by `GET /rest/api/3/issue/{issueKey}/comment` or as part of issue details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    /// The comment ID.
    pub id: String,
    /// The comment body in Atlassian Document Format.
    pub body: AtlassianDoc,
    /// The user who authored the comment.
    pub author: User,
    /// When the comment was created.
    pub created: String,
    /// When the comment was last updated.
    pub updated: String,
    /// URL to view the comment.
    #[serde(rename = "self", default)]
    pub self_url: Option<String>,
}

/// Comments response from JIRA API.
///
/// Returned by `GET /rest/api/3/issue/{issueKey}/comment`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentsResponse {
    /// The index of the first result.
    pub start_at: u32,
    /// Maximum results requested.
    pub max_results: u32,
    /// Total number of comments.
    pub total: u32,
    /// The list of comments.
    #[serde(default)]
    pub comments: Vec<Comment>,
}

impl CommentsResponse {
    /// Check if there are more pages of results.
    pub fn has_more(&self) -> bool {
        self.start_at + (self.comments.len() as u32) < self.total
    }

    /// Get the starting index for the next page.
    pub fn next_start(&self) -> u32 {
        self.start_at + self.comments.len() as u32
    }
}

/// Request body for adding a comment to an issue.
///
/// Used with `POST /rest/api/3/issue/{issueKey}/comment`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCommentRequest {
    /// The comment body in Atlassian Document Format.
    pub body: AtlassianDoc,
}

impl AddCommentRequest {
    /// Create a new comment request from plain text.
    ///
    /// Converts the plain text into an Atlassian Document Format structure.
    pub fn from_text(text: &str) -> Self {
        Self {
            body: AtlassianDoc::from_text(text),
        }
    }
}

/// Atlassian Document Format (ADF) content.
///
/// JIRA uses ADF for rich text fields like descriptions and comments.
/// This struct represents the document structure and provides methods
/// to extract plain text for display.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtlassianDoc {
    /// The document type (always "doc" for root documents).
    #[serde(rename = "type")]
    pub doc_type: String,
    /// The document version (typically 1).
    #[serde(default)]
    pub version: Option<u32>,
    /// The content nodes within the document.
    #[serde(default)]
    pub content: Vec<serde_json::Value>,
}

impl AtlassianDoc {
    /// Convert ADF content to plain text for display.
    ///
    /// This recursively extracts text nodes from the document structure,
    /// preserving basic formatting like paragraphs and line breaks.
    pub fn to_plain_text(&self) -> String {
        let mut result = String::new();
        for node in &self.content {
            Self::extract_text(node, &mut result);
        }
        result.trim().to_string()
    }

    fn extract_text(node: &serde_json::Value, result: &mut String) {
        match node {
            serde_json::Value::Object(obj) => {
                let node_type = obj.get("type").and_then(|t| t.as_str());

                match node_type {
                    Some("text") => {
                        if let Some(text) = obj.get("text").and_then(|t| t.as_str()) {
                            result.push_str(text);
                        }
                    }
                    Some("paragraph") | Some("heading") => {
                        if let Some(content) = obj.get("content") {
                            if let serde_json::Value::Array(items) = content {
                                for item in items {
                                    Self::extract_text(item, result);
                                }
                            }
                        }
                        if !result.ends_with('\n') && !result.is_empty() {
                            result.push('\n');
                        }
                    }
                    Some("hardBreak") => {
                        result.push('\n');
                    }
                    Some("bulletList") | Some("orderedList") => {
                        if let Some(content) = obj.get("content") {
                            if let serde_json::Value::Array(items) = content {
                                for item in items {
                                    Self::extract_text(item, result);
                                }
                            }
                        }
                    }
                    Some("listItem") => {
                        result.push_str("• ");
                        if let Some(content) = obj.get("content") {
                            if let serde_json::Value::Array(items) = content {
                                for item in items {
                                    Self::extract_text(item, result);
                                }
                            }
                        }
                    }
                    Some("codeBlock") => {
                        if let Some(content) = obj.get("content") {
                            if let serde_json::Value::Array(items) = content {
                                for item in items {
                                    Self::extract_text(item, result);
                                }
                            }
                        }
                        if !result.ends_with('\n') {
                            result.push('\n');
                        }
                    }
                    Some("blockquote") => {
                        result.push_str("> ");
                        if let Some(content) = obj.get("content") {
                            if let serde_json::Value::Array(items) = content {
                                for item in items {
                                    Self::extract_text(item, result);
                                }
                            }
                        }
                    }
                    Some("mention") => {
                        if let Some(attrs) = obj.get("attrs") {
                            if let Some(text) = attrs.get("text").and_then(|t| t.as_str()) {
                                result.push('@');
                                result.push_str(text);
                            }
                        }
                    }
                    Some("emoji") => {
                        if let Some(attrs) = obj.get("attrs") {
                            if let Some(shortname) = attrs.get("shortName").and_then(|s| s.as_str())
                            {
                                result.push_str(shortname);
                            }
                        }
                    }
                    Some("inlineCard") | Some("mediaGroup") | Some("mediaSingle") => {
                        // Skip media/card nodes, they don't have useful text representation
                    }
                    _ => {
                        // For unknown nodes, try to recurse into content
                        if let Some(content) = obj.get("content") {
                            if let serde_json::Value::Array(items) = content {
                                for item in items {
                                    Self::extract_text(item, result);
                                }
                            }
                        }
                    }
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    Self::extract_text(item, result);
                }
            }
            _ => {}
        }
    }

    /// Create an ADF document from plain text.
    ///
    /// Converts plain text into an Atlassian Document Format structure
    /// by splitting on newlines and creating paragraph nodes.
    pub fn from_text(text: &str) -> Self {
        let paragraphs: Vec<serde_json::Value> = text
            .lines()
            .map(|line| {
                if line.is_empty() {
                    // Empty paragraph
                    serde_json::json!({
                        "type": "paragraph",
                        "content": []
                    })
                } else {
                    serde_json::json!({
                        "type": "paragraph",
                        "content": [{
                            "type": "text",
                            "text": line
                        }]
                    })
                }
            })
            .collect();

        Self {
            doc_type: "doc".to_string(),
            version: Some(1),
            content: paragraphs,
        }
    }
}

impl Default for AtlassianDoc {
    fn default() -> Self {
        Self {
            doc_type: "doc".to_string(),
            version: Some(1),
            content: vec![],
        }
    }
}

impl AtlassianDoc {
    /// Create an Atlassian Document from plain text.
    ///
    /// Each line becomes a paragraph. Empty lines are preserved.
    pub fn from_plain_text(text: &str) -> Self {
        let content = text
            .lines()
            .map(|line| {
                serde_json::json!({
                    "type": "paragraph",
                    "content": if line.is_empty() {
                        vec![]
                    } else {
                        vec![serde_json::json!({
                            "type": "text",
                            "text": line
                        })]
                    }
                })
            })
            .collect();

        Self {
            doc_type: "doc".to_string(),
            version: Some(1),
            content,
        }
    }
}

// ============================================================================
// Filter Types
// ============================================================================

/// Sprint filter options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SprintFilter {
    /// Filter by current/open sprints.
    Current,
    /// Filter by a specific sprint ID.
    Specific(String),
}

/// Filter state for issues.
///
/// This struct holds all the filter criteria that can be applied to issues.
/// It can generate a JQL query string from the current filter state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterState {
    /// Filter by statuses (multi-select).
    pub statuses: Vec<String>,
    /// Filter by assignee account IDs (multi-select).
    pub assignees: Vec<String>,
    /// Special flag to filter by current user ("Assigned to me").
    pub assignee_is_me: bool,
    /// Filter by project key.
    pub project: Option<String>,
    /// Filter by labels (multi-select).
    pub labels: Vec<String>,
    /// Filter by components (multi-select).
    pub components: Vec<String>,
    /// Filter by sprint.
    pub sprint: Option<SprintFilter>,
    /// Filter by epic keys (multi-select).
    pub epics: Vec<String>,
}

impl FilterState {
    /// Create a new empty filter state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert the filter state to a JQL query string.
    ///
    /// Returns an empty string if no filters are active.
    pub fn to_jql(&self) -> String {
        let mut clauses = Vec::new();

        if !self.statuses.is_empty() {
            let statuses = self
                .statuses
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ");
            clauses.push(format!("status IN ({})", statuses));
        }

        if self.assignee_is_me {
            clauses.push("assignee = currentUser()".to_string());
        } else if !self.assignees.is_empty() {
            let assignees = self
                .assignees
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ");
            clauses.push(format!("assignee IN ({})", assignees));
        }

        if let Some(project) = &self.project {
            clauses.push(format!("project = \"{}\"", project));
        }

        if !self.labels.is_empty() {
            let labels = self
                .labels
                .iter()
                .map(|l| format!("\"{}\"", l))
                .collect::<Vec<_>>()
                .join(", ");
            clauses.push(format!("labels IN ({})", labels));
        }

        if !self.components.is_empty() {
            let components = self
                .components
                .iter()
                .map(|c| format!("\"{}\"", c))
                .collect::<Vec<_>>()
                .join(", ");
            clauses.push(format!("component IN ({})", components));
        }

        match &self.sprint {
            Some(SprintFilter::Current) => {
                clauses.push("sprint IN openSprints()".to_string());
            }
            Some(SprintFilter::Specific(id)) => {
                clauses.push(format!("sprint = {}", id));
            }
            None => {}
        }

        if !self.epics.is_empty() {
            let epics = self
                .epics
                .iter()
                .map(|e| format!("\"{}\"", e))
                .collect::<Vec<_>>()
                .join(", ");
            clauses.push(format!("parent IN ({})", epics));
        }

        if clauses.is_empty() {
            String::new()
        } else {
            clauses.join(" AND ")
        }
    }

    /// Check if the filter state is empty (no filters applied).
    pub fn is_empty(&self) -> bool {
        self.statuses.is_empty()
            && self.assignees.is_empty()
            && !self.assignee_is_me
            && self.project.is_none()
            && self.labels.is_empty()
            && self.components.is_empty()
            && self.sprint.is_none()
            && self.epics.is_empty()
    }

    /// Clear all filters.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Get a summary of active filters for display.
    pub fn summary(&self) -> Vec<String> {
        let mut parts = Vec::new();

        if !self.statuses.is_empty() {
            parts.push(format!("Status: {}", self.statuses.join(", ")));
        }

        if self.assignee_is_me {
            parts.push("Assigned to me".to_string());
        } else if !self.assignees.is_empty() {
            parts.push(format!("Assignee: {} selected", self.assignees.len()));
        }

        if let Some(project) = &self.project {
            parts.push(format!("Project: {}", project));
        }

        if !self.labels.is_empty() {
            parts.push(format!("Labels: {}", self.labels.join(", ")));
        }

        if !self.components.is_empty() {
            parts.push(format!("Components: {}", self.components.join(", ")));
        }

        match &self.sprint {
            Some(SprintFilter::Current) => {
                parts.push("Sprint: Current".to_string());
            }
            Some(SprintFilter::Specific(name)) => {
                parts.push(format!("Sprint: {}", name));
            }
            None => {}
        }

        if !self.epics.is_empty() {
            parts.push(format!("Epic: {}", self.epics.join(", ")));
        }

        parts
    }

    /// Toggle a status in the filter.
    pub fn toggle_status(&mut self, status: &str) {
        if let Some(pos) = self.statuses.iter().position(|s| s == status) {
            self.statuses.remove(pos);
        } else {
            self.statuses.push(status.to_string());
        }
    }

    /// Toggle a label in the filter.
    pub fn toggle_label(&mut self, label: &str) {
        if let Some(pos) = self.labels.iter().position(|l| l == label) {
            self.labels.remove(pos);
        } else {
            self.labels.push(label.to_string());
        }
    }

    /// Toggle a component in the filter.
    pub fn toggle_component(&mut self, component: &str) {
        if let Some(pos) = self.components.iter().position(|c| c == component) {
            self.components.remove(pos);
        } else {
            self.components.push(component.to_string());
        }
    }

    /// Toggle an assignee in the filter.
    pub fn toggle_assignee(&mut self, account_id: &str) {
        if let Some(pos) = self.assignees.iter().position(|a| a == account_id) {
            self.assignees.remove(pos);
        } else {
            self.assignees.push(account_id.to_string());
        }
    }

    /// Set the project filter.
    pub fn set_project(&mut self, project: Option<String>) {
        self.project = project;
    }

    /// Set the sprint filter.
    pub fn set_sprint(&mut self, sprint: Option<SprintFilter>) {
        self.sprint = sprint;
    }

    /// Toggle "Assigned to me" filter.
    pub fn toggle_assigned_to_me(&mut self) {
        self.assignee_is_me = !self.assignee_is_me;
        if self.assignee_is_me {
            // Clear other assignees when "Assigned to me" is selected
            self.assignees.clear();
        }
    }
}

/// A selectable filter option.
#[derive(Debug, Clone)]
pub struct FilterOption {
    /// The unique identifier for this option.
    pub id: String,
    /// The display label for this option.
    pub label: String,
}

impl FilterOption {
    /// Create a new filter option.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// Available filter options fetched from JIRA.
#[derive(Debug, Clone, Default)]
pub struct FilterOptions {
    /// Available statuses.
    pub statuses: Vec<FilterOption>,
    /// Available users/assignees.
    pub users: Vec<FilterOption>,
    /// Available projects.
    pub projects: Vec<FilterOption>,
    /// Available labels.
    pub labels: Vec<FilterOption>,
    /// Available components.
    pub components: Vec<FilterOption>,
    /// Available sprints.
    pub sprints: Vec<FilterOption>,
    /// Available epics.
    pub epics: Vec<FilterOption>,
}

impl FilterOptions {
    /// Create empty filter options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if filter options have been loaded.
    pub fn is_loaded(&self) -> bool {
        // Consider loaded if we have at least statuses
        !self.statuses.is_empty()
    }

    /// Add a label if it doesn't already exist.
    pub fn add_label(&mut self, label: &str) {
        if !self.labels.iter().any(|l| l.id == label) {
            self.labels.push(FilterOption::new(label, label));
            // Keep labels sorted
            self.labels.sort_by(|a, b| a.label.cmp(&b.label));
        }
    }

    /// Add a component if it doesn't already exist.
    pub fn add_component(&mut self, component: &str) {
        if !self.components.iter().any(|c| c.id == component) {
            self.components.push(FilterOption::new(component, component));
            // Keep components sorted
            self.components.sort_by(|a, b| a.label.cmp(&b.label));
        }
    }

    /// Add an epic if it doesn't already exist.
    pub fn add_epic(&mut self, key: &str, summary: &str) {
        if !self.epics.iter().any(|e| e.id == key) {
            self.epics
                .push(FilterOption::new(key, &format!("{} - {}", key, summary)));
            // Keep epics sorted by key
            self.epics.sort_by(|a, b| a.id.cmp(&b.id));
        }
    }
}

/// A JIRA sprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sprint {
    /// The sprint ID.
    pub id: u64,
    /// The sprint name.
    pub name: String,
    /// The sprint state (future, active, closed).
    pub state: String,
    /// The sprint start date.
    #[serde(default)]
    pub start_date: Option<String>,
    /// The sprint end date.
    #[serde(default)]
    pub end_date: Option<String>,
}

/// Response from the sprints endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SprintsResponse {
    /// Maximum results per page.
    pub max_results: u32,
    /// Starting index.
    pub start_at: u32,
    /// Whether this is the last page.
    #[serde(default)]
    pub is_last: bool,
    /// The list of sprints.
    #[serde(default)]
    pub values: Vec<Sprint>,
}

/// Response from the labels endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelsResponse {
    /// The list of labels.
    pub values: Vec<String>,
    /// Total number of labels.
    #[serde(default)]
    pub total: u32,
    /// Maximum results per page.
    #[serde(default, rename = "maxResults")]
    pub max_results: u32,
    /// Starting index.
    #[serde(default, rename = "startAt")]
    pub start_at: u32,
    /// Whether this is the last page.
    #[serde(default, rename = "isLast")]
    pub is_last: bool,
}

/// A JIRA board.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// The board ID.
    pub id: u64,
    /// The board name.
    pub name: String,
    /// The board type (scrum, kanban).
    #[serde(rename = "type")]
    pub board_type: String,
}

/// Response from the boards endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardsResponse {
    /// Maximum results per page.
    pub max_results: u32,
    /// Starting index.
    pub start_at: u32,
    /// Whether this is the last page.
    #[serde(default)]
    pub is_last: bool,
    /// The list of boards.
    #[serde(default)]
    pub values: Vec<Board>,
}

// ============================================================================
// Issue Update Types
// ============================================================================

/// Request body for updating an issue.
///
/// Uses the JIRA REST API v3 issue update format.
/// Both `fields` and `update` can be used together.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct IssueUpdateRequest {
    /// Direct field updates (simple set operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<FieldUpdates>,
    /// Complex update operations (add, remove, set).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<UpdateOperations>,
}

/// Direct field updates for an issue.
///
/// Each field that is `Some` will be updated to the provided value.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct FieldUpdates {
    /// Update the issue summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Update the issue description (in Atlassian Document Format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<AtlassianDoc>,
    /// Update the assignee. Use `NullableUserRef::unassign()` to unassign,
    /// or `NullableUserRef::assign(id)` to assign.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<NullableUserRef>,
    /// Update the priority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<PriorityRef>,
    /// Update story points (customfield_10016).
    #[serde(rename = "customfield_10016", skip_serializing_if = "Option::is_none")]
    pub story_points: Option<f32>,
    /// Update sprint assignment (customfield_10020).
    #[serde(rename = "customfield_10020", skip_serializing_if = "Option::is_none")]
    pub sprint: Option<i64>,
}

/// Reference to a user by account ID.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct UserRef {
    /// The user's account ID.
    #[serde(rename = "accountId")]
    pub account_id: String,
}

impl UserRef {
    /// Create a new user reference.
    pub fn new(account_id: impl Into<String>) -> Self {
        Self {
            account_id: account_id.into(),
        }
    }

    /// Create an "unassigned" user reference (null assignee).
    pub fn unassigned() -> Option<Self> {
        None
    }
}

/// Nullable user reference that explicitly serializes as `null` when unassigning.
///
/// This is needed because Jira API requires `"assignee": null` to unassign,
/// but serde's `skip_serializing_if` would omit the field entirely.
///
/// - `NullableUserRef(None)` serializes as `null` (unassign)
/// - `NullableUserRef(Some(ref))` serializes as the user ref (assign)
#[derive(Debug, Clone, PartialEq)]
pub struct NullableUserRef(pub Option<UserRef>);

impl NullableUserRef {
    /// Create a nullable reference for assigning to a user.
    pub fn assign(account_id: impl Into<String>) -> Self {
        Self(Some(UserRef::new(account_id)))
    }

    /// Create a nullable reference for unassigning (serializes as null).
    pub fn unassign() -> Self {
        Self(None)
    }
}

impl serde::Serialize for NullableUserRef {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            Some(user_ref) => user_ref.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}

/// Reference to a priority by ID.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PriorityRef {
    /// The priority ID.
    pub id: String,
}

impl PriorityRef {
    /// Create a new priority reference.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// Complex update operations for list fields.
///
/// Used for operations like add/remove on labels and components.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct UpdateOperations {
    /// Label add/remove operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<LabelOperation>>,
    /// Component add/remove operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<ComponentOperation>>,
}

/// Operation to add or remove a label.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LabelOperation {
    /// Add a label.
    Add(String),
    /// Remove a label.
    Remove(String),
}

/// Operation to add or remove a component.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ComponentOperation {
    /// Add a component by name.
    Add { name: String },
    /// Remove a component by name.
    Remove { name: String },
}

// ============================================================================
// Transition Types
// ============================================================================

/// Response from the transitions endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TransitionsResponse {
    /// Available transitions for the issue.
    pub transitions: Vec<Transition>,
}

/// A workflow transition for an issue.
#[derive(Debug, Clone, Deserialize)]
pub struct Transition {
    /// The transition ID.
    pub id: String,
    /// The transition name (e.g., "Start Progress", "Done").
    pub name: String,
    /// The target status after this transition.
    pub to: TransitionTarget,
    /// Fields that may be required or available during this transition.
    #[serde(default)]
    pub fields: std::collections::HashMap<String, TransitionField>,
}

/// The target status of a transition.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionTarget {
    /// The status ID.
    pub id: String,
    /// The status name.
    pub name: String,
    /// The status category.
    #[serde(default)]
    pub status_category: Option<StatusCategory>,
}

/// A field that may be required during a transition.
#[derive(Debug, Clone, Deserialize)]
pub struct TransitionField {
    /// Whether this field is required for the transition.
    pub required: bool,
    /// The field name.
    pub name: String,
}

/// Request to perform a status transition.
#[derive(Debug, Clone, Serialize)]
pub struct TransitionRequest {
    /// The transition to perform.
    pub transition: TransitionRef,
    /// Optional fields to set during the transition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<FieldUpdates>,
}

/// Reference to a transition by ID.
#[derive(Debug, Clone, Serialize)]
pub struct TransitionRef {
    /// The transition ID.
    pub id: String,
}

impl TransitionRef {
    /// Create a new transition reference.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

// ============================================================================
// Changelog Types
// ============================================================================

/// Paginated changelog response from JIRA API.
///
/// Returned by `GET /rest/api/3/issue/{issueKey}/changelog`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Changelog {
    /// The list of history entries.
    #[serde(default, rename = "values")]
    pub histories: Vec<ChangeHistory>,
    /// Starting index for pagination.
    #[serde(default)]
    pub start_at: u32,
    /// Maximum results per page.
    #[serde(default)]
    pub max_results: u32,
    /// Total number of history entries.
    #[serde(default)]
    pub total: u32,
    /// Whether this is the last page.
    #[serde(default)]
    pub is_last: bool,
}

impl Changelog {
    /// Check if there are more pages of results.
    pub fn has_more(&self) -> bool {
        if self.is_last {
            return false;
        }
        self.start_at + (self.histories.len() as u32) < self.total
    }

    /// Get the starting index for the next page.
    pub fn next_start(&self) -> u32 {
        self.start_at + self.histories.len() as u32
    }
}

/// A single history entry representing a set of changes made at one time.
#[derive(Debug, Clone, Deserialize)]
pub struct ChangeHistory {
    /// The history entry ID.
    pub id: String,
    /// The user who made the changes.
    pub author: User,
    /// When the changes were made (ISO 8601 format).
    pub created: String,
    /// The list of individual field changes.
    pub items: Vec<ChangeItem>,
}

/// A single field change within a history entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeItem {
    /// The field that was changed.
    pub field: String,
    /// The type of field (jira, custom, etc.).
    #[serde(default)]
    pub field_type: Option<String>,
    /// The internal "from" value.
    #[serde(rename = "from")]
    pub from_value: Option<String>,
    /// The display "from" value.
    #[serde(rename = "fromString")]
    pub from_string: Option<String>,
    /// The internal "to" value.
    #[serde(rename = "to")]
    pub to_value: Option<String>,
    /// The display "to" value.
    #[serde(rename = "toString")]
    pub to_string: Option<String>,
}

impl ChangeItem {
    /// Get the display value for "from" (prefers fromString over from).
    pub fn display_from(&self) -> &str {
        self.from_string
            .as_deref()
            .or(self.from_value.as_deref())
            .unwrap_or("(none)")
    }

    /// Get the display value for "to" (prefers toString over to).
    pub fn display_to(&self) -> &str {
        self.to_string
            .as_deref()
            .or(self.to_value.as_deref())
            .unwrap_or("(none)")
    }

    /// Get the category of change based on the field name.
    pub fn change_type(&self) -> ChangeType {
        match self.field.to_lowercase().as_str() {
            "status" => ChangeType::Status,
            "assignee" => ChangeType::Assignee,
            "priority" => ChangeType::Priority,
            "summary" | "description" => ChangeType::Content,
            "labels" | "label" => ChangeType::Tags,
            "component" | "components" => ChangeType::Tags,
            "sprint" => ChangeType::Sprint,
            "fix version" | "fixversions" => ChangeType::Version,
            "resolution" => ChangeType::Resolution,
            "link" | "issuelinks" => ChangeType::Link,
            "attachment" => ChangeType::Attachment,
            "comment" => ChangeType::Comment,
            _ => ChangeType::Other,
        }
    }
}

/// Categories of changes for visual styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    /// Status/workflow changes.
    Status,
    /// Assignee changes.
    Assignee,
    /// Priority changes.
    Priority,
    /// Content changes (summary, description).
    Content,
    /// Tag changes (labels, components).
    Tags,
    /// Sprint changes.
    Sprint,
    /// Version changes.
    Version,
    /// Resolution changes.
    Resolution,
    /// Link changes.
    Link,
    /// Attachment changes.
    Attachment,
    /// Comment changes.
    Comment,
    /// Other field changes.
    Other,
}

impl ChangeType {
    /// Get an icon for this change type.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Status => "->",
            Self::Assignee => "@",
            Self::Priority => "!",
            Self::Content => "#",
            Self::Tags => "*",
            Self::Sprint => "~",
            Self::Version => "v",
            Self::Resolution => "x",
            Self::Link => "+",
            Self::Attachment => "^",
            Self::Comment => ">",
            Self::Other => "-",
        }
    }
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
            next_page_token: None,
            is_last: false,
        };
        assert!(result.has_more());

        // Last page: start=50, got 50 issues, total is 100 -> no more
        let result = SearchResult {
            start_at: 50,
            max_results: 50,
            total: 100,
            issues: (0..50).map(|_| create_test_issue()).collect(),
            next_page_token: None,
            is_last: false,
        };
        assert!(!result.has_more());

        // Partial last page: start=90, got 10 issues, total is 100 -> no more
        let result = SearchResult {
            start_at: 90,
            max_results: 50,
            total: 100,
            issues: (0..10).map(|_| create_test_issue()).collect(),
            next_page_token: None,
            is_last: false,
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
                issue_links: vec![],
                subtasks: vec![],
                parent: None,
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
            next_page_token: None,
            is_last: false,
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

    #[test]
    fn test_issue_convenience_methods() {
        let issue = create_test_issue();
        assert_eq!(issue.assignee_name(), "Unassigned");
        assert_eq!(issue.priority_name(), "None");
        assert_eq!(issue.description_text(), "");
        assert!(issue.project_key().is_none());

        // Issue with assignee and priority
        let issue_with_data = Issue {
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
                priority: Some(Priority {
                    id: "2".to_string(),
                    name: "High".to_string(),
                    icon_url: None,
                }),
                assignee: Some(User {
                    account_id: "abc123".to_string(),
                    display_name: "John Doe".to_string(),
                    email_address: None,
                    active: true,
                    avatar_urls: None,
                }),
                reporter: None,
                project: Some(Project {
                    id: "10000".to_string(),
                    key: "TEST".to_string(),
                    name: "Test Project".to_string(),
                    avatar_urls: None,
                }),
                labels: vec![],
                components: vec![],
                created: None,
                updated: None,
                duedate: None,
                story_points: None,
                issue_links: vec![],
                subtasks: vec![],
                parent: None,
            },
        };
        assert_eq!(issue_with_data.assignee_name(), "John Doe");
        assert_eq!(issue_with_data.priority_name(), "High");
        assert_eq!(issue_with_data.project_key(), Some("TEST"));
    }

    #[test]
    fn test_issue_display() {
        let issue = create_test_issue();
        assert_eq!(format!("{}", issue), "TEST-1: Test");
    }

    #[test]
    fn test_status_display() {
        let status = Status {
            id: "1".to_string(),
            name: "In Progress".to_string(),
            status_category: None,
        };
        assert_eq!(format!("{}", status), "In Progress");
    }

    #[test]
    fn test_priority_display() {
        let priority = Priority {
            id: "2".to_string(),
            name: "High".to_string(),
            icon_url: None,
        };
        assert_eq!(format!("{}", priority), "High");
    }

    #[test]
    fn test_issue_type_display() {
        let issue_type = IssueType {
            id: "1".to_string(),
            name: "Bug".to_string(),
            subtask: false,
            description: None,
            icon_url: None,
        };
        assert_eq!(format!("{}", issue_type), "Bug");
    }

    #[test]
    fn test_user_display() {
        let user = User {
            account_id: "abc123".to_string(),
            display_name: "Jane Smith".to_string(),
            email_address: Some("jane@example.com".to_string()),
            active: true,
            avatar_urls: None,
        };
        assert_eq!(format!("{}", user), "Jane Smith");
    }

    #[test]
    fn test_atlassian_doc_simple_paragraph() {
        let json = r#"{
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "Hello, world!"}
                    ]
                }
            ]
        }"#;

        let doc: AtlassianDoc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.to_plain_text(), "Hello, world!");
    }

    #[test]
    fn test_atlassian_doc_multiple_paragraphs() {
        let json = r#"{
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "First paragraph."}
                    ]
                },
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "Second paragraph."}
                    ]
                }
            ]
        }"#;

        let doc: AtlassianDoc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.to_plain_text(), "First paragraph.\nSecond paragraph.");
    }

    #[test]
    fn test_atlassian_doc_bullet_list() {
        let json = r#"{
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "bulletList",
                    "content": [
                        {
                            "type": "listItem",
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [
                                        {"type": "text", "text": "Item one"}
                                    ]
                                }
                            ]
                        },
                        {
                            "type": "listItem",
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [
                                        {"type": "text", "text": "Item two"}
                                    ]
                                }
                            ]
                        }
                    ]
                }
            ]
        }"#;

        let doc: AtlassianDoc = serde_json::from_str(json).unwrap();
        let text = doc.to_plain_text();
        assert!(text.contains("• Item one"));
        assert!(text.contains("• Item two"));
    }

    #[test]
    fn test_atlassian_doc_heading() {
        let json = r#"{
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "heading",
                    "attrs": {"level": 1},
                    "content": [
                        {"type": "text", "text": "Title"}
                    ]
                },
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "Body text."}
                    ]
                }
            ]
        }"#;

        let doc: AtlassianDoc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.to_plain_text(), "Title\nBody text.");
    }

    #[test]
    fn test_atlassian_doc_code_block() {
        let json = r#"{
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "codeBlock",
                    "attrs": {"language": "rust"},
                    "content": [
                        {"type": "text", "text": "fn main() {}"}
                    ]
                }
            ]
        }"#;

        let doc: AtlassianDoc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.to_plain_text(), "fn main() {}");
    }

    #[test]
    fn test_atlassian_doc_mention() {
        let json = r#"{
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "Hello "},
                        {
                            "type": "mention",
                            "attrs": {
                                "id": "abc123",
                                "text": "John Doe"
                            }
                        },
                        {"type": "text", "text": "!"}
                    ]
                }
            ]
        }"#;

        let doc: AtlassianDoc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.to_plain_text(), "Hello @John Doe!");
    }

    #[test]
    fn test_atlassian_doc_empty() {
        let doc = AtlassianDoc::default();
        assert_eq!(doc.to_plain_text(), "");
    }

    #[test]
    fn test_atlassian_doc_hard_break() {
        let json = r#"{
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "Line one"},
                        {"type": "hardBreak"},
                        {"type": "text", "text": "Line two"}
                    ]
                }
            ]
        }"#;

        let doc: AtlassianDoc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.to_plain_text(), "Line one\nLine two");
    }

    #[test]
    fn test_parse_comment() {
        let json = r#"{
            "id": "10001",
            "body": {
                "type": "doc",
                "version": 1,
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {"type": "text", "text": "This is a comment."}
                        ]
                    }
                ]
            },
            "author": {
                "accountId": "abc123",
                "displayName": "John Doe",
                "active": true
            },
            "created": "2024-01-15T10:00:00.000+0000",
            "updated": "2024-01-15T10:00:00.000+0000"
        }"#;

        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "10001");
        assert_eq!(comment.author.display_name, "John Doe");
        assert_eq!(comment.body.to_plain_text(), "This is a comment.");
    }

    #[test]
    fn test_parse_comments_response() {
        let json = r#"{
            "startAt": 0,
            "maxResults": 50,
            "total": 1,
            "comments": [
                {
                    "id": "10001",
                    "body": {
                        "type": "doc",
                        "version": 1,
                        "content": []
                    },
                    "author": {
                        "accountId": "abc123",
                        "displayName": "Test User",
                        "active": true
                    },
                    "created": "2024-01-15T10:00:00.000+0000",
                    "updated": "2024-01-15T10:00:00.000+0000"
                }
            ]
        }"#;

        let response: CommentsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total, 1);
        assert_eq!(response.comments.len(), 1);
        assert!(!response.has_more());
    }

    #[test]
    fn test_issue_with_adf_description() {
        let json = r#"{
            "id": "10001",
            "key": "PROJ-123",
            "self": "https://example.com",
            "fields": {
                "summary": "Test issue",
                "description": {
                    "type": "doc",
                    "version": 1,
                    "content": [
                        {
                            "type": "paragraph",
                            "content": [
                                {"type": "text", "text": "This is the description."}
                            ]
                        }
                    ]
                },
                "status": {"id": "1", "name": "Open"},
                "issuetype": {"id": "1", "name": "Bug"}
            }
        }"#;

        let issue: Issue = serde_json::from_str(json).unwrap();
        assert_eq!(issue.description_text(), "This is the description.");
    }

    #[test]
    fn test_parse_issue_with_null_fields() {
        let json = r#"{
            "id": "10001",
            "key": "PROJ-123",
            "self": "https://example.com",
            "fields": {
                "summary": "Test issue",
                "description": null,
                "status": {"id": "1", "name": "Open"},
                "issuetype": {"id": "1", "name": "Bug"},
                "priority": null,
                "assignee": null,
                "reporter": null,
                "project": null,
                "labels": [],
                "components": []
            }
        }"#;

        let issue: Issue = serde_json::from_str(json).unwrap();
        assert_eq!(issue.key, "PROJ-123");
        assert!(issue.priority().is_none());
        assert!(issue.assignee().is_none());
        assert_eq!(issue.assignee_name(), "Unassigned");
        assert_eq!(issue.priority_name(), "None");
    }

    // ========================================================================
    // FilterState tests
    // ========================================================================

    #[test]
    fn test_filter_state_default() {
        let filter = FilterState::default();
        assert!(filter.is_empty());
        assert_eq!(filter.to_jql(), "");
    }

    #[test]
    fn test_filter_state_with_statuses() {
        let mut filter = FilterState::new();
        filter.toggle_status("Open");
        filter.toggle_status("In Progress");

        assert!(!filter.is_empty());
        let jql = filter.to_jql();
        assert_eq!(jql, r#"status IN ("Open", "In Progress")"#);
    }

    #[test]
    fn test_filter_state_toggle_status_removes() {
        let mut filter = FilterState::new();
        filter.toggle_status("Open");
        assert_eq!(filter.statuses.len(), 1);

        filter.toggle_status("Open");
        assert!(filter.statuses.is_empty());
    }

    #[test]
    fn test_filter_state_assigned_to_me() {
        let mut filter = FilterState::new();
        filter.toggle_assigned_to_me();

        assert!(!filter.is_empty());
        assert!(filter.assignee_is_me);
        assert_eq!(filter.to_jql(), "assignee = currentUser()");
    }

    #[test]
    fn test_filter_state_assigned_to_me_clears_assignees() {
        let mut filter = FilterState::new();
        filter.toggle_assignee("user1");
        filter.toggle_assignee("user2");
        assert_eq!(filter.assignees.len(), 2);

        filter.toggle_assigned_to_me();
        assert!(filter.assignee_is_me);
        assert!(filter.assignees.is_empty());
    }

    #[test]
    fn test_filter_state_with_assignees() {
        let mut filter = FilterState::new();
        filter.toggle_assignee("user1");
        filter.toggle_assignee("user2");

        let jql = filter.to_jql();
        assert_eq!(jql, r#"assignee IN ("user1", "user2")"#);
    }

    #[test]
    fn test_filter_state_with_project() {
        let mut filter = FilterState::new();
        filter.set_project(Some("PROJ".to_string()));

        assert_eq!(filter.to_jql(), r#"project = "PROJ""#);
    }

    #[test]
    fn test_filter_state_with_labels() {
        let mut filter = FilterState::new();
        filter.toggle_label("bug");
        filter.toggle_label("urgent");

        assert_eq!(filter.to_jql(), r#"labels IN ("bug", "urgent")"#);
    }

    #[test]
    fn test_filter_state_with_components() {
        let mut filter = FilterState::new();
        filter.toggle_component("frontend");
        filter.toggle_component("api");

        assert_eq!(filter.to_jql(), r#"component IN ("frontend", "api")"#);
    }

    #[test]
    fn test_filter_state_with_current_sprint() {
        let mut filter = FilterState::new();
        filter.set_sprint(Some(SprintFilter::Current));

        assert_eq!(filter.to_jql(), "sprint IN openSprints()");
    }

    #[test]
    fn test_filter_state_with_specific_sprint() {
        let mut filter = FilterState::new();
        filter.set_sprint(Some(SprintFilter::Specific("123".to_string())));

        assert_eq!(filter.to_jql(), "sprint = 123");
    }

    #[test]
    fn test_filter_state_combined_filters() {
        let mut filter = FilterState::new();
        filter.toggle_status("Open");
        filter.toggle_assigned_to_me();
        filter.set_project(Some("PROJ".to_string()));
        filter.toggle_label("bug");
        filter.set_sprint(Some(SprintFilter::Current));

        let jql = filter.to_jql();
        assert!(jql.contains(r#"status IN ("Open")"#));
        assert!(jql.contains("assignee = currentUser()"));
        assert!(jql.contains(r#"project = "PROJ""#));
        assert!(jql.contains(r#"labels IN ("bug")"#));
        assert!(jql.contains("sprint IN openSprints()"));
        // All clauses connected with AND
        assert_eq!(jql.matches(" AND ").count(), 4);
    }

    #[test]
    fn test_filter_state_clear() {
        let mut filter = FilterState::new();
        filter.toggle_status("Open");
        filter.toggle_assigned_to_me();
        filter.set_project(Some("PROJ".to_string()));

        assert!(!filter.is_empty());

        filter.clear();
        assert!(filter.is_empty());
        assert_eq!(filter.to_jql(), "");
    }

    #[test]
    fn test_filter_state_summary() {
        let mut filter = FilterState::new();
        filter.toggle_status("Open");
        filter.toggle_assigned_to_me();
        filter.set_project(Some("PROJ".to_string()));

        let summary = filter.summary();
        assert!(summary.iter().any(|s| s.contains("Status: Open")));
        assert!(summary.iter().any(|s| s.contains("Assigned to me")));
        assert!(summary.iter().any(|s| s.contains("Project: PROJ")));
    }

    #[test]
    fn test_filter_option_new() {
        let opt = FilterOption::new("id123", "My Label");
        assert_eq!(opt.id, "id123");
        assert_eq!(opt.label, "My Label");
    }

    #[test]
    fn test_filter_options_is_loaded() {
        let mut opts = FilterOptions::new();
        assert!(!opts.is_loaded());

        opts.statuses.push(FilterOption::new("1", "Open"));
        assert!(opts.is_loaded());
    }

    #[test]
    fn test_parse_sprint() {
        let json = r#"{
            "id": 123,
            "name": "Sprint 1",
            "state": "active"
        }"#;

        let sprint: Sprint = serde_json::from_str(json).unwrap();
        assert_eq!(sprint.id, 123);
        assert_eq!(sprint.name, "Sprint 1");
        assert_eq!(sprint.state, "active");
    }

    #[test]
    fn test_parse_board() {
        let json = r#"{
            "id": 1,
            "name": "My Board",
            "type": "scrum"
        }"#;

        let board: Board = serde_json::from_str(json).unwrap();
        assert_eq!(board.id, 1);
        assert_eq!(board.name, "My Board");
        assert_eq!(board.board_type, "scrum");
    }

    // ========================================================================
    // Issue Update Types tests
    // ========================================================================

    #[test]
    fn test_issue_update_request_serialization() {
        let update = IssueUpdateRequest {
            fields: Some(FieldUpdates {
                summary: Some("New summary".to_string()),
                ..Default::default()
            }),
            update: None,
        };

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"summary\":\"New summary\""));
        assert!(!json.contains("\"update\""));
    }

    #[test]
    fn test_issue_update_request_empty() {
        let update = IssueUpdateRequest::default();
        let json = serde_json::to_string(&update).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_field_updates_serialization() {
        let fields = FieldUpdates {
            summary: Some("Test".to_string()),
            story_points: Some(5.0),
            ..Default::default()
        };

        let json = serde_json::to_string(&fields).unwrap();
        assert!(json.contains("\"summary\":\"Test\""));
        assert!(json.contains("\"customfield_10016\":5.0"));
    }

    #[test]
    fn test_user_ref_new() {
        let user_ref = UserRef::new("abc123");
        assert_eq!(user_ref.account_id, "abc123");
    }

    #[test]
    fn test_user_ref_serialization() {
        let user_ref = UserRef::new("abc123");
        let json = serde_json::to_string(&user_ref).unwrap();
        assert_eq!(json, r#"{"accountId":"abc123"}"#);
    }

    #[test]
    fn test_priority_ref_new() {
        let priority_ref = PriorityRef::new("2");
        assert_eq!(priority_ref.id, "2");
    }

    #[test]
    fn test_priority_ref_serialization() {
        let priority_ref = PriorityRef::new("2");
        let json = serde_json::to_string(&priority_ref).unwrap();
        assert_eq!(json, r#"{"id":"2"}"#);
    }

    #[test]
    fn test_label_operation_add_serialization() {
        let op = LabelOperation::Add("bug".to_string());
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, r#"{"add":"bug"}"#);
    }

    #[test]
    fn test_label_operation_remove_serialization() {
        let op = LabelOperation::Remove("bug".to_string());
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, r#"{"remove":"bug"}"#);
    }

    #[test]
    fn test_component_operation_add_serialization() {
        let op = ComponentOperation::Add {
            name: "frontend".to_string(),
        };
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, r#"{"add":{"name":"frontend"}}"#);
    }

    #[test]
    fn test_component_operation_remove_serialization() {
        let op = ComponentOperation::Remove {
            name: "frontend".to_string(),
        };
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, r#"{"remove":{"name":"frontend"}}"#);
    }

    #[test]
    fn test_update_operations_with_labels() {
        let ops = UpdateOperations {
            labels: Some(vec![
                LabelOperation::Add("new-label".to_string()),
                LabelOperation::Remove("old-label".to_string()),
            ]),
            ..Default::default()
        };

        let json = serde_json::to_string(&ops).unwrap();
        assert!(json.contains("\"labels\""));
        assert!(json.contains(r#"{"add":"new-label"}"#));
        assert!(json.contains(r#"{"remove":"old-label"}"#));
    }

    // ========================================================================
    // Transition Types tests
    // ========================================================================

    #[test]
    fn test_parse_transitions_response() {
        let json = r#"{
            "transitions": [
                {
                    "id": "11",
                    "name": "Start Progress",
                    "to": {
                        "id": "3",
                        "name": "In Progress"
                    }
                },
                {
                    "id": "21",
                    "name": "Done",
                    "to": {
                        "id": "10",
                        "name": "Done",
                        "statusCategory": {
                            "id": 3,
                            "key": "done",
                            "name": "Done"
                        }
                    }
                }
            ]
        }"#;

        let response: TransitionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.transitions.len(), 2);
        assert_eq!(response.transitions[0].id, "11");
        assert_eq!(response.transitions[0].name, "Start Progress");
        assert_eq!(response.transitions[0].to.name, "In Progress");
        assert_eq!(
            response.transitions[1]
                .to
                .status_category
                .as_ref()
                .unwrap()
                .key,
            "done"
        );
    }

    #[test]
    fn test_parse_transition_with_fields() {
        let json = r#"{
            "id": "31",
            "name": "Resolve Issue",
            "to": {
                "id": "5",
                "name": "Resolved"
            },
            "fields": {
                "resolution": {
                    "required": true,
                    "name": "Resolution"
                }
            }
        }"#;

        let transition: Transition = serde_json::from_str(json).unwrap();
        assert_eq!(transition.id, "31");
        assert!(transition.fields.contains_key("resolution"));
        assert!(transition.fields["resolution"].required);
    }

    #[test]
    fn test_transition_ref_new() {
        let tr = TransitionRef::new("11");
        assert_eq!(tr.id, "11");
    }

    #[test]
    fn test_transition_request_serialization() {
        let request = TransitionRequest {
            transition: TransitionRef::new("11"),
            fields: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(json, r#"{"transition":{"id":"11"}}"#);
    }

    #[test]
    fn test_transition_request_with_fields() {
        let request = TransitionRequest {
            transition: TransitionRef::new("11"),
            fields: Some(FieldUpdates {
                summary: Some("Updated summary".to_string()),
                ..Default::default()
            }),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains(r#""transition":{"id":"11"}"#));
        assert!(json.contains(r#""summary":"Updated summary""#));
    }

    #[test]
    fn test_full_issue_update_request() {
        let update = IssueUpdateRequest {
            fields: Some(FieldUpdates {
                summary: Some("New title".to_string()),
                assignee: Some(NullableUserRef::assign("user123")),
                priority: Some(PriorityRef::new("2")),
                story_points: Some(3.0),
                ..Default::default()
            }),
            update: Some(UpdateOperations {
                labels: Some(vec![LabelOperation::Add("urgent".to_string())]),
                components: Some(vec![ComponentOperation::Add {
                    name: "API".to_string(),
                }]),
            }),
        };

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"summary\":\"New title\""));
        assert!(json.contains("\"accountId\":\"user123\""));
        assert!(json.contains("\"id\":\"2\""));
        assert!(json.contains("\"customfield_10016\":3.0"));
        assert!(json.contains(r#"{"add":"urgent"}"#));
        assert!(json.contains(r#"{"add":{"name":"API"}}"#));
    }

    #[test]
    fn test_nullable_user_ref_assign_serialization() {
        // Assigning to a user should serialize the account ID
        let assign = NullableUserRef::assign("user456");
        let json = serde_json::to_string(&assign).unwrap();
        assert_eq!(json, r#"{"accountId":"user456"}"#);
    }

    #[test]
    fn test_nullable_user_ref_unassign_serialization() {
        // Unassigning should serialize as null
        let unassign = NullableUserRef::unassign();
        let json = serde_json::to_string(&unassign).unwrap();
        assert_eq!(json, "null");
    }

    #[test]
    fn test_unassign_issue_request_serialization() {
        // Full request to unassign should include "assignee": null
        let update = IssueUpdateRequest {
            fields: Some(FieldUpdates {
                assignee: Some(NullableUserRef::unassign()),
                ..Default::default()
            }),
            update: None,
        };

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains(r#""assignee":null"#));
        // The full JSON should be: {"fields":{"assignee":null}}
        assert_eq!(json, r#"{"fields":{"assignee":null}}"#);
    }

    // ========================================================================
    // Changelog Types tests
    // ========================================================================

    #[test]
    fn test_parse_changelog() {
        let json = r#"{
            "startAt": 0,
            "maxResults": 50,
            "total": 2,
            "isLast": true,
            "values": [
                {
                    "id": "10001",
                    "author": {
                        "accountId": "abc123",
                        "displayName": "John Doe",
                        "active": true
                    },
                    "created": "2024-01-15T10:00:00.000+0000",
                    "items": [
                        {
                            "field": "status",
                            "fieldtype": "jira",
                            "from": "1",
                            "fromString": "Open",
                            "to": "2",
                            "toString": "In Progress"
                        }
                    ]
                },
                {
                    "id": "10002",
                    "author": {
                        "accountId": "def456",
                        "displayName": "Jane Smith",
                        "active": true
                    },
                    "created": "2024-01-16T14:30:00.000+0000",
                    "items": [
                        {
                            "field": "assignee",
                            "fieldtype": "jira",
                            "from": null,
                            "fromString": null,
                            "to": "abc123",
                            "toString": "John Doe"
                        }
                    ]
                }
            ]
        }"#;

        let changelog: Changelog = serde_json::from_str(json).unwrap();
        assert_eq!(changelog.start_at, 0);
        assert_eq!(changelog.max_results, 50);
        assert_eq!(changelog.total, 2);
        assert!(changelog.is_last);
        assert_eq!(changelog.histories.len(), 2);
        assert!(!changelog.has_more());
    }

    #[test]
    fn test_changelog_has_more() {
        // First page with more to come
        let changelog = Changelog {
            histories: vec![],
            start_at: 0,
            max_results: 50,
            total: 100,
            is_last: false,
        };
        assert!(changelog.has_more());

        // Last page
        let changelog = Changelog {
            histories: vec![],
            start_at: 50,
            max_results: 50,
            total: 100,
            is_last: true,
        };
        assert!(!changelog.has_more());
    }

    #[test]
    fn test_changelog_next_start() {
        fn create_test_history(id: &str) -> ChangeHistory {
            ChangeHistory {
                id: id.to_string(),
                author: User {
                    account_id: "test".to_string(),
                    display_name: "Test".to_string(),
                    email_address: None,
                    active: true,
                    avatar_urls: None,
                },
                created: "2024-01-15T10:00:00.000+0000".to_string(),
                items: vec![],
            }
        }

        let changelog = Changelog {
            histories: vec![
                create_test_history("1"),
                create_test_history("2"),
                create_test_history("3"),
            ],
            start_at: 0,
            max_results: 50,
            total: 10,
            is_last: false,
        };
        assert_eq!(changelog.next_start(), 3);
    }

    #[test]
    fn test_change_item_display() {
        let item = ChangeItem {
            field: "status".to_string(),
            field_type: Some("jira".to_string()),
            from_value: Some("1".to_string()),
            from_string: Some("Open".to_string()),
            to_value: Some("2".to_string()),
            to_string: Some("In Progress".to_string()),
        };

        assert_eq!(item.display_from(), "Open");
        assert_eq!(item.display_to(), "In Progress");
    }

    #[test]
    fn test_change_item_display_fallback() {
        // When toString/fromString is missing, fall back to to/from
        let item = ChangeItem {
            field: "customfield_123".to_string(),
            field_type: Some("custom".to_string()),
            from_value: Some("old_value".to_string()),
            from_string: None,
            to_value: Some("new_value".to_string()),
            to_string: None,
        };

        assert_eq!(item.display_from(), "old_value");
        assert_eq!(item.display_to(), "new_value");
    }

    #[test]
    fn test_change_item_display_none() {
        // When all values are None
        let item = ChangeItem {
            field: "test".to_string(),
            field_type: None,
            from_value: None,
            from_string: None,
            to_value: None,
            to_string: None,
        };

        assert_eq!(item.display_from(), "(none)");
        assert_eq!(item.display_to(), "(none)");
    }

    #[test]
    fn test_change_type_categorization() {
        let test_cases = vec![
            ("status", ChangeType::Status),
            ("Status", ChangeType::Status),
            ("assignee", ChangeType::Assignee),
            ("priority", ChangeType::Priority),
            ("summary", ChangeType::Content),
            ("description", ChangeType::Content),
            ("labels", ChangeType::Tags),
            ("component", ChangeType::Tags),
            ("sprint", ChangeType::Sprint),
            ("Fix Version", ChangeType::Version),
            ("resolution", ChangeType::Resolution),
            ("Link", ChangeType::Link),
            ("attachment", ChangeType::Attachment),
            ("comment", ChangeType::Comment),
            ("unknownfield", ChangeType::Other),
        ];

        for (field, expected_type) in test_cases {
            let item = ChangeItem {
                field: field.to_string(),
                field_type: None,
                from_value: None,
                from_string: None,
                to_value: None,
                to_string: None,
            };
            assert_eq!(
                item.change_type(),
                expected_type,
                "Field '{}' should be {:?}",
                field,
                expected_type
            );
        }
    }

    #[test]
    fn test_change_type_icons() {
        assert_eq!(ChangeType::Status.icon(), "->");
        assert_eq!(ChangeType::Assignee.icon(), "@");
        assert_eq!(ChangeType::Priority.icon(), "!");
        assert_eq!(ChangeType::Content.icon(), "#");
        assert_eq!(ChangeType::Tags.icon(), "*");
        assert_eq!(ChangeType::Sprint.icon(), "~");
        assert_eq!(ChangeType::Other.icon(), "-");
    }
}
