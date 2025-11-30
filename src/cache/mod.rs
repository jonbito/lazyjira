//! Issue caching for offline viewing.
//!
//! This module provides caching functionality to store issue data locally
//! for offline access and reduced API calls.

use std::collections::HashMap;

use crate::api::Issue;

/// Cache for storing JIRA issues.
pub struct Cache {
    /// Cached issues by key.
    issues: HashMap<String, Issue>,
}

impl Cache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self {
            issues: HashMap::new(),
        }
    }

    /// Get an issue from the cache.
    pub fn get(&self, key: &str) -> Option<&Issue> {
        self.issues.get(key)
    }

    /// Store an issue in the cache.
    pub fn insert(&mut self, issue: Issue) {
        self.issues.insert(issue.key.clone(), issue);
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
