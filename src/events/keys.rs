//! Key binding definitions.

/// Key binding configuration.
pub struct KeyBindings {
    /// Whether vim-style bindings are enabled.
    pub vim_mode: bool,
}

impl KeyBindings {
    /// Create new key bindings.
    pub fn new(vim_mode: bool) -> Self {
        Self { vim_mode }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::new(true)
    }
}
