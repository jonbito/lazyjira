//! Event handler implementation.
//!
//! Polls for terminal events and converts them to application events.

use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent};

use super::Event;

/// The tick rate for the event loop in milliseconds.
const TICK_RATE_MS: u64 = 100;

/// Handles application events by polling crossterm for terminal events.
pub struct EventHandler {
    /// The tick rate duration.
    tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler with the default tick rate.
    pub fn new() -> Self {
        Self {
            tick_rate: Duration::from_millis(TICK_RATE_MS),
        }
    }

    /// Create a new event handler with a custom tick rate.
    pub fn with_tick_rate(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    /// Poll for the next event.
    ///
    /// This method blocks until an event is available or the tick rate elapses.
    /// Returns `Event::Tick` if no event occurred within the tick rate.
    pub fn next(&self) -> std::io::Result<Event> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key_event) => Ok(Event::Key(key_event)),
                CrosstermEvent::Resize(width, height) => Ok(Event::Resize(width, height)),
                // Mouse events are not currently handled
                CrosstermEvent::Mouse(_) => Ok(Event::Tick),
                // Focus events are not currently handled
                CrosstermEvent::FocusGained | CrosstermEvent::FocusLost => Ok(Event::Tick),
                // Paste events are not currently handled
                CrosstermEvent::Paste(_) => Ok(Event::Tick),
            }
        } else {
            // No event within tick rate, return tick event
            Ok(Event::Tick)
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_handler_new() {
        let handler = EventHandler::new();
        assert_eq!(handler.tick_rate, Duration::from_millis(TICK_RATE_MS));
    }

    #[test]
    fn test_event_handler_with_tick_rate() {
        let handler = EventHandler::with_tick_rate(50);
        assert_eq!(handler.tick_rate, Duration::from_millis(50));
    }

    #[test]
    fn test_event_handler_default() {
        let handler = EventHandler::default();
        assert_eq!(handler.tick_rate, Duration::from_millis(TICK_RATE_MS));
    }
}
