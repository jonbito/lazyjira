//! Command registry for the command palette.
//!
//! Provides command definitions, fuzzy search, and recent command tracking.

// Command registry API items are provided for external use
#![allow(dead_code)]

use std::collections::VecDeque;

/// A command that can be executed from the command palette.
#[derive(Debug, Clone)]
pub struct Command {
    /// Unique identifier for the command.
    pub id: String,
    /// Display title shown in the palette.
    pub title: String,
    /// Optional description for additional context.
    pub description: Option<String>,
    /// Category for organization.
    pub category: CommandCategory,
    /// Additional keywords for search.
    pub keywords: Vec<String>,
    /// Keyboard shortcut hint (if any).
    pub shortcut: Option<String>,
    /// The action to perform when executed.
    pub action: CommandAction,
}

/// Categories for organizing commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    /// Navigation commands (go to views).
    Navigation,
    /// Issue-related commands.
    Issue,
    /// Profile management commands.
    Profile,
    /// Filter and search commands.
    Filter,
    /// Application settings.
    Settings,
    /// Help and documentation.
    Help,
}

impl CommandCategory {
    /// Get the display name for this category.
    pub fn display(&self) -> &'static str {
        match self {
            Self::Navigation => "Navigation",
            Self::Issue => "Issue",
            Self::Profile => "Profile",
            Self::Filter => "Filter",
            Self::Settings => "Settings",
            Self::Help => "Help",
        }
    }
}

/// Actions that can be triggered by commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandAction {
    /// Navigate to the issue list view.
    GoToList,
    /// Navigate to profile management.
    GoToProfiles,
    /// Navigate to filter panel.
    GoToFilters,
    /// Show help screen.
    GoToHelp,
    /// Refresh the current issue list.
    RefreshIssues,
    /// Open profile switcher.
    SwitchProfile,
    /// Open JQL input.
    OpenJqlInput,
    /// Clear all active filters.
    ClearFilters,
    /// Clear the issue cache.
    ClearCache,
}

/// Registry of all available commands with search and history.
pub struct CommandRegistry {
    /// All registered commands.
    commands: Vec<Command>,
    /// Recently used command IDs (most recent first).
    recent: VecDeque<String>,
}

impl CommandRegistry {
    /// Maximum number of recent commands to track.
    pub const MAX_RECENT: usize = 10;

    /// Create a new command registry with default commands.
    pub fn new() -> Self {
        let commands = vec![
            Command {
                id: "goto.list".to_string(),
                title: "Go to Issue List".to_string(),
                description: Some("Navigate to the issue list view".to_string()),
                category: CommandCategory::Navigation,
                keywords: vec!["issues".to_string(), "home".to_string(), "back".to_string()],
                shortcut: Some("Esc".to_string()),
                action: CommandAction::GoToList,
            },
            Command {
                id: "goto.profiles".to_string(),
                title: "Manage Profiles".to_string(),
                description: Some("View and edit JIRA profiles".to_string()),
                category: CommandCategory::Profile,
                keywords: vec![
                    "account".to_string(),
                    "connection".to_string(),
                    "jira".to_string(),
                ],
                shortcut: Some("P".to_string()),
                action: CommandAction::GoToProfiles,
            },
            Command {
                id: "profile.switch".to_string(),
                title: "Switch Profile".to_string(),
                description: Some("Quick switch to another profile".to_string()),
                category: CommandCategory::Profile,
                keywords: vec![
                    "change".to_string(),
                    "account".to_string(),
                    "select".to_string(),
                ],
                shortcut: Some("p".to_string()),
                action: CommandAction::SwitchProfile,
            },
            Command {
                id: "issue.refresh".to_string(),
                title: "Refresh Issues".to_string(),
                description: Some("Reload issues from JIRA".to_string()),
                category: CommandCategory::Issue,
                keywords: vec![
                    "reload".to_string(),
                    "update".to_string(),
                    "fetch".to_string(),
                ],
                shortcut: Some("r".to_string()),
                action: CommandAction::RefreshIssues,
            },
            Command {
                id: "filter.jql".to_string(),
                title: "Enter JQL Query".to_string(),
                description: Some("Search with JIRA Query Language".to_string()),
                category: CommandCategory::Filter,
                keywords: vec![
                    "search".to_string(),
                    "query".to_string(),
                    "find".to_string(),
                ],
                shortcut: Some(":".to_string()),
                action: CommandAction::OpenJqlInput,
            },
            Command {
                id: "filter.panel".to_string(),
                title: "Open Filter Panel".to_string(),
                description: Some("Show filter options for issues".to_string()),
                category: CommandCategory::Filter,
                keywords: vec![
                    "search".to_string(),
                    "status".to_string(),
                    "assignee".to_string(),
                ],
                shortcut: Some("f".to_string()),
                action: CommandAction::GoToFilters,
            },
            Command {
                id: "filter.clear".to_string(),
                title: "Clear All Filters".to_string(),
                description: Some("Remove all active filters".to_string()),
                category: CommandCategory::Filter,
                keywords: vec!["reset".to_string(), "remove".to_string()],
                shortcut: None,
                action: CommandAction::ClearFilters,
            },
            Command {
                id: "cache.clear".to_string(),
                title: "Clear Cache".to_string(),
                description: Some("Remove cached issue data".to_string()),
                category: CommandCategory::Settings,
                keywords: vec![
                    "reset".to_string(),
                    "storage".to_string(),
                    "data".to_string(),
                ],
                shortcut: None,
                action: CommandAction::ClearCache,
            },
            Command {
                id: "help.show".to_string(),
                title: "Show Help".to_string(),
                description: Some("Display keyboard shortcuts and help".to_string()),
                category: CommandCategory::Help,
                keywords: vec![
                    "shortcuts".to_string(),
                    "keys".to_string(),
                    "documentation".to_string(),
                ],
                shortcut: Some("?".to_string()),
                action: CommandAction::GoToHelp,
            },
        ];

        Self {
            commands,
            recent: VecDeque::with_capacity(Self::MAX_RECENT),
        }
    }

    /// Search for commands matching the query.
    ///
    /// Returns commands sorted by relevance score (highest first).
    /// If query is empty, returns all commands with recent ones boosted.
    pub fn search(&self, query: &str) -> Vec<&Command> {
        if query.is_empty() {
            // Return all commands, with recent ones first
            let mut results: Vec<(&Command, i32)> = self
                .commands
                .iter()
                .map(|cmd| {
                    let recent_boost = self.recent_boost(&cmd.id);
                    (cmd, recent_boost)
                })
                .collect();

            results.sort_by(|a, b| b.1.cmp(&a.1));
            results.into_iter().map(|(cmd, _)| cmd).collect()
        } else {
            let query_lower = query.to_lowercase();
            let mut results: Vec<(&Command, i32)> = self
                .commands
                .iter()
                .filter_map(|cmd| {
                    let score = self.match_score(cmd, &query_lower);
                    if score > 0 {
                        Some((cmd, score))
                    } else {
                        None
                    }
                })
                .collect();

            results.sort_by(|a, b| b.1.cmp(&a.1));
            results.into_iter().map(|(cmd, _)| cmd).collect()
        }
    }

    /// Calculate the match score for a command against a query.
    fn match_score(&self, cmd: &Command, query: &str) -> i32 {
        let mut score = 0;

        // Title match (highest priority)
        let title_lower = cmd.title.to_lowercase();
        if title_lower.contains(query) {
            score += 100;
            // Bonus for prefix match
            if title_lower.starts_with(query) {
                score += 50;
            }
            // Bonus for word boundary match
            if title_lower
                .split_whitespace()
                .any(|word| word.starts_with(query))
            {
                score += 25;
            }
        }

        // Keyword match
        for keyword in &cmd.keywords {
            let keyword_lower = keyword.to_lowercase();
            if keyword_lower.contains(query) {
                score += 50;
                if keyword_lower.starts_with(query) {
                    score += 25;
                }
            }
        }

        // ID match (for power users)
        if cmd.id.to_lowercase().contains(query) {
            score += 25;
        }

        // Description match (lower priority)
        if let Some(desc) = &cmd.description {
            if desc.to_lowercase().contains(query) {
                score += 10;
            }
        }

        // Boost recent commands
        score += self.recent_boost(&cmd.id);

        score
    }

    /// Get the boost score for recently used commands.
    fn recent_boost(&self, id: &str) -> i32 {
        if let Some(pos) = self.recent.iter().position(|i| i == id) {
            // Most recent gets highest boost, decreasing for older entries
            ((Self::MAX_RECENT - pos) as i32) * 10
        } else {
            0
        }
    }

    /// Record a command as recently used.
    pub fn record_used(&mut self, id: &str) {
        // Remove if already in recent list
        self.recent.retain(|i| i != id);

        // Add to front
        self.recent.push_front(id.to_string());

        // Trim to max size
        while self.recent.len() > Self::MAX_RECENT {
            self.recent.pop_back();
        }
    }

    /// Get all registered commands.
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    /// Get recent command IDs.
    pub fn recent(&self) -> Vec<&str> {
        self.recent.iter().map(|s| s.as_str()).collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry() {
        let registry = CommandRegistry::new();
        assert!(!registry.commands().is_empty());
        assert!(registry.recent().is_empty());
    }

    #[test]
    fn test_search_empty_query() {
        let registry = CommandRegistry::new();
        let results = registry.search("");
        // Should return all commands
        assert_eq!(results.len(), registry.commands().len());
    }

    #[test]
    fn test_search_by_title() {
        let registry = CommandRegistry::new();

        // Search for "refresh"
        let results = registry.search("refresh");
        assert!(!results.is_empty());
        assert!(results[0].title.to_lowercase().contains("refresh"));
    }

    #[test]
    fn test_search_by_keyword() {
        let registry = CommandRegistry::new();

        // Search for "reload" (keyword of refresh command)
        let results = registry.search("reload");
        assert!(!results.is_empty());
        assert!(results.iter().any(|c| c.id == "issue.refresh"));
    }

    #[test]
    fn test_search_case_insensitive() {
        let registry = CommandRegistry::new();

        let results_lower = registry.search("help");
        let results_upper = registry.search("HELP");

        assert_eq!(results_lower.len(), results_upper.len());
    }

    #[test]
    fn test_search_no_match() {
        let registry = CommandRegistry::new();

        let results = registry.search("xyznomatch");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_partial_match() {
        let registry = CommandRegistry::new();

        // "go" should match multiple "Go to X" commands
        let results = registry.search("go");
        assert!(results.len() >= 2);
    }

    #[test]
    fn test_record_used() {
        let mut registry = CommandRegistry::new();

        registry.record_used("help.show");
        assert_eq!(registry.recent(), vec!["help.show"]);

        registry.record_used("issue.refresh");
        assert_eq!(registry.recent(), vec!["issue.refresh", "help.show"]);
    }

    #[test]
    fn test_record_used_deduplication() {
        let mut registry = CommandRegistry::new();

        registry.record_used("help.show");
        registry.record_used("issue.refresh");
        registry.record_used("help.show"); // Use again

        // help.show should be at front, not duplicated
        assert_eq!(registry.recent(), vec!["help.show", "issue.refresh"]);
    }

    #[test]
    fn test_record_used_max_size() {
        let mut registry = CommandRegistry::new();

        // Add more than MAX_RECENT commands
        for i in 0..15 {
            registry.record_used(&format!("cmd.{}", i));
        }

        assert_eq!(registry.recent().len(), CommandRegistry::MAX_RECENT);
        // Most recent should be first
        assert_eq!(registry.recent()[0], "cmd.14");
    }

    #[test]
    fn test_recent_boost_in_search() {
        let mut registry = CommandRegistry::new();

        // Record "help.show" as recently used
        registry.record_used("help.show");

        // Search for something that matches multiple commands
        let results = registry.search("");

        // Recent command should appear first or very high
        let help_pos = results.iter().position(|c| c.id == "help.show").unwrap();
        assert!(help_pos < 3, "Recent command should be near top");
    }

    #[test]
    fn test_command_categories() {
        let registry = CommandRegistry::new();

        // Verify we have commands in different categories
        let categories: Vec<_> = registry.commands().iter().map(|c| c.category).collect();

        assert!(categories.contains(&CommandCategory::Navigation));
        assert!(categories.contains(&CommandCategory::Profile));
        assert!(categories.contains(&CommandCategory::Filter));
        assert!(categories.contains(&CommandCategory::Help));
    }

    #[test]
    fn test_command_shortcuts() {
        let registry = CommandRegistry::new();

        // Some commands should have shortcuts
        let with_shortcuts: Vec<_> = registry
            .commands()
            .iter()
            .filter(|c| c.shortcut.is_some())
            .collect();

        assert!(!with_shortcuts.is_empty());
    }

    #[test]
    fn test_category_display() {
        assert_eq!(CommandCategory::Navigation.display(), "Navigation");
        assert_eq!(CommandCategory::Issue.display(), "Issue");
        assert_eq!(CommandCategory::Profile.display(), "Profile");
        assert_eq!(CommandCategory::Filter.display(), "Filter");
        assert_eq!(CommandCategory::Settings.display(), "Settings");
        assert_eq!(CommandCategory::Help.display(), "Help");
    }

    #[test]
    fn test_word_boundary_bonus() {
        let registry = CommandRegistry::new();

        // "go" should prefer "Go to Issue List" over commands where "go" is mid-word
        let results = registry.search("go");
        assert!(!results.is_empty());
        // First result should start with "Go"
        assert!(results[0].title.to_lowercase().starts_with("go"));
    }
}
