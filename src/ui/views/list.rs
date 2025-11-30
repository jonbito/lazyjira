//! Issue list view.

/// The issue list view.
pub struct ListView {
    /// Currently selected index.
    pub selected: usize,
}

impl ListView {
    /// Create a new list view.
    pub fn new() -> Self {
        Self { selected: 0 }
    }
}

impl Default for ListView {
    fn default() -> Self {
        Self::new()
    }
}
