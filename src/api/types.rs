//! JIRA API response types.

use serde::{Deserialize, Serialize};

/// A JIRA issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// The issue key (e.g., "PROJ-123").
    pub key: String,
    /// The issue summary.
    pub summary: String,
    /// The issue status.
    pub status: String,
}
