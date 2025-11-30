//! LazyJira - A terminal-based user interface for JIRA
//!
//! This application provides a TUI for managing JIRA issues directly from the terminal.

mod api;
mod app;
mod cache;
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
use events::EventHandler;

/// Application result type.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging first (before any other operations)
    if let Err(e) = logging::init() {
        eprintln!("Warning: Failed to initialize logging: {}", e);
        // Continue without logging rather than failing completely
    }

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
    let mut app = App::new();
    let event_handler = EventHandler::new();

    loop {
        // Render the current view (View in TEA)
        terminal.draw(|frame| app.view(frame))?;

        // Wait for and handle events (Update in TEA)
        let event = event_handler.next()?;
        app.update(event);

        // Check if we should quit
        if app.should_quit() {
            break;
        }
    }

    Ok(())
}
