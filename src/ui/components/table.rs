//! Reusable table component.

/// A reusable table widget for displaying tabular data.
pub struct Table {
    /// Column headers.
    pub headers: Vec<String>,
    /// Currently selected row.
    pub selected: usize,
}

impl Table {
    /// Create a new table with the given headers.
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            selected: 0,
        }
    }
}
