//! Filter/search view.

/// The filter view for searching and filtering issues.
pub struct FilterView {
    /// The current filter query.
    pub query: String,
}

impl FilterView {
    /// Create a new filter view.
    pub fn new() -> Self {
        Self {
            query: String::new(),
        }
    }
}

impl Default for FilterView {
    fn default() -> Self {
        Self::new()
    }
}
