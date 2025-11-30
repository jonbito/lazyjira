//! Modal dialog component.

/// A modal dialog widget.
pub struct Modal {
    /// The modal title.
    pub title: String,
    /// Whether the modal is visible.
    pub visible: bool,
}

impl Modal {
    /// Create a new modal with the given title.
    pub fn new(title: String) -> Self {
        Self {
            title,
            visible: false,
        }
    }
}
