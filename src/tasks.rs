//! Async task management for non-blocking API operations.
//!
//! This module provides a way to execute async operations in background tasks
//! while keeping the UI responsive. It uses tokio channels to communicate
//! results back to the main event loop.
//!
//! # Architecture
//!
//! The task system follows a simple pattern:
//! 1. The main loop detects a pending operation (e.g., `pending_load_more`)
//! 2. Instead of awaiting inline, it spawns a background task via `TaskSpawner`
//! 3. The main loop continues rendering and handling events
//! 4. When the task completes, it sends an `ApiMessage` through the channel
//! 5. The main loop polls the channel with `try_recv()` and handles results
//!
//! # Adding New Task Types
//!
//! To add a new async operation:
//! 1. Add a variant to `ApiMessage` for the result
//! 2. Add a spawn method to `TaskSpawner`
//! 3. Handle the message in the main event loop

use tokio::sync::mpsc;

use crate::api::types::{
    Changelog, Comment, FieldUpdates, FilterOptions, Issue, IssueLinkType, IssueSuggestion,
    IssueUpdateRequest, Priority, SearchResult, Transition, User,
};
use crate::api::JiraClient;
use crate::config::Profile;

/// Messages sent from background tasks to the main event loop.
///
/// Each variant represents the result of an async operation. The main loop
/// matches on these to update application state appropriately.
#[derive(Debug)]
pub enum ApiMessage {
    /// Initial client connection result
    ClientConnected(Result<JiraClient, String>),

    /// Issue search results (initial fetch or refresh)
    IssuesFetched {
        jql: String,
        result: Result<SearchResult, String>,
        is_background_refresh: bool,
    },

    /// Filter options loaded
    FilterOptionsFetched(Result<FilterOptions, String>),

    /// Pagination load more results
    LoadMoreFetched {
        result: Result<SearchResult, String>,
        offset: u32,
    },

    /// Transitions for an issue
    TransitionsFetched {
        #[allow(dead_code)] // Used for Debug output
        issue_key: String,
        result: Result<Vec<Transition>, String>,
    },

    /// Transition execution result
    TransitionExecuted {
        issue_key: String,
        result: Result<Issue, String>,
    },

    /// Assignable users for a project
    AssigneesFetched {
        result: Result<Vec<User>, String>,
    },

    /// Assignee change result
    AssigneeChanged {
        result: Result<Issue, String>,
    },

    /// Priorities fetched
    PrioritiesFetched(Result<Vec<Priority>, String>),

    /// Priority change result
    PriorityChanged {
        result: Result<Issue, String>,
    },

    /// Comments fetched for an issue
    CommentsFetched {
        result: Result<(Vec<Comment>, u32), String>,
    },

    /// Comment submitted
    CommentSubmitted {
        result: Result<Comment, String>,
    },

    /// Issue update result
    IssueUpdated {
        result: Result<Issue, String>,
    },

    /// Labels fetched
    LabelsFetched(Result<Vec<String>, String>),

    /// Label added/removed result
    LabelChanged {
        result: Result<Issue, String>,
    },

    /// Components fetched for a project
    ComponentsFetched(Result<Vec<String>, String>),

    /// Component added/removed result
    ComponentChanged {
        result: Result<Issue, String>,
    },

    /// Changelog fetched
    ChangelogFetched {
        result: Result<Changelog, String>,
        is_append: bool,
    },

    /// Navigation to linked issue
    LinkedIssueFetched {
        result: Result<Issue, String>,
    },

    /// Link types fetched
    LinkTypesFetched(Result<Vec<IssueLinkType>, String>),

    /// Issue search for linking
    IssueSearchResults {
        result: Result<Vec<IssueSuggestion>, String>,
    },

    /// Link created
    LinkCreated {
        issue_key: String,
        result: Result<(), String>,
    },

    /// Link deleted
    LinkDeleted {
        issue_key: String,
        result: Result<(), String>,
    },
}

/// Spawns background tasks for async operations.
///
/// This struct holds a channel sender and provides methods to spawn
/// various types of async operations. Each method clones the necessary
/// data and spawns a tokio task that sends its result through the channel.
#[derive(Clone)]
pub struct TaskSpawner {
    tx: mpsc::UnboundedSender<ApiMessage>,
}

impl TaskSpawner {
    /// Create a new TaskSpawner with the given channel sender.
    pub fn new(tx: mpsc::UnboundedSender<ApiMessage>) -> Self {
        Self { tx }
    }

    /// Spawn a task to connect to JIRA with the given profile.
    pub fn spawn_connect(&self, profile: Profile) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let result = JiraClient::new(&profile)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::ClientConnected(result));
        });
    }

    /// Spawn a task to fetch issues with the given JQL query.
    pub fn spawn_fetch_issues(
        &self,
        client: &JiraClient,
        jql: String,
        start_at: u32,
        page_size: u32,
        is_background_refresh: bool,
    ) {
        let tx = self.tx.clone();
        let client = client.clone();
        let jql_clone = jql.clone();
        tokio::spawn(async move {
            let result = client
                .search_issues(&jql_clone, start_at, page_size)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::IssuesFetched {
                jql,
                result,
                is_background_refresh,
            });
        });
    }

    /// Spawn a task to fetch filter options.
    pub fn spawn_fetch_filter_options(&self, client: &JiraClient) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = client.get_filter_options().await.map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::FilterOptionsFetched(result));
        });
    }

    /// Spawn a task to load more issues (pagination).
    pub fn spawn_load_more(
        &self,
        client: &JiraClient,
        jql: String,
        offset: u32,
        page_size: u32,
    ) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = client
                .search_issues(&jql, offset, page_size)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::LoadMoreFetched { result, offset });
        });
    }

    /// Spawn a task to fetch transitions for an issue.
    pub fn spawn_fetch_transitions(&self, client: &JiraClient, issue_key: String) {
        let tx = self.tx.clone();
        let client = client.clone();
        let key = issue_key.clone();
        tokio::spawn(async move {
            let result = client.get_transitions(&key).await.map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::TransitionsFetched { issue_key, result });
        });
    }

    /// Spawn a task to execute a transition on an issue.
    pub fn spawn_transition(
        &self,
        client: &JiraClient,
        issue_key: String,
        transition_id: String,
        fields: Option<FieldUpdates>,
    ) {
        let tx = self.tx.clone();
        let client = client.clone();
        let key = issue_key.clone();
        tokio::spawn(async move {
            let result = async {
                client
                    .transition_issue(&key, &transition_id, fields)
                    .await
                    .map_err(|e| e.to_string())?;
                // Fetch updated issue after transition
                client.get_issue(&key).await.map_err(|e| e.to_string())
            }
            .await;
            let _ = tx.send(ApiMessage::TransitionExecuted { issue_key, result });
        });
    }

    /// Spawn a task to fetch assignable users for a project.
    pub fn spawn_fetch_assignees(&self, client: &JiraClient, project_key: String) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = client
                .get_assignable_users(&project_key)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::AssigneesFetched { result });
        });
    }

    /// Spawn a task to change an issue's assignee.
    pub fn spawn_change_assignee(
        &self,
        client: &JiraClient,
        issue_key: String,
        account_id: Option<String>,
    ) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = async {
                client
                    .update_assignee(&issue_key, account_id.as_deref())
                    .await
                    .map_err(|e| e.to_string())?;
                client.get_issue(&issue_key).await.map_err(|e| e.to_string())
            }
            .await;
            let _ = tx.send(ApiMessage::AssigneeChanged { result });
        });
    }

    /// Spawn a task to fetch priorities.
    pub fn spawn_fetch_priorities(&self, client: &JiraClient) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = client.get_priorities().await.map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::PrioritiesFetched(result));
        });
    }

    /// Spawn a task to change an issue's priority.
    pub fn spawn_change_priority(
        &self,
        client: &JiraClient,
        issue_key: String,
        priority_id: String,
    ) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = async {
                client
                    .update_priority(&issue_key, &priority_id)
                    .await
                    .map_err(|e| e.to_string())?;
                client.get_issue(&issue_key).await.map_err(|e| e.to_string())
            }
            .await;
            let _ = tx.send(ApiMessage::PriorityChanged { result });
        });
    }

    /// Spawn a task to fetch comments for an issue.
    pub fn spawn_fetch_comments(&self, client: &JiraClient, issue_key: String) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = client
                .get_comments(&issue_key, 0, 50)
                .await
                .map(|r| (r.comments, r.total))
                .map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::CommentsFetched { result });
        });
    }

    /// Spawn a task to submit a comment to an issue.
    pub fn spawn_submit_comment(&self, client: &JiraClient, issue_key: String, body: String) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = client
                .add_comment(&issue_key, &body)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::CommentSubmitted { result });
        });
    }

    /// Spawn a task to update an issue.
    pub fn spawn_update_issue(
        &self,
        client: &JiraClient,
        issue_key: String,
        update_request: IssueUpdateRequest,
    ) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = async {
                client
                    .update_issue(&issue_key, update_request)
                    .await
                    .map_err(|e| e.to_string())?;
                client.get_issue(&issue_key).await.map_err(|e| e.to_string())
            }
            .await;
            let _ = tx.send(ApiMessage::IssueUpdated { result });
        });
    }

    /// Spawn a task to fetch labels.
    pub fn spawn_fetch_labels(&self, client: &JiraClient) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = client.get_labels().await.map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::LabelsFetched(result));
        });
    }

    /// Spawn a task to add a label to an issue.
    pub fn spawn_add_label(&self, client: &JiraClient, issue_key: String, label: String) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = async {
                client
                    .add_labels(&issue_key, vec![label])
                    .await
                    .map_err(|e| e.to_string())?;
                client.get_issue(&issue_key).await.map_err(|e| e.to_string())
            }
            .await;
            let _ = tx.send(ApiMessage::LabelChanged { result });
        });
    }

    /// Spawn a task to remove a label from an issue.
    pub fn spawn_remove_label(&self, client: &JiraClient, issue_key: String, label: String) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = async {
                client
                    .remove_labels(&issue_key, vec![label])
                    .await
                    .map_err(|e| e.to_string())?;
                client.get_issue(&issue_key).await.map_err(|e| e.to_string())
            }
            .await;
            let _ = tx.send(ApiMessage::LabelChanged { result });
        });
    }

    /// Spawn a task to fetch components for a project.
    pub fn spawn_fetch_components(&self, client: &JiraClient, project_key: String) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = client
                .get_project_components(&project_key)
                .await
                .map(|components| components.into_iter().map(|c| c.name).collect())
                .map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::ComponentsFetched(result));
        });
    }

    /// Spawn a task to add a component to an issue.
    pub fn spawn_add_component(&self, client: &JiraClient, issue_key: String, component: String) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = async {
                client
                    .add_components(&issue_key, vec![component])
                    .await
                    .map_err(|e| e.to_string())?;
                client.get_issue(&issue_key).await.map_err(|e| e.to_string())
            }
            .await;
            let _ = tx.send(ApiMessage::ComponentChanged { result });
        });
    }

    /// Spawn a task to remove a component from an issue.
    pub fn spawn_remove_component(
        &self,
        client: &JiraClient,
        issue_key: String,
        component: String,
    ) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = async {
                client
                    .remove_components(&issue_key, vec![component])
                    .await
                    .map_err(|e| e.to_string())?;
                client.get_issue(&issue_key).await.map_err(|e| e.to_string())
            }
            .await;
            let _ = tx.send(ApiMessage::ComponentChanged { result });
        });
    }

    /// Spawn a task to fetch changelog for an issue.
    pub fn spawn_fetch_changelog(
        &self,
        client: &JiraClient,
        issue_key: String,
        start_at: u32,
        is_append: bool,
    ) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = client
                .get_changelog(&issue_key, start_at, 50)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::ChangelogFetched { result, is_append });
        });
    }

    /// Spawn a task to fetch a linked issue for navigation.
    pub fn spawn_fetch_linked_issue(&self, client: &JiraClient, issue_key: String) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = client.get_issue(&issue_key).await.map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::LinkedIssueFetched { result });
        });
    }

    /// Spawn a task to fetch issue link types.
    pub fn spawn_fetch_link_types(&self, client: &JiraClient) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = client.get_issue_link_types().await.map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::LinkTypesFetched(result));
        });
    }

    /// Spawn a task to search issues for linking.
    pub fn spawn_search_issues_for_link(
        &self,
        client: &JiraClient,
        query: String,
        exclude_key: String,
    ) {
        let tx = self.tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let result = client
                .search_issues_for_picker(&query, Some(&exclude_key))
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::IssueSearchResults { result });
        });
    }

    /// Spawn a task to create an issue link.
    pub fn spawn_create_link(
        &self,
        client: &JiraClient,
        current_key: String,
        target_key: String,
        link_type_name: String,
        is_outward: bool,
    ) {
        let tx = self.tx.clone();
        let client = client.clone();
        let issue_key = current_key.clone();
        tokio::spawn(async move {
            let (outward_key, inward_key) = if is_outward {
                (current_key, target_key)
            } else {
                (target_key, current_key)
            };
            let result = client
                .create_issue_link(&link_type_name, &outward_key, &inward_key)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::LinkCreated { issue_key, result });
        });
    }

    /// Spawn a task to delete an issue link.
    pub fn spawn_delete_link(&self, client: &JiraClient, link_id: String, issue_key: String) {
        let tx = self.tx.clone();
        let client = client.clone();
        let key = issue_key.clone();
        tokio::spawn(async move {
            let result = client
                .delete_issue_link(&link_id)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(ApiMessage::LinkDeleted {
                issue_key: key,
                result,
            });
        });
    }
}

/// Create a new task channel and spawner.
///
/// Returns a tuple of (receiver, spawner). The receiver should be polled
/// in the main event loop, and the spawner should be used to spawn tasks.
pub fn create_task_channel() -> (mpsc::UnboundedReceiver<ApiMessage>, TaskSpawner) {
    let (tx, rx) = mpsc::unbounded_channel();
    (rx, TaskSpawner::new(tx))
}
