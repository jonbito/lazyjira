//! Issue detail view.

/// The issue detail view.
pub struct DetailView {
    /// The issue key being displayed.
    pub issue_key: Option<String>,
}

impl DetailView {
    /// Create a new detail view.
    pub fn new() -> Self {
        Self { issue_key: None }
    }
}

impl Default for DetailView {
    fn default() -> Self {
        Self::new()
    }
}
