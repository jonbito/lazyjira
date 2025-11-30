//! Profile management view.

/// The profile management view.
pub struct ProfileView {
    /// Currently selected profile index.
    pub selected: usize,
}

impl ProfileView {
    /// Create a new profile view.
    pub fn new() -> Self {
        Self { selected: 0 }
    }
}

impl Default for ProfileView {
    fn default() -> Self {
        Self::new()
    }
}
