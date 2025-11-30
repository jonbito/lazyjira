//! Event handling for the application.
//!
//! This module handles keyboard input, API responses, and other events.

mod handler;
mod keys;

pub use handler::EventHandler;
pub use keys::KeyBindings;
