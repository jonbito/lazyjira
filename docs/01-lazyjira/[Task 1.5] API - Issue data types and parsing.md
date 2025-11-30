# Task 1.5: Issue Data Types and Parsing

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 1.5
**Area:** Backend/API
**Estimated Effort:** M (4-6 hours)

## Description

Define Rust data structures for JIRA issues and related entities. Implement serde deserialization for the JIRA REST API v3 response format with proper handling of optional fields and nested structures.

## Acceptance Criteria

- [x] Issue struct with all required fields (Key, Summary, Status, Assignee, Priority)
- [x] Support for optional fields (Description, Labels, Components, Sprint, etc.)
- [x] User struct for assignee/reporter
- [x] Status, Priority, IssueType enums/structs
- [x] SearchResult struct for paginated responses
- [x] Comment struct for issue comments
- [x] Proper handling of null/missing fields
- [x] Display implementations for TUI rendering

## Implementation Details

### Approach

1. Study JIRA REST API v3 response format
2. Define core structs with serde attributes
3. Handle JIRA's nested field structure
4. Implement Default for optional nested objects
5. Add Display traits for TUI rendering
6. Create conversion methods for internal use

### Files to Modify/Create

- `src/api/types.rs`: All API response types
- `src/api/mod.rs`: Re-export types

### Technical Specifications

**JIRA API Response Structure:**
```json
{
  "key": "PROJ-123",
  "fields": {
    "summary": "Issue title",
    "description": { "type": "doc", "content": [...] },
    "status": { "name": "In Progress", "id": "3" },
    "assignee": { "displayName": "John Doe", "emailAddress": "john@example.com" },
    "priority": { "name": "High", "id": "2" },
    "issuetype": { "name": "Bug", "id": "1" },
    "labels": ["backend", "urgent"],
    "created": "2024-01-15T10:30:00.000+0000",
    "updated": "2024-01-16T14:20:00.000+0000"
  }
}
```

**Rust Types:**
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub key: String,
    #[serde(flatten)]
    pub fields: IssueFields,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueFields {
    pub summary: String,
    pub description: Option<AtlassianDoc>,
    pub status: Status,
    pub assignee: Option<User>,
    pub reporter: Option<User>,
    pub priority: Option<Priority>,
    #[serde(rename = "issuetype")]
    pub issue_type: IssueType,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub components: Vec<Component>,
    pub created: String,
    pub updated: String,
    // Sprint stored in customfield - will handle later
}

#[derive(Debug, Clone, Deserialize)]
pub struct Status {
    pub name: String,
    pub id: String,
    #[serde(rename = "statusCategory")]
    pub category: Option<StatusCategory>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusCategory {
    pub key: String,  // "new", "indeterminate", "done"
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "emailAddress")]
    pub email: Option<String>,
    #[serde(rename = "accountId")]
    pub account_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Priority {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueType {
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub subtask: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Component {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    pub issues: Vec<Issue>,
    #[serde(rename = "startAt")]
    pub start_at: u32,
    #[serde(rename = "maxResults")]
    pub max_results: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Comment {
    pub id: String,
    pub body: AtlassianDoc,
    pub author: User,
    pub created: String,
    pub updated: String,
}

// Atlassian Document Format (ADF) - simplified
#[derive(Debug, Clone, Deserialize)]
pub struct AtlassianDoc {
    #[serde(rename = "type")]
    pub doc_type: String,
    #[serde(default)]
    pub content: Vec<serde_json::Value>,
}

impl AtlassianDoc {
    /// Convert ADF to plain text for display
    pub fn to_plain_text(&self) -> String {
        // Implement ADF to text conversion
        // Extract text nodes recursively
    }
}
```

**Display Implementations:**
```rust
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Issue {
    pub fn assignee_name(&self) -> &str {
        self.fields.assignee
            .as_ref()
            .map(|u| u.display_name.as_str())
            .unwrap_or("Unassigned")
    }

    pub fn priority_name(&self) -> &str {
        self.fields.priority
            .as_ref()
            .map(|p| p.name.as_str())
            .unwrap_or("None")
    }
}
```

## Testing Requirements

- [x] Parse real JIRA API response samples
- [x] Handle issues with missing optional fields
- [x] Handle null assignee/reporter gracefully
- [x] Parse search results with pagination info
- [x] Convert Atlassian Doc to plain text
- [x] Display implementations produce expected output

## Dependencies

- **Prerequisite Tasks:** Task 1.1
- **Blocks Tasks:** Task 1.6, Task 1.7
- **External:** serde, serde_json

## Definition of Done

- [x] All acceptance criteria met
- [x] Types handle all common JIRA field combinations
- [x] Unit tests with sample API responses
- [x] Documentation for all public types
- [x] Edge cases handled (null, empty arrays, etc.)

## Completion Summary

**Completed:** 2024-11-29

### Files Modified

- `src/api/types.rs` - Added Comment, CommentsResponse, and AtlassianDoc structs; added Display implementations for Issue, Status, Priority, IssueType, User; added convenience methods (assignee_name, priority_name, description_text, project_key); added 24 comprehensive tests
- `src/api/mod.rs` - Updated re-exports to include all new types

### Key Implementation Decisions

1. **AtlassianDoc.to_plain_text()**: Implemented recursive text extraction supporting paragraphs, headings, bullet lists, ordered lists, code blocks, blockquotes, mentions, emojis, and hard breaks
2. **Description handling**: IssueFields.description remains as `Option<serde_json::Value>` to handle both ADF and legacy string formats; Issue.description_text() method handles conversion
3. **Display traits**: Added fmt::Display for Issue, Status, Priority, IssueType, and User for TUI rendering
4. **Convenience methods**: Issue has assignee_name(), priority_name(), description_text(), and project_key() for safe access with sensible defaults

### Test Coverage

- 24 unit tests covering:
  - Parsing minimal and full JIRA API responses
  - Handling null/missing optional fields
  - AtlassianDoc to plain text conversion (paragraphs, lists, headings, code blocks, mentions, hard breaks)
  - Display trait implementations
  - SearchResult and CommentsResponse pagination
  - Issue convenience methods
