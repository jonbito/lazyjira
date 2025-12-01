# Task 4.8: Create and Manage Issue Links

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 4.8
**Area:** Frontend/UI + API
**Estimated Effort:** L (6-10 hours)

## Description

Add the ability to create, view, and delete issue links directly from the issue detail view. Users should be able to link the current issue to other issues with various relationship types (blocks, is blocked by, relates to, duplicates, etc.).

## Acceptance Criteria

- [ ] Add new link from issue detail view (keybinding: `L`)
- [ ] Fetch available link types from JIRA API
- [ ] Search for target issue by key or text
- [ ] Select link type from picker (blocks, relates to, duplicates, etc.)
- [ ] Create link via API
- [ ] Delete existing link from linked issues section (keybinding: `d` when focused)
- [ ] Confirmation dialog before deleting links
- [ ] Show success/error notifications
- [ ] Refresh linked issues section after changes

## Implementation Details

### Approach

1. Add API methods for link types, creating links, and deleting links
2. Create link type picker component
3. Create issue search/selector component for target issue
4. Add create link workflow in detail view
5. Add delete link functionality in linked issues section
6. Handle async operations with loading states

### Files to Modify/Create

- `src/api/client.rs`: Add link management API methods
- `src/api/types.rs`: Add link creation request types
- `src/ui/components/link_picker.rs`: New - Link type picker component
- `src/ui/components/issue_search.rs`: New - Issue search/selector component
- `src/ui/components/linked_issues.rs`: Add delete functionality
- `src/ui/views/detail.rs`: Add create link workflow
- `src/app.rs`: Add pending link operations

### Technical Specifications

**API Endpoints:**

```
GET /rest/api/3/issueLinkType - Get available link types
POST /rest/api/3/issueLink - Create a link
DELETE /rest/api/3/issueLink/{linkId} - Delete a link
GET /rest/api/3/issue/picker - Search for issues
```

**New Types:**

```rust
/// Request to create an issue link.
#[derive(Debug, Clone, Serialize)]
pub struct CreateIssueLinkRequest {
    /// The type of link.
    #[serde(rename = "type")]
    pub link_type: IssueLinkTypeRef,
    /// The inward issue (affected by this issue).
    #[serde(rename = "inwardIssue")]
    pub inward_issue: IssueKeyRef,
    /// The outward issue (this issue affects).
    #[serde(rename = "outwardIssue")]
    pub outward_issue: IssueKeyRef,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssueLinkTypeRef {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssueKeyRef {
    pub key: String,
}

/// Response from issue link types endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct IssueLinkTypesResponse {
    #[serde(rename = "issueLinkTypes")]
    pub issue_link_types: Vec<IssueLinkType>,
}

/// Issue picker suggestion.
#[derive(Debug, Clone, Deserialize)]
pub struct IssueSuggestion {
    pub key: String,
    pub summary: String,
    #[serde(rename = "summaryText")]
    pub summary_text: Option<String>,
}

/// Issue picker response.
#[derive(Debug, Clone, Deserialize)]
pub struct IssuePickerResponse {
    pub sections: Vec<IssuePickerSection>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssuePickerSection {
    pub label: String,
    pub issues: Vec<IssueSuggestion>,
}
```

**API Client Methods:**

```rust
impl JiraClient {
    /// Get available issue link types.
    pub async fn get_issue_link_types(&self) -> Result<Vec<IssueLinkType>> {
        let url = format!("{}/rest/api/3/issueLinkType", self.base_url);
        let response: IssueLinkTypesResponse = self.client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?
            .json()
            .await?;
        Ok(response.issue_link_types)
    }

    /// Create a link between two issues.
    pub async fn create_issue_link(
        &self,
        link_type: &str,
        inward_issue_key: &str,
        outward_issue_key: &str,
    ) -> Result<()> {
        let url = format!("{}/rest/api/3/issueLink", self.base_url);
        let request = CreateIssueLinkRequest {
            link_type: IssueLinkTypeRef { name: link_type.to_string() },
            inward_issue: IssueKeyRef { key: inward_issue_key.to_string() },
            outward_issue: IssueKeyRef { key: outward_issue_key.to_string() },
        };
        self.client
            .post(&url)
            .headers(self.auth_headers())
            .json(&request)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Delete an issue link.
    pub async fn delete_issue_link(&self, link_id: &str) -> Result<()> {
        let url = format!("{}/rest/api/3/issueLink/{}", self.base_url, link_id);
        self.client
            .delete(&url)
            .headers(self.auth_headers())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Search for issues (for issue picker).
    pub async fn search_issues_for_picker(
        &self,
        query: &str,
        current_issue_key: Option<&str>,
    ) -> Result<Vec<IssueSuggestion>> {
        let mut url = format!(
            "{}/rest/api/3/issue/picker?query={}",
            self.base_url,
            urlencoding::encode(query)
        );
        if let Some(key) = current_issue_key {
            url.push_str(&format!("&currentIssueKey={}", key));
        }
        let response: IssuePickerResponse = self.client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?
            .json()
            .await?;

        // Flatten all sections into a single list
        Ok(response.sections
            .into_iter()
            .flat_map(|s| s.issues)
            .collect())
    }
}
```

**Link Picker Component:**

```rust
pub struct LinkPicker {
    link_types: Vec<IssueLinkType>,
    selected: usize,
    visible: bool,
    loading: bool,
}

pub enum LinkPickerAction {
    Select(IssueLinkType),
    Cancel,
}

impl LinkPicker {
    pub fn new() -> Self { ... }
    pub fn show(&mut self) { ... }
    pub fn hide(&mut self) { ... }
    pub fn set_link_types(&mut self, types: Vec<IssueLinkType>) { ... }
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<LinkPickerAction> { ... }
    pub fn render(&self, frame: &mut Frame, area: Rect) { ... }
}
```

**Issue Search Component:**

```rust
pub struct IssueSearch {
    query: String,
    suggestions: Vec<IssueSuggestion>,
    selected: usize,
    visible: bool,
    loading: bool,
    debounce_timer: Option<Instant>,
}

pub enum IssueSearchAction {
    Select(String), // Issue key
    Cancel,
    Search(String), // Trigger search with query
}

impl IssueSearch {
    pub fn new() -> Self { ... }
    pub fn show(&mut self) { ... }
    pub fn hide(&mut self) { ... }
    pub fn set_suggestions(&mut self, suggestions: Vec<IssueSuggestion>) { ... }
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<IssueSearchAction> { ... }
    pub fn render(&self, frame: &mut Frame, area: Rect) { ... }
}
```

**Create Link Workflow:**

1. User presses `L` in detail view
2. Link type picker opens → User selects link type
3. Issue search opens → User searches and selects target issue
4. Confirmation shown → User confirms
5. API call to create link
6. Success notification, refresh linked issues section

**Delete Link Workflow:**

1. User focuses linked issues section with `r`
2. User navigates to link to delete
3. User presses `d`
4. Confirmation dialog shown
5. API call to delete link
6. Success notification, refresh linked issues section

### Key Bindings

| Key | Context | Action |
|-----|---------|--------|
| `L` | Detail view | Open create link workflow |
| `d` | Linked issues focused | Delete selected link |
| `Enter` | Link picker | Select link type |
| `Enter` | Issue search | Select issue |
| `Esc` | Any picker | Cancel |
| `j`/`k` | Pickers | Navigate |

## Testing Requirements

- [ ] Link types fetched and displayed correctly
- [ ] Issue search returns relevant results
- [ ] Link creation succeeds with valid data
- [ ] Link deletion works with confirmation
- [ ] Error handling for API failures
- [ ] Loading states shown during async operations
- [ ] Linked issues section refreshes after changes
- [ ] Keyboard navigation works in all pickers

## Dependencies

- **Prerequisite Tasks:** Task 4.5 (Linked issues display)
- **Blocks Tasks:** None
- **External:** JIRA issue link API, issue picker API

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Create and delete links work correctly
- [ ] Error states handled gracefully
- [ ] Loading indicators shown
- [ ] Unit tests for new components
- [ ] Integration with existing linked issues section
