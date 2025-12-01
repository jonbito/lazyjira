//! Key binding definitions and registry.
//!
//! Provides a centralized registry of all keyboard shortcuts organized by context.

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

/// The context in which a keybinding is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyContext {
    /// Global keybindings available in all views.
    Global,
    /// Keybindings for the issue list view.
    IssueList,
    /// Keybindings for the issue detail view.
    IssueDetail,
    /// Keybindings for profile management.
    ProfileManagement,
    /// Keybindings for the filter panel.
    FilterPanel,
    /// Keybindings for text editor/input mode.
    Editor,
    /// Keybindings for JQL input.
    JqlInput,
}

impl KeyContext {
    /// Get the display name for this context.
    pub fn display(&self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::IssueList => "Issue List",
            Self::IssueDetail => "Issue Detail",
            Self::ProfileManagement => "Profile Management",
            Self::FilterPanel => "Filter Panel",
            Self::Editor => "Editor",
            Self::JqlInput => "JQL Input",
        }
    }
}

/// A single keybinding definition.
#[derive(Debug, Clone)]
pub struct Keybinding {
    /// The key or key combination (e.g., "?", "Ctrl+C", "j / ↓").
    pub key: String,
    /// Short action name for internal use.
    pub action: String,
    /// Human-readable description of what the key does.
    pub description: String,
    /// The context in which this keybinding is active.
    pub context: KeyContext,
}

impl Keybinding {
    /// Create a new keybinding.
    pub fn new(key: &str, action: &str, description: &str, context: KeyContext) -> Self {
        Self {
            key: key.to_string(),
            action: action.to_string(),
            description: description.to_string(),
            context,
        }
    }
}

/// Get all keybindings organized by context.
pub fn get_keybindings() -> Vec<Keybinding> {
    vec![
        // Global keybindings
        Keybinding::new("?", "help", "Show this help panel", KeyContext::Global),
        Keybinding::new("Ctrl+C", "quit", "Quit application", KeyContext::Global),
        Keybinding::new(
            "p",
            "switch_profile",
            "Switch JIRA profile (quick)",
            KeyContext::Global,
        ),
        Keybinding::new(
            "P",
            "manage_profiles",
            "Manage profiles (CRUD)",
            KeyContext::Global,
        ),
        Keybinding::new("r", "refresh", "Refresh current view", KeyContext::Global),
        Keybinding::new(
            "Ctrl+P / Ctrl+K",
            "command_palette",
            "Open command palette",
            KeyContext::Global,
        ),
        // Issue List keybindings
        Keybinding::new("j / ↓", "move_down", "Move down", KeyContext::IssueList),
        Keybinding::new("k / ↑", "move_up", "Move up", KeyContext::IssueList),
        Keybinding::new("gg", "go_top", "Go to first issue", KeyContext::IssueList),
        Keybinding::new("G", "go_bottom", "Go to last issue", KeyContext::IssueList),
        Keybinding::new("Ctrl+d", "page_down", "Page down", KeyContext::IssueList),
        Keybinding::new("Ctrl+u", "page_up", "Page up", KeyContext::IssueList),
        Keybinding::new(
            "Enter",
            "open_issue",
            "Open issue details",
            KeyContext::IssueList,
        ),
        Keybinding::new("f", "filter", "Open filter panel", KeyContext::IssueList),
        Keybinding::new(
            ": / /",
            "jql",
            "Open JQL query input",
            KeyContext::IssueList,
        ),
        Keybinding::new("q", "quit", "Quit application", KeyContext::IssueList),
        // Issue Detail keybindings
        Keybinding::new(
            "j / ↓",
            "scroll_down",
            "Scroll down",
            KeyContext::IssueDetail,
        ),
        Keybinding::new("k / ↑", "scroll_up", "Scroll up", KeyContext::IssueDetail),
        Keybinding::new("g", "go_top", "Go to top", KeyContext::IssueDetail),
        Keybinding::new("G", "go_bottom", "Go to bottom", KeyContext::IssueDetail),
        Keybinding::new("Ctrl+d", "page_down", "Page down", KeyContext::IssueDetail),
        Keybinding::new("Ctrl+u", "page_up", "Page up", KeyContext::IssueDetail),
        Keybinding::new("e", "edit", "Edit issue", KeyContext::IssueDetail),
        Keybinding::new("s", "status", "Change status", KeyContext::IssueDetail),
        Keybinding::new("c", "comment", "Add comment", KeyContext::IssueDetail),
        Keybinding::new("a", "assign", "Change assignee", KeyContext::IssueDetail),
        Keybinding::new("y", "priority", "Change priority", KeyContext::IssueDetail),
        Keybinding::new("l", "labels", "Edit labels", KeyContext::IssueDetail),
        Keybinding::new(
            "o",
            "components",
            "Edit components",
            KeyContext::IssueDetail,
        ),
        Keybinding::new("L", "link", "Link issue", KeyContext::IssueDetail),
        Keybinding::new(
            "q / Esc",
            "back",
            "Go back to list",
            KeyContext::IssueDetail,
        ),
        // JQL Input keybindings
        Keybinding::new("Enter", "execute", "Execute query", KeyContext::JqlInput),
        Keybinding::new("↑ / ↓", "history", "Browse history", KeyContext::JqlInput),
        Keybinding::new("Esc", "cancel", "Cancel", KeyContext::JqlInput),
        // Profile Management keybindings
        Keybinding::new("a", "add", "Add new profile", KeyContext::ProfileManagement),
        Keybinding::new(
            "e",
            "edit",
            "Edit selected profile",
            KeyContext::ProfileManagement,
        ),
        Keybinding::new(
            "d",
            "delete",
            "Delete selected profile",
            KeyContext::ProfileManagement,
        ),
        Keybinding::new(
            "s",
            "set_default",
            "Set as default profile",
            KeyContext::ProfileManagement,
        ),
        Keybinding::new(
            "Space",
            "switch",
            "Switch to profile",
            KeyContext::ProfileManagement,
        ),
        Keybinding::new("q / Esc", "back", "Go back", KeyContext::ProfileManagement),
        // Filter Panel keybindings
        Keybinding::new(
            "Tab / ← / →",
            "switch_section",
            "Switch section",
            KeyContext::FilterPanel,
        ),
        Keybinding::new(
            "↑ / ↓",
            "navigate",
            "Navigate in section",
            KeyContext::FilterPanel,
        ),
        Keybinding::new(
            "Space",
            "toggle",
            "Toggle selection",
            KeyContext::FilterPanel,
        ),
        Keybinding::new("c", "clear", "Clear all filters", KeyContext::FilterPanel),
        Keybinding::new("Enter", "apply", "Apply filters", KeyContext::FilterPanel),
        Keybinding::new("Esc", "cancel", "Cancel", KeyContext::FilterPanel),
        // Editor keybindings
        Keybinding::new("Ctrl+S", "save", "Save changes", KeyContext::Editor),
        Keybinding::new("Esc", "cancel", "Cancel editing", KeyContext::Editor),
    ]
}

/// Get keybindings filtered by context.
pub fn get_keybindings_for_context(context: KeyContext) -> Vec<Keybinding> {
    get_keybindings()
        .into_iter()
        .filter(|b| b.context == context)
        .collect()
}

/// Get keybindings grouped by context.
pub fn get_keybindings_grouped() -> Vec<(KeyContext, Vec<Keybinding>)> {
    let contexts = [
        KeyContext::Global,
        KeyContext::IssueList,
        KeyContext::IssueDetail,
        KeyContext::JqlInput,
        KeyContext::ProfileManagement,
        KeyContext::FilterPanel,
        KeyContext::Editor,
    ];

    let all_bindings = get_keybindings();

    contexts
        .iter()
        .filter_map(|ctx| {
            let bindings: Vec<Keybinding> = all_bindings
                .iter()
                .filter(|b| &b.context == ctx)
                .cloned()
                .collect();
            if bindings.is_empty() {
                None
            } else {
                Some((*ctx, bindings))
            }
        })
        .collect()
}

/// Get contextual hint text for a given context.
pub fn get_context_hints(context: KeyContext) -> &'static str {
    match context {
        KeyContext::Global => "[?] help",
        KeyContext::IssueList => "[j/k] navigate  [Enter] open  [f] filter  [/] search  [?] help",
        KeyContext::IssueDetail => {
            "[e] edit  [c] comment  [s] status  [a] assign  [q] back  [?] help"
        }
        KeyContext::ProfileManagement => "[a] add  [e] edit  [d] delete  [s] default  [q] back",
        KeyContext::FilterPanel => {
            "[Space] toggle  [Tab] section  [Enter] apply  [c] clear  [Esc] close"
        }
        KeyContext::JqlInput => "[Enter] execute  [↑/↓] history  [Esc] cancel",
        KeyContext::Editor => "[Ctrl+S] save  [Esc] cancel",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybindings_new() {
        let bindings = KeyBindings::new(true);
        assert!(bindings.vim_mode);
    }

    #[test]
    fn test_keybindings_default() {
        let bindings = KeyBindings::default();
        assert!(bindings.vim_mode);
    }

    #[test]
    fn test_key_context_display() {
        assert_eq!(KeyContext::Global.display(), "Global");
        assert_eq!(KeyContext::IssueList.display(), "Issue List");
        assert_eq!(KeyContext::IssueDetail.display(), "Issue Detail");
        assert_eq!(
            KeyContext::ProfileManagement.display(),
            "Profile Management"
        );
        assert_eq!(KeyContext::FilterPanel.display(), "Filter Panel");
        assert_eq!(KeyContext::Editor.display(), "Editor");
        assert_eq!(KeyContext::JqlInput.display(), "JQL Input");
    }

    #[test]
    fn test_get_keybindings_not_empty() {
        let bindings = get_keybindings();
        assert!(!bindings.is_empty());
    }

    #[test]
    fn test_get_keybindings_has_help() {
        let bindings = get_keybindings();
        let help_binding = bindings.iter().find(|b| b.action == "help");
        assert!(help_binding.is_some());
        let help = help_binding.unwrap();
        assert_eq!(help.key, "?");
        assert_eq!(help.context, KeyContext::Global);
    }

    #[test]
    fn test_get_keybindings_for_context() {
        let global_bindings = get_keybindings_for_context(KeyContext::Global);
        assert!(!global_bindings.is_empty());
        assert!(global_bindings
            .iter()
            .all(|b| b.context == KeyContext::Global));
    }

    #[test]
    fn test_get_keybindings_grouped() {
        let grouped = get_keybindings_grouped();
        assert!(!grouped.is_empty());

        // Check that Global context is present
        let global = grouped.iter().find(|(ctx, _)| *ctx == KeyContext::Global);
        assert!(global.is_some());
    }

    #[test]
    fn test_keybinding_new() {
        let binding = Keybinding::new("?", "help", "Show help", KeyContext::Global);
        assert_eq!(binding.key, "?");
        assert_eq!(binding.action, "help");
        assert_eq!(binding.description, "Show help");
        assert_eq!(binding.context, KeyContext::Global);
    }

    #[test]
    fn test_get_context_hints() {
        let hints = get_context_hints(KeyContext::IssueList);
        assert!(hints.contains("navigate"));
        assert!(hints.contains("help"));
    }
}
