//! LazyJira - A terminal-based user interface for JIRA
//!
//! This application provides a TUI for managing JIRA issues directly from the terminal.

mod api;
mod app;
mod cache;
mod commands;
mod config;
mod error;
mod events;
mod logging;
mod ui;

use std::io::{self, stdout};
use std::panic;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::App;
use config::Config;
use events::EventHandler;
use ui::{init_theme, load_theme};

/// Application result type.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging first (before any other operations)
    if let Err(e) = logging::init() {
        eprintln!("Warning: Failed to initialize logging: {}", e);
        // Continue without logging rather than failing completely
    }

    // Load configuration and initialize theme before anything else
    let config = Config::load().unwrap_or_default();
    let theme = load_theme(
        &config.settings.theme,
        config.settings.custom_theme.as_ref(),
    );
    init_theme(theme);

    // Set up panic hook to restore terminal on crash
    setup_panic_hook();

    // Initialize terminal
    let mut terminal = setup_terminal()?;

    // Run the application
    let result = run_app(&mut terminal).await;

    // Restore terminal state
    restore_terminal(&mut terminal)?;

    // Log shutdown
    logging::shutdown();

    // Propagate any error from the application
    result
}

/// Set up a panic hook that restores the terminal state before panicking.
///
/// This ensures that even if the application crashes, the terminal will be
/// restored to its normal state.
fn setup_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Attempt to restore terminal - ignore errors since we're already panicking
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);

        // Call the original panic hook
        original_hook(panic_info);
    }));
}

/// Initialize the terminal for TUI rendering.
///
/// This enables raw mode and switches to the alternate screen buffer.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to its original state.
///
/// This disables raw mode and switches back to the main screen buffer.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Run the main application loop.
///
/// This implements the main event loop following The Elm Architecture pattern:
/// 1. Render the current view
/// 2. Wait for and handle events
/// 3. Update state based on events
/// 4. Repeat until quit
async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    use api::JiraClient;
    use cache::{CacheManager, CacheStatus};
    use tracing::{debug, error, info, warn};

    let mut app = App::new();
    let event_handler = EventHandler::new();

    // Create a JiraClient if a profile is configured
    let mut client: Option<JiraClient> = None;
    let mut cache_manager: Option<CacheManager> = None;

    if let Some(profile) = app.current_profile().cloned() {
        // Initialize cache manager
        let cache_ttl = app.config().settings.cache_ttl_minutes;
        match CacheManager::with_max_size(
            &profile.name,
            cache_ttl,
            app.config().settings.cache_max_size_mb,
        ) {
            Ok(cm) => {
                debug!("Cache manager initialized for profile: {}", profile.name);
                cache_manager = Some(cm);
            }
            Err(e) => {
                warn!("Failed to initialize cache: {}", e);
            }
        }

        match JiraClient::new(&profile).await {
            Ok(c) => {
                info!("Connected to JIRA as profile: {}", profile.name);
                client = Some(c);
            }
            Err(e) => {
                warn!("Failed to create JIRA client: {}", e);
                app.notify_error(format!("Failed to connect to JIRA: {}", e));
            }
        }
    } else {
        app.notify_warning("No profile configured. Press 'P' to add a profile.");
    }

    // Initial issue fetch
    let mut needs_fetch = client.is_some();
    let mut needs_filter_options = client.is_some();

    loop {
        // Fetch issues if needed
        if needs_fetch {
            needs_fetch = false;
            let jql = app.effective_jql();
            // New JIRA API requires bounded queries - default to showing user's issues
            // Use the current sort state from the list view for ordering
            let default_jql = format!(
                "assignee = currentUser() OR reporter = currentUser() {}",
                app.list_view().sort().to_jql()
            );
            let jql_query = if jql.is_empty() { &default_jql } else { &jql };

            debug!("Fetching issues with JQL: {}", jql_query);

            // Try cache first
            let cached_result = cache_manager
                .as_ref()
                .and_then(|cm| cm.get_search_results(jql_query));

            if let Some(cached) = cached_result {
                // Use cached data
                info!(
                    "Loaded {} issues from cache (total: {})",
                    cached.results.issues.len(),
                    cached.results.total
                );
                let issues_count = cached.results.issues.len() as u32;
                app.list_view_mut().set_issues(cached.results.issues);
                app.list_view_mut().set_loading(false);
                app.list_view_mut().pagination_mut().update_from_response(
                    0,
                    issues_count,
                    cached.results.total,
                );
                app.list_view_mut()
                    .set_cache_status(Some(CacheStatus::FromCache));

                // Also fetch fresh data in the background (if client available)
                if let Some(ref c) = client {
                    match c.search_issues(jql_query, 0, 50).await {
                        Ok(result) => {
                            // Update cache
                            if let Some(ref cm) = cache_manager {
                                if let Err(e) = cm.set_search_results(jql_query, &result) {
                                    debug!("Failed to update cache: {}", e);
                                }
                            }
                            // Update display with fresh data
                            let issues_count = result.issues.len() as u32;
                            app.list_view_mut().set_issues(result.issues);
                            app.list_view_mut().pagination_mut().update_from_response(
                                0,
                                issues_count,
                                result.total,
                            );
                            app.list_view_mut()
                                .set_cache_status(Some(CacheStatus::Fresh));
                        }
                        Err(e) => {
                            // Cache data is still valid, just show a warning
                            debug!("Background refresh failed (using cached data): {}", e);
                        }
                    }
                }
            } else if let Some(ref c) = client {
                // No cache, fetch from API
                match c.search_issues(jql_query, 0, 50).await {
                    Ok(result) => {
                        info!(
                            "Loaded {} issues from API (total: {})",
                            result.issues.len(),
                            result.total
                        );
                        // Store in cache
                        if let Some(ref cm) = cache_manager {
                            if let Err(e) = cm.set_search_results(jql_query, &result) {
                                debug!("Failed to cache results: {}", e);
                            }
                        }
                        let issues_count = result.issues.len() as u32;
                        app.list_view_mut().set_issues(result.issues);
                        app.list_view_mut().set_loading(false);
                        app.list_view_mut().pagination_mut().update_from_response(
                            0,
                            issues_count,
                            result.total,
                        );
                        app.list_view_mut()
                            .set_cache_status(Some(CacheStatus::Fresh));
                    }
                    Err(e) => {
                        error!("Failed to fetch issues: {}", e);
                        app.notify_error(format!("Failed to fetch issues: {}", e));
                        app.list_view_mut().set_loading(false);
                        app.list_view_mut().set_cache_status(None);
                    }
                }
            } else {
                // No client available
                app.list_view_mut().set_loading(false);
                app.list_view_mut()
                    .set_cache_status(Some(CacheStatus::Offline));
            }
        }

        // Fetch filter options if needed (once at startup)
        if needs_filter_options {
            if let Some(ref c) = client {
                needs_filter_options = false;
                match c.get_filter_options().await {
                    Ok(options) => {
                        debug!("Loaded filter options");
                        app.set_filter_options(options);
                    }
                    Err(e) => {
                        debug!("Failed to load filter options: {}", e);
                    }
                }
            }
        }

        // Render the current view (View in TEA)
        terminal.draw(|frame| app.view(frame))?;

        // Wait for and handle events (Update in TEA)
        let event = event_handler.next()?;

        // Check list view state before update to detect actions
        let was_loading = app.list_view().is_loading();
        let old_profile = app.current_profile().map(|p| p.name.clone());

        app.update(event);

        // Check if we need to refresh issues
        let is_loading_now = app.list_view().is_loading();
        let new_profile = app.current_profile().map(|p| p.name.clone());

        // Detect profile switch - need to recreate client and cache manager
        if old_profile != new_profile {
            if let Some(profile) = app.current_profile().cloned() {
                // Recreate cache manager for new profile
                let cache_ttl = app.config().settings.cache_ttl_minutes;
                match CacheManager::with_max_size(
                    &profile.name,
                    cache_ttl,
                    app.config().settings.cache_max_size_mb,
                ) {
                    Ok(cm) => {
                        debug!("Cache manager initialized for profile: {}", profile.name);
                        cache_manager = Some(cm);
                    }
                    Err(e) => {
                        warn!("Failed to initialize cache: {}", e);
                        cache_manager = None;
                    }
                }

                match JiraClient::new(&profile).await {
                    Ok(c) => {
                        info!("Switched to profile: {}", profile.name);
                        client = Some(c);
                        needs_fetch = true;
                        needs_filter_options = true;
                    }
                    Err(e) => {
                        error!("Failed to connect to new profile: {}", e);
                        app.notify_error(format!("Failed to connect: {}", e));
                        client = None;
                        app.list_view_mut().set_loading(false);
                    }
                }
            } else {
                client = None;
                cache_manager = None;
            }
        }
        // Detect refresh request (loading changed from false to true)
        else if !was_loading && is_loading_now && client.is_some() {
            needs_fetch = true;
        }

        // Handle pending fetch transitions request
        if let Some(issue_key) = app.take_pending_fetch_transitions() {
            if let Some(ref c) = client {
                debug!("Fetching transitions for issue: {}", issue_key);
                match c.get_transitions(&issue_key).await {
                    Ok(transitions) => {
                        debug!("Loaded {} transitions", transitions.len());
                        app.set_transitions(transitions);
                    }
                    Err(e) => {
                        error!("Failed to fetch transitions: {}", e);
                        app.handle_fetch_transitions_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_fetch_transitions_failure("No JIRA connection");
            }
        }

        // Handle pending transition execution
        if let Some((issue_key, transition_id, fields)) = app.take_pending_transition() {
            if let Some(ref c) = client {
                debug!(
                    "Executing transition {} on issue {}",
                    transition_id, issue_key
                );
                match c.transition_issue(&issue_key, &transition_id, fields).await {
                    Ok(()) => {
                        // Fetch the updated issue to get the new status
                        match c.get_issue(&issue_key).await {
                            Ok(updated_issue) => {
                                info!(
                                    "Transition successful, issue {} now has status: {}",
                                    issue_key, updated_issue.fields.status.name
                                );
                                app.handle_transition_success(updated_issue);
                            }
                            Err(e) => {
                                // Transition succeeded but we couldn't fetch the updated issue
                                warn!(
                                    "Transition succeeded but failed to fetch updated issue: {}",
                                    e
                                );
                                app.notify_success(format!("Issue {} status changed", issue_key));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to execute transition: {}", e);
                        app.handle_transition_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_transition_failure("No JIRA connection");
            }
        }

        // Handle pending fetch assignees request
        if let Some((_issue_key, project_key)) = app.take_pending_fetch_assignees() {
            if let Some(ref c) = client {
                debug!("Fetching assignable users for project: {}", project_key);
                match c.get_assignable_users(&project_key).await {
                    Ok(users) => {
                        debug!("Loaded {} assignable users", users.len());
                        app.set_assignable_users(users);
                    }
                    Err(e) => {
                        error!("Failed to fetch assignable users: {}", e);
                        app.handle_fetch_assignees_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_fetch_assignees_failure("No JIRA connection");
            }
        }

        // Handle pending assignee change
        if let Some((issue_key, account_id)) = app.take_pending_assignee_change() {
            if let Some(ref c) = client {
                debug!(
                    "Changing assignee on issue {} to {:?}",
                    issue_key, account_id
                );
                match c.update_assignee(&issue_key, account_id.as_deref()).await {
                    Ok(()) => {
                        // Fetch the updated issue to get the new assignee
                        match c.get_issue(&issue_key).await {
                            Ok(updated_issue) => {
                                info!("Assignee changed for issue {}", issue_key);
                                app.handle_assignee_change_success(updated_issue);
                            }
                            Err(e) => {
                                warn!("Assignee changed but failed to fetch updated issue: {}", e);
                                app.notify_success(format!("Issue {} assignee changed", issue_key));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to change assignee: {}", e);
                        app.handle_assignee_change_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_assignee_change_failure("No JIRA connection");
            }
        }

        // Handle pending fetch priorities request
        if let Some(_issue_key) = app.take_pending_fetch_priorities() {
            if let Some(ref c) = client {
                debug!("Fetching priorities");
                match c.get_priorities().await {
                    Ok(priorities) => {
                        debug!("Loaded {} priorities", priorities.len());
                        app.set_priorities(priorities);
                    }
                    Err(e) => {
                        error!("Failed to fetch priorities: {}", e);
                        app.handle_fetch_priorities_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_fetch_priorities_failure("No JIRA connection");
            }
        }

        // Handle pending priority change
        if let Some((issue_key, priority_id)) = app.take_pending_priority_change() {
            if let Some(ref c) = client {
                debug!(
                    "Changing priority on issue {} to {}",
                    issue_key, priority_id
                );
                match c.update_priority(&issue_key, &priority_id).await {
                    Ok(()) => {
                        // Fetch the updated issue to get the new priority
                        match c.get_issue(&issue_key).await {
                            Ok(updated_issue) => {
                                info!("Priority changed for issue {}", issue_key);
                                app.handle_priority_change_success(updated_issue);
                            }
                            Err(e) => {
                                warn!("Priority changed but failed to fetch updated issue: {}", e);
                                app.notify_success(format!("Issue {} priority changed", issue_key));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to change priority: {}", e);
                        app.handle_priority_change_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_priority_change_failure("No JIRA connection");
            }
        }

        // Handle fetch comments request
        if let Some(issue_key) = app.take_pending_fetch_comments() {
            if let Some(ref c) = client {
                debug!("Fetching comments for issue {}", issue_key);
                match c.get_comments(&issue_key, 0, 50).await {
                    Ok(response) => {
                        debug!("Loaded {} comments", response.comments.len());
                        app.handle_comments_fetched(response.comments, response.total);
                    }
                    Err(e) => {
                        error!("Failed to fetch comments: {}", e);
                        app.handle_fetch_comments_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_fetch_comments_failure("No JIRA connection");
            }
        }

        // Handle submit comment request
        if let Some((issue_key, body)) = app.take_pending_submit_comment() {
            if let Some(ref c) = client {
                debug!("Submitting comment to issue {}", issue_key);
                match c.add_comment(&issue_key, &body).await {
                    Ok(comment) => {
                        info!("Comment added to issue {}", issue_key);
                        app.handle_comment_submitted(comment);
                    }
                    Err(e) => {
                        error!("Failed to submit comment: {}", e);
                        app.handle_submit_comment_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_submit_comment_failure("No JIRA connection");
            }
        }

        // Handle fetch labels request
        if let Some(_issue_key) = app.take_pending_fetch_labels() {
            if let Some(ref c) = client {
                debug!("Fetching labels");
                match c.get_labels().await {
                    Ok(labels) => {
                        debug!("Loaded {} labels", labels.len());
                        app.set_labels(labels);
                    }
                    Err(e) => {
                        error!("Failed to fetch labels: {}", e);
                        app.handle_fetch_labels_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_fetch_labels_failure("No JIRA connection");
            }
        }

        // Handle add label request
        if let Some((issue_key, label)) = app.take_pending_add_label() {
            if let Some(ref c) = client {
                debug!("Adding label {} to issue {}", label, issue_key);
                match c.add_labels(&issue_key, vec![label.clone()]).await {
                    Ok(()) => {
                        // Fetch the updated issue
                        match c.get_issue(&issue_key).await {
                            Ok(updated_issue) => {
                                info!("Label added to issue {}", issue_key);
                                app.handle_label_change_success(updated_issue);
                            }
                            Err(e) => {
                                warn!("Label added but failed to fetch updated issue: {}", e);
                                app.notify_success(format!("Label '{}' added", label));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to add label: {}", e);
                        app.handle_label_change_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_label_change_failure("No JIRA connection");
            }
        }

        // Handle remove label request
        if let Some((issue_key, label)) = app.take_pending_remove_label() {
            if let Some(ref c) = client {
                debug!("Removing label {} from issue {}", label, issue_key);
                match c.remove_labels(&issue_key, vec![label.clone()]).await {
                    Ok(()) => {
                        // Fetch the updated issue
                        match c.get_issue(&issue_key).await {
                            Ok(updated_issue) => {
                                info!("Label removed from issue {}", issue_key);
                                app.handle_label_change_success(updated_issue);
                            }
                            Err(e) => {
                                warn!("Label removed but failed to fetch updated issue: {}", e);
                                app.notify_success(format!("Label '{}' removed", label));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to remove label: {}", e);
                        app.handle_label_change_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_label_change_failure("No JIRA connection");
            }
        }

        // Handle fetch components request
        if let Some((_issue_key, project_key)) = app.take_pending_fetch_components() {
            if let Some(ref c) = client {
                debug!("Fetching components for project {}", project_key);
                match c.get_project_components(&project_key).await {
                    Ok(components) => {
                        debug!("Loaded {} components", components.len());
                        let component_names: Vec<String> =
                            components.into_iter().map(|c| c.name).collect();
                        app.set_components(component_names);
                    }
                    Err(e) => {
                        error!("Failed to fetch components: {}", e);
                        app.handle_fetch_components_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_fetch_components_failure("No JIRA connection");
            }
        }

        // Handle add component request
        if let Some((issue_key, component)) = app.take_pending_add_component() {
            if let Some(ref c) = client {
                debug!("Adding component {} to issue {}", component, issue_key);
                match c.add_components(&issue_key, vec![component.clone()]).await {
                    Ok(()) => {
                        // Fetch the updated issue
                        match c.get_issue(&issue_key).await {
                            Ok(updated_issue) => {
                                info!("Component added to issue {}", issue_key);
                                app.handle_component_change_success(updated_issue);
                            }
                            Err(e) => {
                                warn!("Component added but failed to fetch updated issue: {}", e);
                                app.notify_success(format!("Component '{}' added", component));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to add component: {}", e);
                        app.handle_component_change_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_component_change_failure("No JIRA connection");
            }
        }

        // Handle remove component request
        if let Some((issue_key, component)) = app.take_pending_remove_component() {
            if let Some(ref c) = client {
                debug!("Removing component {} from issue {}", component, issue_key);
                match c
                    .remove_components(&issue_key, vec![component.clone()])
                    .await
                {
                    Ok(()) => {
                        // Fetch the updated issue
                        match c.get_issue(&issue_key).await {
                            Ok(updated_issue) => {
                                info!("Component removed from issue {}", issue_key);
                                app.handle_component_change_success(updated_issue);
                            }
                            Err(e) => {
                                warn!("Component removed but failed to fetch updated issue: {}", e);
                                app.notify_success(format!("Component '{}' removed", component));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to remove component: {}", e);
                        app.handle_component_change_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_component_change_failure("No JIRA connection");
            }
        }

        // Handle fetch changelog request
        if let Some((issue_key, start_at)) = app.take_pending_fetch_changelog() {
            if let Some(ref c) = client {
                debug!("Fetching changelog for issue {} (start_at: {})", issue_key, start_at);
                match c.get_changelog(&issue_key, start_at, 50).await {
                    Ok(changelog) => {
                        debug!(
                            "Loaded {} history entries (total: {})",
                            changelog.histories.len(),
                            changelog.total
                        );
                        // If start_at > 0, we're appending to existing history
                        app.handle_changelog_fetched(changelog, start_at > 0);
                    }
                    Err(e) => {
                        error!("Failed to fetch changelog: {}", e);
                        app.handle_fetch_changelog_failure(&e.to_string());
                    }
                }
            } else {
                app.handle_fetch_changelog_failure("No JIRA connection");
            }
        }

        // Check if we should quit
        if app.should_quit() {
            break;
        }
    }

    Ok(())
}
