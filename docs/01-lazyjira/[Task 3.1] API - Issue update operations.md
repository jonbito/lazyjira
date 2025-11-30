# Task 3.1: Issue Update Operations

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 3.1
**Area:** Backend/API
**Estimated Effort:** M (6-8 hours)

## Description

Implement JIRA API operations for updating issues including summary, description, status transitions, assignee, priority, labels, and other fields. This provides the backend foundation for the editing UI.

## Acceptance Criteria

- [ ] Update issue summary
- [ ] Update issue description (ADF format)
- [ ] Change issue status (workflow transitions)
- [ ] Change assignee with user search
- [ ] Update priority
- [ ] Add/remove labels
- [ ] Add/remove components
- [ ] Update sprint assignment
- [ ] Update story points/estimates
- [ ] Get available transitions for an issue
- [ ] Proper error handling for conflicts/permissions

## Implementation Details

### Approach

1. Study JIRA REST API v3 update endpoints
2. Implement generic issue update method
3. Add specific methods for common operations
4. Implement transitions API
5. Handle optimistic locking/conflicts
6. Add user search for assignee changes

### Files to Modify/Create

- `src/api/client.rs`: Update operations
- `src/api/types.rs`: Update request types
- `src/api/transitions.rs`: Workflow transition types

### Technical Specifications

**Issue Update Request:**
```rust
#[derive(Debug, Serialize)]
pub struct IssueUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<FieldUpdates>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<UpdateOperations>,
}

#[derive(Debug, Serialize, Default)]
pub struct FieldUpdates {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<AtlassianDoc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<UserRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<PriorityRef>,
    #[serde(rename = "customfield_10016", skip_serializing_if = "Option::is_none")]
    pub story_points: Option<f32>,
    // Sprint is typically customfield_10020
    #[serde(rename = "customfield_10020", skip_serializing_if = "Option::is_none")]
    pub sprint: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct UserRef {
    #[serde(rename = "accountId")]
    pub account_id: String,
}

#[derive(Debug, Serialize)]
pub struct PriorityRef {
    pub id: String,
}

#[derive(Debug, Serialize, Default)]
pub struct UpdateOperations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<LabelOperation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<ComponentOperation>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LabelOperation {
    Add(String),
    Remove(String),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentOperation {
    Add { name: String },
    Remove { name: String },
}
```

**Transition Types:**
```rust
#[derive(Debug, Deserialize)]
pub struct TransitionsResponse {
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Transition {
    pub id: String,
    pub name: String,
    pub to: TransitionTarget,
    #[serde(default)]
    pub fields: HashMap<String, TransitionField>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransitionTarget {
    pub id: String,
    pub name: String,
    #[serde(rename = "statusCategory")]
    pub category: StatusCategory,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransitionField {
    pub required: bool,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct TransitionRequest {
    pub transition: TransitionRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<FieldUpdates>,
}

#[derive(Debug, Serialize)]
pub struct TransitionRef {
    pub id: String,
}
```

**API Client Methods:**
```rust
impl JiraClient {
    /// Update issue fields
    #[tracing::instrument(skip(self, update))]
    pub async fn update_issue(&self, key: &str, update: IssueUpdateRequest) -> Result<()> {
        let url = format!("{}/rest/api/3/issue/{}", self.base_url, key);

        let response = self.client
            .put(&url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "application/json")
            .json(&update)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Updated issue {}", key);
            Ok(())
        } else {
            let error: JiraError = response.json().await?;
            Err(ApiError::UpdateFailed(error.message()))
        }
    }

    /// Get available transitions for an issue
    pub async fn get_transitions(&self, key: &str) -> Result<Vec<Transition>> {
        let url = format!("{}/rest/api/3/issue/{}/transitions", self.base_url, key);
        let response: TransitionsResponse = self.get(&url).await?;
        Ok(response.transitions)
    }

    /// Perform a status transition
    #[tracing::instrument(skip(self))]
    pub async fn transition_issue(
        &self,
        key: &str,
        transition_id: &str,
        fields: Option<FieldUpdates>,
    ) -> Result<()> {
        let url = format!("{}/rest/api/3/issue/{}/transitions", self.base_url, key);

        let request = TransitionRequest {
            transition: TransitionRef { id: transition_id.to_string() },
            fields,
        };

        let response = self.client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Transitioned issue {} via {}", key, transition_id);
            Ok(())
        } else {
            let error: JiraError = response.json().await?;
            Err(ApiError::TransitionFailed(error.message()))
        }
    }

    /// Search for users (for assignee selection)
    pub async fn search_users(&self, query: &str) -> Result<Vec<User>> {
        let url = format!(
            "{}/rest/api/3/user/search?query={}",
            self.base_url,
            urlencoding::encode(query)
        );
        self.get(&url).await
    }

    /// Get available priorities
    pub async fn get_priorities(&self) -> Result<Vec<Priority>> {
        let url = format!("{}/rest/api/3/priority", self.base_url);
        self.get(&url).await
    }

    // Convenience methods for common updates
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

    pub async fn update_assignee(&self, key: &str, account_id: &str) -> Result<()> {
        let update = IssueUpdateRequest {
            fields: Some(FieldUpdates {
                assignee: Some(UserRef { account_id: account_id.to_string() }),
                ..Default::default()
            }),
            update: None,
        };
        self.update_issue(key, update).await
    }

    pub async fn add_labels(&self, key: &str, labels: Vec<String>) -> Result<()> {
        let operations: Vec<LabelOperation> = labels
            .into_iter()
            .map(LabelOperation::Add)
            .collect();

        let update = IssueUpdateRequest {
            fields: None,
            update: Some(UpdateOperations {
                labels: Some(operations),
                ..Default::default()
            }),
        };
        self.update_issue(key, update).await
    }

    pub async fn remove_labels(&self, key: &str, labels: Vec<String>) -> Result<()> {
        let operations: Vec<LabelOperation> = labels
            .into_iter()
            .map(LabelOperation::Remove)
            .collect();

        let update = IssueUpdateRequest {
            fields: None,
            update: Some(UpdateOperations {
                labels: Some(operations),
                ..Default::default()
            }),
        };
        self.update_issue(key, update).await
    }
}
```

**Error Handling:**
```rust
#[derive(Debug, Deserialize)]
struct JiraError {
    #[serde(rename = "errorMessages")]
    error_messages: Vec<String>,
    errors: HashMap<String, String>,
}

impl JiraError {
    fn message(&self) -> String {
        if !self.error_messages.is_empty() {
            self.error_messages.join(", ")
        } else {
            self.errors.values().cloned().collect::<Vec<_>>().join(", ")
        }
    }
}

#[derive(Debug, Error)]
pub enum ApiError {
    // ... existing errors ...

    #[error("Failed to update issue: {0}")]
    UpdateFailed(String),

    #[error("Failed to transition issue: {0}")]
    TransitionFailed(String),

    #[error("Conflict: issue was modified by another user")]
    Conflict,

    #[error("You don't have permission to modify this issue")]
    PermissionDenied,
}
```

## Testing Requirements

- [ ] Update summary works
- [ ] Update description works (with ADF)
- [ ] Get transitions returns available options
- [ ] Transition changes issue status
- [ ] Assign to user works
- [ ] Add/remove labels works
- [ ] Invalid transition returns error
- [ ] Permission denied handled correctly
- [ ] User search returns results

## Dependencies

- **Prerequisite Tasks:** Task 1.4, Task 1.5
- **Blocks Tasks:** Task 3.2, 3.3, 3.4, 3.5
- **External:** JIRA REST API v3

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Error messages are user-friendly
- [ ] Updates invalidate cache
- [ ] All common update scenarios covered
- [ ] Integration tests with mock server
