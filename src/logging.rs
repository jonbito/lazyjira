//! Logging configuration using the tracing ecosystem.
//!
//! This module configures structured logging with:
//! - File-based output (to avoid TUI corruption)
//! - Daily log rotation
//! - Environment-based log level configuration
//! - Span-based context for async operations

use std::path::PathBuf;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};

/// Default log level if RUST_LOG is not set.
const DEFAULT_LOG_FILTER: &str = "lazyjira=info,warn";

/// Initialize the logging system.
///
/// Sets up tracing with:
/// - Daily rotating file appender in the user's local data directory
/// - Log level configuration via `RUST_LOG` environment variable
/// - Structured output with file/line numbers and thread IDs
///
/// # Log Directory
///
/// Logs are stored in the platform-specific local data directory:
/// - Linux: `~/.local/share/lazyjira/logs/`
/// - macOS: `~/Library/Application Support/lazyjira/logs/`
/// - Windows: `C:\Users\<User>\AppData\Local\lazyjira\logs\`
///
/// # Log Levels
///
/// Configure via `RUST_LOG` environment variable:
/// - `RUST_LOG=debug` - Verbose output for debugging
/// - `RUST_LOG=lazyjira=debug` - Debug only for lazyjira
/// - `RUST_LOG=lazyjira=trace` - Very verbose, frame-by-frame details
///
/// # Errors
///
/// Returns an error if:
/// - The log directory cannot be determined or created
/// - The tracing subscriber cannot be set
///
/// # Example
///
/// ```no_run
/// use lazyjira::logging;
///
/// // Initialize with default settings
/// logging::init().expect("Failed to initialize logging");
///
/// // Or with debug mode
/// std::env::set_var("RUST_LOG", "lazyjira=debug");
/// logging::init().expect("Failed to initialize logging");
/// ```
pub fn init() -> anyhow::Result<()> {
    let log_dir = get_log_directory()?;
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "lazyjira.log");

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER));

    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        )
        .with(filter);

    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "LazyJira starting up");
    tracing::debug!(log_dir = %log_dir.display(), "Log directory");

    Ok(())
}

/// Get the log directory path.
///
/// Returns the platform-specific local data directory with `lazyjira/logs` appended.
fn get_log_directory() -> anyhow::Result<PathBuf> {
    let base_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine local data directory"))?;

    Ok(base_dir.join("lazyjira").join("logs"))
}

/// Get the path where logs are stored.
///
/// This is useful for displaying to users where they can find log files.
pub fn log_directory() -> Option<PathBuf> {
    get_log_directory().ok()
}

/// Log application shutdown.
///
/// Call this before the application exits to log a clean shutdown message.
pub fn shutdown() {
    tracing::info!("LazyJira shutting down");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_directory_has_expected_structure() {
        let dir = get_log_directory().unwrap();
        assert!(dir.ends_with("lazyjira/logs"));
    }

    #[test]
    fn test_log_directory_public_function() {
        let dir = log_directory();
        assert!(dir.is_some());
        assert!(dir.unwrap().ends_with("lazyjira/logs"));
    }
}
