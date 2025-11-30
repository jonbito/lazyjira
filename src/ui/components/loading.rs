//! Loading indicator component.
//!
//! This module provides loading indicators for async operations,
//! with animated spinners to show the user that work is in progress.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

/// Spinner animation frames.
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Alternative spinner using simple characters for compatibility.
const SIMPLE_SPINNER_FRAMES: &[&str] = &["|", "/", "-", "\\"];

/// Dots spinner animation.
const DOTS_SPINNER_FRAMES: &[&str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

/// The type of spinner to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpinnerStyle {
    /// Braille dots spinner (default).
    #[default]
    Braille,
    /// Simple ASCII spinner for compatibility.
    Simple,
    /// Dots spinner.
    Dots,
}

impl SpinnerStyle {
    /// Get the frames for this spinner style.
    pub fn frames(&self) -> &'static [&'static str] {
        match self {
            SpinnerStyle::Braille => SPINNER_FRAMES,
            SpinnerStyle::Simple => SIMPLE_SPINNER_FRAMES,
            SpinnerStyle::Dots => DOTS_SPINNER_FRAMES,
        }
    }
}

/// A loading indicator with an animated spinner.
#[derive(Debug, Clone)]
pub struct LoadingIndicator {
    /// The message to display.
    message: String,
    /// Current spinner frame index.
    spinner_state: usize,
    /// The spinner style.
    spinner_style: SpinnerStyle,
    /// Whether the loading indicator is active.
    active: bool,
}

impl Default for LoadingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadingIndicator {
    /// Create a new loading indicator.
    pub fn new() -> Self {
        Self {
            message: "Loading...".to_string(),
            spinner_state: 0,
            spinner_style: SpinnerStyle::default(),
            active: false,
        }
    }

    /// Create a loading indicator with a custom message.
    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            spinner_state: 0,
            spinner_style: SpinnerStyle::default(),
            active: false,
        }
    }

    /// Set the spinner style.
    pub fn with_style(mut self, style: SpinnerStyle) -> Self {
        self.spinner_style = style;
        self
    }

    /// Set the message.
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// Get the current message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Start the loading indicator.
    pub fn start(&mut self) {
        self.active = true;
        self.spinner_state = 0;
    }

    /// Start with a specific message.
    pub fn start_with_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
        self.start();
    }

    /// Stop the loading indicator.
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Check if the loading indicator is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Advance the spinner animation.
    ///
    /// This should be called on each tick event.
    pub fn tick(&mut self) {
        if self.active {
            let frames = self.spinner_style.frames();
            self.spinner_state = (self.spinner_state + 1) % frames.len();
        }
    }

    /// Get the current spinner frame.
    pub fn spinner_frame(&self) -> &'static str {
        let frames = self.spinner_style.frames();
        frames[self.spinner_state]
    }

    /// Render the loading indicator centered in the given area.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.active {
            return;
        }

        let spinner = self.spinner_frame();
        let text = format!("{} {}", spinner, self.message);
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    /// Render the loading indicator left-aligned in the given area.
    pub fn render_left(&self, frame: &mut Frame, area: Rect) {
        if !self.active {
            return;
        }

        let spinner = self.spinner_frame();
        let text = format!("{} {}", spinner, self.message);
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    /// Get the formatted loading text (for embedding in other widgets).
    pub fn text(&self) -> String {
        if self.active {
            format!("{} {}", self.spinner_frame(), self.message)
        } else {
            String::new()
        }
    }
}

/// A simple inline loading indicator for status bars.
#[derive(Debug, Clone, Default)]
pub struct InlineLoader {
    /// Current spinner frame index.
    spinner_state: usize,
    /// Whether the loader is active.
    active: bool,
}

impl InlineLoader {
    /// Create a new inline loader.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start the loader.
    pub fn start(&mut self) {
        self.active = true;
    }

    /// Stop the loader.
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Check if active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Advance the animation.
    pub fn tick(&mut self) {
        if self.active {
            self.spinner_state = (self.spinner_state + 1) % SPINNER_FRAMES.len();
        }
    }

    /// Get the current frame, or an empty string if not active.
    pub fn frame(&self) -> &'static str {
        if self.active {
            SPINNER_FRAMES[self.spinner_state]
        } else {
            " "
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_style_frames() {
        assert_eq!(SpinnerStyle::Braille.frames().len(), 10);
        assert_eq!(SpinnerStyle::Simple.frames().len(), 4);
        assert_eq!(SpinnerStyle::Dots.frames().len(), 8);
    }

    #[test]
    fn test_loading_indicator_new() {
        let loader = LoadingIndicator::new();
        assert_eq!(loader.message(), "Loading...");
        assert!(!loader.is_active());
    }

    #[test]
    fn test_loading_indicator_with_message() {
        let loader = LoadingIndicator::with_message("Fetching issues...");
        assert_eq!(loader.message(), "Fetching issues...");
    }

    #[test]
    fn test_loading_indicator_with_style() {
        let loader = LoadingIndicator::new().with_style(SpinnerStyle::Simple);
        assert_eq!(loader.spinner_style, SpinnerStyle::Simple);
    }

    #[test]
    fn test_loading_indicator_start_stop() {
        let mut loader = LoadingIndicator::new();
        assert!(!loader.is_active());

        loader.start();
        assert!(loader.is_active());

        loader.stop();
        assert!(!loader.is_active());
    }

    #[test]
    fn test_loading_indicator_start_with_message() {
        let mut loader = LoadingIndicator::new();
        loader.start_with_message("Custom message");
        assert!(loader.is_active());
        assert_eq!(loader.message(), "Custom message");
    }

    #[test]
    fn test_loading_indicator_tick() {
        let mut loader = LoadingIndicator::new();
        loader.start();

        let initial_frame = loader.spinner_frame();
        loader.tick();
        let next_frame = loader.spinner_frame();

        // Should advance to next frame
        assert_ne!(initial_frame, next_frame);
    }

    #[test]
    fn test_loading_indicator_tick_inactive() {
        let mut loader = LoadingIndicator::new();
        let initial_state = loader.spinner_state;
        loader.tick();
        // Should not advance when inactive
        assert_eq!(loader.spinner_state, initial_state);
    }

    #[test]
    fn test_loading_indicator_tick_wraps() {
        let mut loader = LoadingIndicator::new();
        loader.start();

        // Tick through all frames
        for _ in 0..SPINNER_FRAMES.len() {
            loader.tick();
        }

        // Should wrap back to first frame
        assert_eq!(loader.spinner_state, 0);
    }

    #[test]
    fn test_loading_indicator_text() {
        let mut loader = LoadingIndicator::new();
        loader.start();
        let text = loader.text();
        assert!(text.contains("Loading..."));
    }

    #[test]
    fn test_loading_indicator_text_inactive() {
        let loader = LoadingIndicator::new();
        let text = loader.text();
        assert!(text.is_empty());
    }

    #[test]
    fn test_inline_loader_new() {
        let loader = InlineLoader::new();
        assert!(!loader.is_active());
    }

    #[test]
    fn test_inline_loader_start_stop() {
        let mut loader = InlineLoader::new();
        loader.start();
        assert!(loader.is_active());

        loader.stop();
        assert!(!loader.is_active());
    }

    #[test]
    fn test_inline_loader_tick() {
        let mut loader = InlineLoader::new();
        loader.start();

        let initial_state = loader.spinner_state;
        loader.tick();
        assert_ne!(loader.spinner_state, initial_state);
    }

    #[test]
    fn test_inline_loader_frame() {
        let mut loader = InlineLoader::new();
        assert_eq!(loader.frame(), " "); // Inactive

        loader.start();
        let frame = loader.frame();
        assert!(SPINNER_FRAMES.contains(&frame));
    }
}
