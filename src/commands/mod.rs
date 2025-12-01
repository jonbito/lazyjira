//! Command system for the command palette.
//!
//! This module provides:
//! - Command definitions with categories
//! - Command registry with fuzzy search
//! - Recent commands tracking

mod registry;

pub use registry::{Command, CommandAction, CommandCategory, CommandRegistry};
