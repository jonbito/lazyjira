//! Event handler implementation.

/// Handles application events.
pub struct EventHandler;

impl EventHandler {
    /// Create a new event handler.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
