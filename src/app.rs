//! Main application state and event loop.

/// The main application struct that holds all state.
pub struct App {
    /// Whether the application should quit.
    pub should_quit: bool,
}

impl App {
    /// Create a new application instance.
    pub fn new() -> Self {
        Self { should_quit: false }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
