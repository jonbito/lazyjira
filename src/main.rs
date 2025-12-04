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
mod tasks;
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

/// Suspends the TUI to allow an external process to use the terminal.
///
/// This function:
/// - Disables raw mode so the external process gets normal terminal behavior
/// - Leaves the alternate screen to show the normal terminal buffer
///
/// After calling this function, the terminal is in a state suitable for
/// running external commands like text editors.
///
/// # Errors
///
/// Returns an error if raw mode cannot be disabled or if leaving the
/// alternate screen fails.
pub fn suspend_tui<W: io::Write>(stdout: &mut W) -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}

/// Resumes the TUI after an external process has completed.
///
/// This function:
/// - Re-enables raw mode for TUI input handling
/// - Re-enters the alternate screen buffer
/// - Clears the terminal to remove any artifacts from the external process
///
/// # Errors
///
/// Returns an error if raw mode cannot be enabled, if entering the
/// alternate screen fails, or if clearing the terminal fails.
pub fn resume_tui<W: io::Write>(
    stdout: &mut W,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    terminal.clear()?;
    Ok(())
}

/// RAII guard that ensures the TUI is resumed even if the external process panics or fails.
///
/// When this guard is dropped, it automatically calls `resume_tui` to restore
/// the terminal state. This ensures that even if an error occurs or the external
/// process is killed, the terminal will be properly restored.
///
/// # Example
///
/// ```ignore
/// let guard = TuiSuspendGuard::new(&mut stdout, &mut terminal)?;
/// // Run external editor - terminal is suspended
/// // ... external process runs ...
/// // When guard goes out of scope, TUI is automatically resumed
/// drop(guard);
/// ```
pub struct TuiSuspendGuard<'a> {
    stdout: &'a mut io::Stdout,
    terminal: &'a mut Terminal<CrosstermBackend<io::Stdout>>,
}

impl<'a> TuiSuspendGuard<'a> {
    /// Creates a new TuiSuspendGuard and suspends the TUI.
    ///
    /// This calls `suspend_tui` immediately. The TUI will be resumed when
    /// the guard is dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if suspending the TUI fails.
    pub fn new(
        stdout: &'a mut io::Stdout,
        terminal: &'a mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<Self> {
        suspend_tui(stdout)?;
        Ok(Self { stdout, terminal })
    }
}

impl<'a> Drop for TuiSuspendGuard<'a> {
    fn drop(&mut self) {
        // Attempt to resume TUI - log error but don't panic since we may already be unwinding
        if let Err(e) = resume_tui(self.stdout, self.terminal) {
            // Use eprintln since logging may not be available during panic unwind
            eprintln!("Warning: Failed to resume TUI: {}", e);
        }
    }
}

/// Run the main application loop.
///
/// This implements the main event loop following The Elm Architecture pattern:
/// 1. Poll for completed async tasks (non-blocking)
/// 2. Render the current view
/// 3. Wait for and handle events
/// 4. Update state based on events
/// 5. Spawn background tasks for pending operations
/// 6. Repeat until quit
///
/// The async task system keeps the UI responsive by running API calls in
/// background tasks and communicating results through channels.
async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    use api::JiraClient;
    use cache::{CacheManager, CacheStatus};
    use tasks::{create_task_channel, ApiMessage};
    use tracing::{debug, error, info, warn};
    use ui::ExternalEditor;

    let mut app = App::new();
    let event_handler = EventHandler::new();

    // Create the async task channel for background operations
    let (mut task_rx, task_spawner) = create_task_channel();

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
                let error_msg = format!("Failed to connect to JIRA: {}", e);
                app.notify_error(&error_msg);
                app.list_view_mut().set_loading(false);
                app.list_view_mut().set_error(error_msg);
            }
        }
    } else {
        // Keep loading state true - render_loading will show "No profile configured" message
        app.notify_warning("No profile configured. Press 'P' to add a profile.");
    }

    // Initial issue fetch
    let mut needs_fetch = client.is_some();
    let mut needs_filter_options = client.is_some();

    loop {
        // =================================================================
        // STEP 1: Poll for completed async tasks (non-blocking)
        // =================================================================
        while let Ok(msg) = task_rx.try_recv() {
            match msg {
                ApiMessage::ClientConnected(result) => match result {
                    Ok(c) => {
                        info!("Connected to JIRA");
                        client = Some(c);
                        needs_fetch = true;
                        needs_filter_options = true;
                        app.list_view_mut().clear_error();
                    }
                    Err(e) => {
                        error!("Failed to connect to JIRA: {}", e);
                        let error_msg = format!("Failed to connect: {}", e);
                        app.notify_error(&error_msg);
                        app.list_view_mut().set_loading(false);
                        app.list_view_mut().set_error(error_msg);
                    }
                },
                ApiMessage::IssuesFetched {
                    jql,
                    result,
                    is_background_refresh,
                } => {
                    match result {
                        Ok(search_result) => {
                            let issues_count = search_result.issues.len() as u32;
                            let has_more = search_result.has_more();
                            let next_page_token = search_result.next_page_token.clone();
                            if is_background_refresh {
                                info!(
                                    "Background refresh: {} issues (total: {}, has_more: {}, has_token: {})",
                                    issues_count, search_result.total, has_more, next_page_token.is_some()
                                );
                            } else {
                                info!(
                                    "Loaded {} issues from API (total: {}, has_more: {}, has_token: {})",
                                    issues_count, search_result.total, has_more, next_page_token.is_some()
                                );
                            }
                            // Update cache
                            if let Some(ref cm) = cache_manager {
                                if let Err(e) = cm.set_search_results(&jql, &search_result) {
                                    debug!("Failed to cache results: {}", e);
                                }
                            }
                            app.list_view_mut().set_issues(search_result.issues);
                            app.list_view_mut().set_loading(false);
                            app.list_view_mut().clear_error();
                            app.list_view_mut().pagination_mut().update_from_response(
                                0,
                                issues_count,
                                search_result.total,
                                has_more,
                                next_page_token,
                            );
                            app.list_view_mut()
                                .set_cache_status(Some(CacheStatus::Fresh));
                        }
                        Err(e) => {
                            if is_background_refresh {
                                debug!("Background refresh failed (using cached data): {}", e);
                            } else {
                                error!("Failed to fetch issues: {}", e);
                                let error_msg = format!("Failed to fetch issues: {}", e);
                                app.notify_error(&error_msg);
                                app.list_view_mut().set_error(&error_msg);
                                app.list_view_mut().set_loading(false);
                                app.list_view_mut().set_cache_status(None);
                            }
                        }
                    }
                }
                ApiMessage::FilterOptionsFetched(result) => match result {
                    Ok(options) => {
                        debug!("Loaded filter options");
                        app.set_filter_options(options);
                    }
                    Err(e) => {
                        debug!("Failed to load filter options: {}", e);
                    }
                },
                ApiMessage::LoadMoreFetched { result } => match result {
                    Ok(search_result) => {
                        let has_more = search_result.has_more();
                        info!(
                            "Loaded {} more issues (total: {}, has_more: {}, has_token: {})",
                            search_result.issues.len(),
                            search_result.total,
                            has_more,
                            search_result.next_page_token.is_some()
                        );
                        app.handle_load_more_success(
                            search_result.issues,
                            search_result.total,
                            has_more,
                            search_result.next_page_token,
                        );
                    }
                    Err(e) => {
                        error!("Failed to load more issues: {}", e);
                        app.handle_load_more_failure(&e);
                    }
                },
                ApiMessage::TransitionsFetched {
                    issue_key: _,
                    result,
                } => match result {
                    Ok(transitions) => {
                        debug!("Loaded {} transitions", transitions.len());
                        app.set_transitions(transitions);
                    }
                    Err(e) => {
                        error!("Failed to fetch transitions: {}", e);
                        app.handle_fetch_transitions_failure(&e);
                    }
                },
                ApiMessage::TransitionExecuted { issue_key, result } => match result {
                    Ok(updated_issue) => {
                        info!(
                            "Transition successful, issue {} now has status: {}",
                            issue_key, updated_issue.fields.status.name
                        );
                        app.handle_transition_success(updated_issue);
                    }
                    Err(e) => {
                        error!("Failed to execute transition: {}", e);
                        app.handle_transition_failure(&e);
                    }
                },
                ApiMessage::AssigneesFetched { result } => match result {
                    Ok(users) => {
                        debug!("Loaded {} assignable users", users.len());
                        app.set_assignable_users(users);
                    }
                    Err(e) => {
                        error!("Failed to fetch assignable users: {}", e);
                        app.handle_fetch_assignees_failure(&e);
                    }
                },
                ApiMessage::AssigneeChanged { result } => match result {
                    Ok(updated_issue) => {
                        info!("Assignee changed for issue {}", updated_issue.key);
                        app.handle_assignee_change_success(updated_issue);
                    }
                    Err(e) => {
                        error!("Failed to change assignee: {}", e);
                        app.handle_assignee_change_failure(&e);
                    }
                },
                ApiMessage::PrioritiesFetched(result) => match result {
                    Ok(priorities) => {
                        debug!("Loaded {} priorities", priorities.len());
                        app.set_priorities(priorities);
                    }
                    Err(e) => {
                        error!("Failed to fetch priorities: {}", e);
                        app.handle_fetch_priorities_failure(&e);
                    }
                },
                ApiMessage::PriorityChanged { result } => match result {
                    Ok(updated_issue) => {
                        info!("Priority changed for issue {}", updated_issue.key);
                        app.handle_priority_change_success(updated_issue);
                    }
                    Err(e) => {
                        error!("Failed to change priority: {}", e);
                        app.handle_priority_change_failure(&e);
                    }
                },
                ApiMessage::CommentsFetched { result } => match result {
                    Ok((comments, total)) => {
                        debug!("Loaded {} comments", comments.len());
                        app.handle_comments_fetched(comments, total);
                    }
                    Err(e) => {
                        error!("Failed to fetch comments: {}", e);
                        app.handle_fetch_comments_failure(&e);
                    }
                },
                ApiMessage::CommentSubmitted { result } => match result {
                    Ok(comment) => {
                        info!("Comment submitted");
                        app.handle_comment_submitted(comment);
                    }
                    Err(e) => {
                        error!("Failed to submit comment: {}", e);
                        app.handle_submit_comment_failure(&e);
                    }
                },
                ApiMessage::IssueUpdated { result } => match result {
                    Ok(updated_issue) => {
                        info!("Issue {} updated successfully", updated_issue.key);
                        app.handle_issue_update_success(updated_issue);
                    }
                    Err(e) => {
                        error!("Failed to update issue: {}", e);
                        app.handle_issue_update_failure(&e);
                    }
                },
                ApiMessage::LabelsFetched(result) => match result {
                    Ok(labels) => {
                        debug!("Loaded {} labels", labels.len());
                        app.set_labels(labels);
                    }
                    Err(e) => {
                        error!("Failed to fetch labels: {}", e);
                        app.handle_fetch_labels_failure(&e);
                    }
                },
                ApiMessage::LabelChanged { result } => match result {
                    Ok(updated_issue) => {
                        info!("Label changed for issue {}", updated_issue.key);
                        app.handle_label_change_success(updated_issue);
                    }
                    Err(e) => {
                        error!("Failed to change label: {}", e);
                        app.handle_label_change_failure(&e);
                    }
                },
                ApiMessage::ComponentsFetched(result) => match result {
                    Ok(components) => {
                        debug!("Loaded {} components", components.len());
                        app.set_components(components);
                    }
                    Err(e) => {
                        error!("Failed to fetch components: {}", e);
                        app.handle_fetch_components_failure(&e);
                    }
                },
                ApiMessage::ComponentChanged { result } => match result {
                    Ok(updated_issue) => {
                        info!("Component changed for issue {}", updated_issue.key);
                        app.handle_component_change_success(updated_issue);
                    }
                    Err(e) => {
                        error!("Failed to change component: {}", e);
                        app.handle_component_change_failure(&e);
                    }
                },
                ApiMessage::ChangelogFetched { result, is_append } => match result {
                    Ok(changelog) => {
                        debug!(
                            "Loaded {} history entries (total: {})",
                            changelog.histories.len(),
                            changelog.total
                        );
                        app.handle_changelog_fetched(changelog, is_append);
                    }
                    Err(e) => {
                        error!("Failed to fetch changelog: {}", e);
                        app.handle_fetch_changelog_failure(&e);
                    }
                },
                ApiMessage::LinkedIssueFetched { result } => match result {
                    Ok(issue) => {
                        info!("Loaded linked issue: {}", issue.key);
                        app.handle_navigate_to_issue_success(issue);
                    }
                    Err(e) => {
                        error!("Failed to load linked issue: {}", e);
                        app.handle_navigate_to_issue_failure(&e);
                    }
                },
                ApiMessage::LinkTypesFetched(result) => match result {
                    Ok(link_types) => {
                        info!("Loaded {} link types", link_types.len());
                        app.handle_link_types_success(link_types);
                    }
                    Err(e) => {
                        error!("Failed to load link types: {}", e);
                        app.handle_link_types_failure(&e);
                    }
                },
                ApiMessage::IssueSearchResults { result } => match result {
                    Ok(suggestions) => {
                        info!("Found {} issue suggestions", suggestions.len());
                        app.handle_issue_search_success(suggestions);
                    }
                    Err(e) => {
                        error!("Failed to search issues: {}", e);
                        app.handle_issue_search_failure(&e);
                    }
                },
                ApiMessage::LinkCreated { issue_key, result } => match result {
                    Ok(()) => {
                        info!("Link created successfully");
                        app.handle_create_link_success(&issue_key);
                    }
                    Err(e) => {
                        error!("Failed to create link: {}", e);
                        app.handle_create_link_failure(&e);
                    }
                },
                ApiMessage::LinkDeleted { issue_key, result } => match result {
                    Ok(()) => {
                        info!("Link deleted successfully");
                        app.handle_delete_link_success(&issue_key);
                    }
                    Err(e) => {
                        error!("Failed to delete link: {}", e);
                        app.handle_delete_link_failure(&e);
                    }
                },
                ApiMessage::IssueCreated(result) => match result {
                    Ok(response) => {
                        info!("Issue created successfully: {}", response.key);
                        app.handle_create_issue_success(response);
                    }
                    Err(e) => {
                        error!("Failed to create issue: {}", e);
                        app.handle_create_issue_failure(&e);
                    }
                },
                ApiMessage::IssueTypesFetched(result) => match result {
                    Ok(issue_types) => {
                        debug!("Loaded {} issue types", issue_types.len());
                        app.handle_issue_types_fetched(issue_types);
                    }
                    Err(e) => {
                        error!("Failed to fetch issue types: {}", e);
                        app.handle_fetch_issue_types_failure(&e);
                    }
                },
            }
        }

        // =================================================================
        // STEP 2: Spawn background tasks for pending fetch operations
        // =================================================================

        // Fetch issues if needed (spawn in background)
        if needs_fetch {
            needs_fetch = false;
            let jql = app.effective_jql();
            let default_jql = format!(
                "assignee = currentUser() OR reporter = currentUser() {}",
                app.list_view().sort().to_jql()
            );
            let jql_query = if jql.is_empty() {
                default_jql.clone()
            } else {
                jql.clone()
            };

            debug!("Fetching issues with JQL: {}", jql_query);

            // Try cache first
            let cached_result = cache_manager
                .as_ref()
                .and_then(|cm| cm.get_search_results(&jql_query));

            if let Some(cached) = cached_result {
                // Use cached data immediately
                let issues_count = cached.results.issues.len() as u32;
                let has_more = cached.results.has_more();
                // Note: cached token may be stale, but background refresh will update it
                let next_page_token = cached.results.next_page_token.clone();
                info!(
                    "Loaded {} issues from cache (total: {}, has_more: {}, has_token: {})",
                    issues_count,
                    cached.results.total,
                    has_more,
                    next_page_token.is_some()
                );
                app.list_view_mut().set_issues(cached.results.issues);
                app.list_view_mut().set_loading(false);
                app.list_view_mut().clear_error();
                app.list_view_mut().pagination_mut().update_from_response(
                    0,
                    issues_count,
                    cached.results.total,
                    has_more,
                    next_page_token,
                );
                app.list_view_mut()
                    .set_cache_status(Some(CacheStatus::FromCache));

                // Also spawn background refresh (non-blocking)
                if let Some(ref c) = client {
                    let page_size = app.list_view().pagination().page_size;
                    task_spawner.spawn_fetch_issues(c, jql_query, 0, page_size, true);
                }
            } else if let Some(ref c) = client {
                // No cache, spawn fetch from API (non-blocking)
                let page_size = app.list_view().pagination().page_size;
                task_spawner.spawn_fetch_issues(c, jql_query, 0, page_size, false);
            } else {
                // No client available
                app.list_view_mut().set_loading(false);
                app.list_view_mut()
                    .set_cache_status(Some(CacheStatus::Offline));
            }
        }

        // Fetch filter options if needed (spawn in background)
        if needs_filter_options {
            if let Some(ref c) = client {
                needs_filter_options = false;
                task_spawner.spawn_fetch_filter_options(c);
            }
        }

        // =================================================================
        // STEP 3: Render the current view (View in TEA)
        // =================================================================
        terminal.draw(|frame| app.view(frame))?;

        // =================================================================
        // STEP 4: Wait for and handle events (Update in TEA)
        // =================================================================
        let event = event_handler.next()?;

        // Check list view state before update to detect actions
        let was_loading = app.list_view().is_loading();
        let old_profile = app.current_profile().cloned();

        app.update(event);

        // Handle pending external editor request (must be synchronous)
        if let Some((issue_key, current_content)) = app.take_pending_external_edit() {
            debug!(issue_key = %issue_key, "Opening external editor");

            // Get a reference to stdout for suspend/resume
            let mut stdout = stdout();

            // Use the guard to ensure TUI is always restored
            let _guard = TuiSuspendGuard::new(&mut stdout, terminal)?;

            // Launch external editor synchronously
            let editor = ExternalEditor::new();
            let result = editor.open(&issue_key, &current_content);

            // Guard is dropped here, restoring TUI

            // Handle the result
            match result {
                Ok(edit_result) if edit_result.was_modified => {
                    info!(issue_key = %issue_key, "External editor content modified");
                    app.apply_external_edit_result(edit_result.content);
                }
                Ok(_) => {
                    debug!(issue_key = %issue_key, "External editor content unchanged");
                    // No changes, do nothing
                }
                Err(e) => {
                    error!(issue_key = %issue_key, error = %e, "External editor error");
                    app.notify_error(format!("Editor error: {}", e));
                }
            }

            // Force a redraw to update the view
            continue;
        }

        // =================================================================
        // STEP 5: Spawn background tasks for pending operations
        // =================================================================

        // Check if we need to refresh issues
        let is_loading_now = app.list_view().is_loading();
        let new_profile = app.current_profile().cloned();

        // Detect profile change (switch or update) - need to recreate client and cache manager
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

                // Spawn client connection in background
                info!("Switching to profile: {}", profile.name);
                client = None; // Clear old client while connecting
                task_spawner.spawn_connect(profile);
            } else {
                client = None;
                cache_manager = None;
            }
        }
        // Detect refresh request (loading changed from false to true)
        else if !was_loading && is_loading_now {
            if client.is_some() {
                needs_fetch = true;
            } else if let Some(profile) = app.current_profile().cloned() {
                // No client but profile exists - spawn reconnection in background
                info!("Reconnecting to JIRA for profile: {}", profile.name);

                // Reinitialize cache manager
                let cache_ttl = app.config().settings.cache_ttl_minutes;
                match CacheManager::with_max_size(
                    &profile.name,
                    cache_ttl,
                    app.config().settings.cache_max_size_mb,
                ) {
                    Ok(cm) => {
                        debug!("Cache manager reinitialized for profile: {}", profile.name);
                        cache_manager = Some(cm);
                    }
                    Err(e) => {
                        warn!("Failed to reinitialize cache: {}", e);
                    }
                }

                task_spawner.spawn_connect(profile);
            } else {
                // No client and no profile configured
                app.list_view_mut().set_loading(false);
                app.list_view_mut()
                    .set_error("No profile configured. Press 'P' to add a profile.");
            }
        }

        // Handle pending load more request (pagination) - spawn in background
        if app.take_pending_load_more() {
            if let Some(ref c) = client {
                let jql = app.effective_jql();
                let default_jql = format!(
                    "assignee = currentUser() OR reporter = currentUser() {}",
                    app.list_view().sort().to_jql()
                );
                let jql_query = if jql.is_empty() { default_jql } else { jql };
                let page_size = app.list_view().pagination().page_size;
                let next_page_token = app.list_view().pagination().next_page_token.clone();

                info!(
                    "Loading more issues: page_size={}, has_token={}, jql={}",
                    page_size,
                    next_page_token.is_some(),
                    jql_query
                );

                task_spawner.spawn_load_more(c, jql_query, page_size, next_page_token);
            } else {
                app.handle_load_more_failure("No JIRA connection");
            }
        }

        // Handle pending fetch transitions request - spawn in background
        if let Some(issue_key) = app.take_pending_fetch_transitions() {
            if let Some(ref c) = client {
                debug!("Fetching transitions for issue: {}", issue_key);
                task_spawner.spawn_fetch_transitions(c, issue_key);
            } else {
                app.handle_fetch_transitions_failure("No JIRA connection");
            }
        }

        // Handle pending transition execution - spawn in background
        if let Some((issue_key, transition_id, fields)) = app.take_pending_transition() {
            if let Some(ref c) = client {
                debug!(
                    "Executing transition {} on issue {}",
                    transition_id, issue_key
                );
                task_spawner.spawn_transition(c, issue_key, transition_id, fields);
            } else {
                app.handle_transition_failure("No JIRA connection");
            }
        }

        // Handle pending fetch assignees request - spawn in background
        if let Some((_issue_key, project_key)) = app.take_pending_fetch_assignees() {
            if let Some(ref c) = client {
                debug!("Fetching assignable users for project: {}", project_key);
                task_spawner.spawn_fetch_assignees(c, project_key);
            } else {
                app.handle_fetch_assignees_failure("No JIRA connection");
            }
        }

        // Handle pending assignee change - spawn in background
        if let Some((issue_key, account_id)) = app.take_pending_assignee_change() {
            if let Some(ref c) = client {
                debug!(
                    "Changing assignee on issue {} to {:?}",
                    issue_key, account_id
                );
                task_spawner.spawn_change_assignee(c, issue_key, account_id);
            } else {
                app.handle_assignee_change_failure("No JIRA connection");
            }
        }

        // Handle pending fetch priorities request - spawn in background
        if let Some(_issue_key) = app.take_pending_fetch_priorities() {
            if let Some(ref c) = client {
                debug!("Fetching priorities");
                task_spawner.spawn_fetch_priorities(c);
            } else {
                app.handle_fetch_priorities_failure("No JIRA connection");
            }
        }

        // Handle pending priority change - spawn in background
        if let Some((issue_key, priority_id)) = app.take_pending_priority_change() {
            if let Some(ref c) = client {
                debug!(
                    "Changing priority on issue {} to {}",
                    issue_key, priority_id
                );
                task_spawner.spawn_change_priority(c, issue_key, priority_id);
            } else {
                app.handle_priority_change_failure("No JIRA connection");
            }
        }

        // Handle fetch comments request - spawn in background
        if let Some(issue_key) = app.take_pending_fetch_comments() {
            if let Some(ref c) = client {
                debug!("Fetching comments for issue {}", issue_key);
                task_spawner.spawn_fetch_comments(c, issue_key);
            } else {
                app.handle_fetch_comments_failure("No JIRA connection");
            }
        }

        // Handle submit comment request - spawn in background
        if let Some((issue_key, body)) = app.take_pending_submit_comment() {
            if let Some(ref c) = client {
                debug!("Submitting comment to issue {}", issue_key);
                task_spawner.spawn_submit_comment(c, issue_key, body);
            } else {
                app.handle_submit_comment_failure("No JIRA connection");
            }
        }

        // Handle issue update request (summary/description edits) - spawn in background
        if let Some((issue_key, update_request)) = app.take_pending_issue_update() {
            if let Some(ref c) = client {
                debug!("Updating issue {}", issue_key);
                task_spawner.spawn_update_issue(c, issue_key, update_request);
            } else {
                app.handle_issue_update_failure("No JIRA connection");
            }
        }

        // Handle fetch labels request - spawn in background
        if let Some(_issue_key) = app.take_pending_fetch_labels() {
            if let Some(ref c) = client {
                debug!("Fetching labels");
                task_spawner.spawn_fetch_labels(c);
            } else {
                app.handle_fetch_labels_failure("No JIRA connection");
            }
        }

        // Handle add label request - spawn in background
        if let Some((issue_key, label)) = app.take_pending_add_label() {
            if let Some(ref c) = client {
                debug!("Adding label {} to issue {}", label, issue_key);
                task_spawner.spawn_add_label(c, issue_key, label);
            } else {
                app.handle_label_change_failure("No JIRA connection");
            }
        }

        // Handle remove label request - spawn in background
        if let Some((issue_key, label)) = app.take_pending_remove_label() {
            if let Some(ref c) = client {
                debug!("Removing label {} from issue {}", label, issue_key);
                task_spawner.spawn_remove_label(c, issue_key, label);
            } else {
                app.handle_label_change_failure("No JIRA connection");
            }
        }

        // Handle fetch components request - spawn in background
        if let Some((_issue_key, project_key)) = app.take_pending_fetch_components() {
            if let Some(ref c) = client {
                debug!("Fetching components for project {}", project_key);
                task_spawner.spawn_fetch_components(c, project_key);
            } else {
                app.handle_fetch_components_failure("No JIRA connection");
            }
        }

        // Handle add component request - spawn in background
        if let Some((issue_key, component)) = app.take_pending_add_component() {
            if let Some(ref c) = client {
                debug!("Adding component {} to issue {}", component, issue_key);
                task_spawner.spawn_add_component(c, issue_key, component);
            } else {
                app.handle_component_change_failure("No JIRA connection");
            }
        }

        // Handle remove component request - spawn in background
        if let Some((issue_key, component)) = app.take_pending_remove_component() {
            if let Some(ref c) = client {
                debug!("Removing component {} from issue {}", component, issue_key);
                task_spawner.spawn_remove_component(c, issue_key, component);
            } else {
                app.handle_component_change_failure("No JIRA connection");
            }
        }

        // Handle fetch changelog request - spawn in background
        if let Some((issue_key, start_at)) = app.take_pending_fetch_changelog() {
            if let Some(ref c) = client {
                debug!(
                    "Fetching changelog for issue {} (start_at: {})",
                    issue_key, start_at
                );
                let is_append = start_at > 0;
                task_spawner.spawn_fetch_changelog(c, issue_key, start_at, is_append);
            } else {
                app.handle_fetch_changelog_failure("No JIRA connection");
            }
        }

        // Handle linked issue navigation request - spawn in background
        if let Some(issue_key) = app.take_pending_navigate_to_issue() {
            if let Some(ref c) = client {
                debug!("Navigating to linked issue: {}", issue_key);
                task_spawner.spawn_fetch_linked_issue(c, issue_key);
            } else {
                app.handle_navigate_to_issue_failure("No JIRA connection");
            }
        }

        // Handle fetch link types request - spawn in background
        if let Some(_issue_key) = app.take_pending_fetch_link_types() {
            if let Some(ref c) = client {
                debug!("Fetching link types");
                task_spawner.spawn_fetch_link_types(c);
            } else {
                app.handle_link_types_failure("No JIRA connection");
            }
        }

        // Handle search issues for linking request - spawn in background
        if let Some((issue_key, query)) = app.take_pending_search_issues_for_link() {
            if let Some(ref c) = client {
                debug!("Searching issues for linking: {}", query);
                task_spawner.spawn_search_issues_for_link(c, query, issue_key);
            } else {
                app.handle_issue_search_failure("No JIRA connection");
            }
        }

        // Handle create link request - spawn in background
        if let Some((current_key, target_key, link_type_name, is_outward)) =
            app.take_pending_create_link()
        {
            if let Some(ref c) = client {
                debug!(
                    "Creating link: {} -> {} (type: {}, outward: {})",
                    current_key, target_key, link_type_name, is_outward
                );
                task_spawner.spawn_create_link(
                    c,
                    current_key,
                    target_key,
                    link_type_name,
                    is_outward,
                );
            } else {
                app.handle_create_link_failure("No JIRA connection");
            }
        }

        // Handle delete link request (after confirmation) - spawn in background
        if let Some((link_id, issue_key)) = app.take_pending_delete_link() {
            if let Some(ref c) = client {
                debug!("Deleting link: {}", link_id);
                task_spawner.spawn_delete_link(c, link_id, issue_key);
            } else {
                app.handle_delete_link_failure("No JIRA connection");
            }
        }

        // Handle pending create issue request - spawn in background
        if let Some(request) = app.take_pending_create_issue() {
            if let Some(ref c) = client {
                debug!("Creating issue in project: {}", request.fields.project.key);
                task_spawner.spawn_create_issue(c, request);
            } else {
                app.handle_create_issue_failure("No JIRA connection");
            }
        }

        // Handle pending fetch issue types request - spawn in background
        if let Some(project_key) = app.take_pending_fetch_issue_types() {
            if let Some(ref c) = client {
                debug!("Fetching issue types for project: {}", project_key);
                task_spawner.spawn_fetch_issue_types(c, project_key);
            } else {
                app.handle_fetch_issue_types_failure("No JIRA connection");
            }
        }

        // Check if we should quit
        if app.should_quit() {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that suspend_tui works with a mock stdout.
    /// Note: This test verifies the function compiles and can be called,
    /// but actual terminal state changes require manual testing.
    #[test]
    fn test_suspend_tui_compiles() {
        // The function is generic over W: io::Write, so we can test compilation
        // with a Vec<u8> as a mock stdout
        let mut mock_stdout: Vec<u8> = Vec::new();

        // We can't actually test terminal state changes without a real terminal,
        // but we can verify the function signature and structure compile correctly.
        // The disable_raw_mode() will fail in tests (no terminal), but the generic
        // constraint allows us to verify the execute! macro works with our type.

        // This is a compile-time test primarily - runtime requires a real terminal
        let _ = suspend_tui(&mut mock_stdout);
    }

    /// Test that resume_tui requires both stdout and terminal references.
    /// This is primarily a compile-time verification test.
    #[test]
    fn test_resume_tui_signature() {
        // This test verifies the function signature is correct.
        // Actual terminal operations require manual testing.

        // The function should accept a mutable stdout reference and a terminal reference.
        // We can't easily create a mock Terminal<CrosstermBackend<io::Stdout>> in tests,
        // so this test primarily documents the expected signature.

        // Compile-time check: the function exists and has the expected signature
        fn _assert_resume_tui_exists<W: io::Write>(
            _stdout: &mut W,
            _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        ) -> Result<()> {
            resume_tui(_stdout, _terminal)
        }
    }

    /// Test that TuiSuspendGuard has the expected structure.
    #[test]
    fn test_tui_suspend_guard_structure() {
        // This test verifies the guard struct has the expected lifetime and field types.
        // Actual RAII behavior requires manual testing with a real terminal.

        // The guard should implement Drop, which we verify by checking it exists as a type
        fn _assert_guard_has_expected_structure<'a>(
            _stdout: &'a mut io::Stdout,
            _terminal: &'a mut Terminal<CrosstermBackend<io::Stdout>>,
        ) {
            // This block compiles only if the guard can be constructed with these types
            // (in a real terminal environment)
        }

        // Verify the struct type exists and can be referenced with proper lifetime constraints
        fn _create_guard<'a>(
            stdout: &'a mut io::Stdout,
            terminal: &'a mut Terminal<CrosstermBackend<io::Stdout>>,
        ) -> Result<TuiSuspendGuard<'a>> {
            TuiSuspendGuard::new(stdout, terminal)
        }
    }

    /// Test that the guard implements Drop trait (compile-time verification).
    #[test]
    fn test_tui_suspend_guard_implements_drop() {
        // This test verifies that TuiSuspendGuard implements Drop.
        // The Drop implementation is critical for ensuring terminal restoration
        // even if an editor crashes or panics.

        fn _assert_drop_impl<T: Drop>() {}

        // This will fail to compile if TuiSuspendGuard doesn't implement Drop
        _assert_drop_impl::<TuiSuspendGuard<'_>>();
    }
}
