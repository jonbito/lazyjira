//! Event handling for the application.
//!
//! This module handles keyboard input, terminal events, and application events.

mod handler;
mod keys;

pub use handler::EventHandler;
pub use keys::{
    get_context_hints, get_keybindings, get_keybindings_for_context, get_keybindings_grouped,
    KeyBindings, Keybinding, KeyContext,
};

use crossterm::event::KeyEvent;

/// Application events that can be processed by the App.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A keyboard key was pressed.
    Key(KeyEvent),
    /// The terminal was resized.
    Resize(u16, u16),
    /// A periodic tick for background processing and animations.
    Tick,
    /// Request to quit the application.
    Quit,
}
