//! Notification/toast component for user feedback.
//!
//! This module provides a toast notification system for displaying
//! transient messages to users (success, error, info, warning).

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// The type of notification, which determines its appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    /// Informational message (blue).
    Info,
    /// Success message (green).
    Success,
    /// Warning message (yellow).
    Warning,
    /// Error message (red).
    Error,
}

impl NotificationType {
    /// Get the icon for this notification type.
    pub fn icon(&self) -> &'static str {
        match self {
            NotificationType::Info => "ℹ",
            NotificationType::Success => "✓",
            NotificationType::Warning => "⚠",
            NotificationType::Error => "✗",
        }
    }

    /// Get the color for this notification type.
    pub fn color(&self) -> Color {
        match self {
            NotificationType::Info => Color::Blue,
            NotificationType::Success => Color::Green,
            NotificationType::Warning => Color::Yellow,
            NotificationType::Error => Color::Red,
        }
    }

    /// Get the style for this notification type.
    pub fn style(&self) -> Style {
        Style::default().fg(self.color())
    }

    /// Get the border style for this notification type.
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.color())
    }
}

/// A single notification message.
#[derive(Debug, Clone)]
pub struct Notification {
    /// The notification message.
    pub message: String,
    /// The type of notification.
    pub notification_type: NotificationType,
    /// When the notification was created.
    pub created_at: Instant,
    /// How long the notification should be displayed.
    pub duration: Duration,
}

impl Notification {
    /// Create a new notification.
    pub fn new(
        message: impl Into<String>,
        notification_type: NotificationType,
        duration: Duration,
    ) -> Self {
        Self {
            message: message.into(),
            notification_type,
            created_at: Instant::now(),
            duration,
        }
    }

    /// Create an info notification with default duration (3 seconds).
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Info, Duration::from_secs(3))
    }

    /// Create a success notification with default duration (3 seconds).
    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Success, Duration::from_secs(3))
    }

    /// Create a warning notification with default duration (5 seconds).
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Warning, Duration::from_secs(5))
    }

    /// Create an error notification with default duration (5 seconds).
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Error, Duration::from_secs(5))
    }

    /// Create a notification with a custom duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Check if the notification has expired.
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }

    /// Get the remaining time before expiration.
    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.created_at.elapsed())
    }

    /// Calculate the progress (0.0 to 1.0) of the notification's lifetime.
    pub fn progress(&self) -> f64 {
        let elapsed = self.created_at.elapsed().as_secs_f64();
        let total = self.duration.as_secs_f64();
        (elapsed / total).min(1.0)
    }
}

/// Manages multiple notifications.
#[derive(Debug)]
pub struct NotificationManager {
    /// Queue of notifications.
    notifications: VecDeque<Notification>,
    /// Maximum number of visible notifications.
    max_visible: usize,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationManager {
    /// Create a new notification manager.
    pub fn new() -> Self {
        Self {
            notifications: VecDeque::new(),
            max_visible: 3,
        }
    }

    /// Create a notification manager with a custom max visible count.
    pub fn with_max_visible(max_visible: usize) -> Self {
        Self {
            notifications: VecDeque::new(),
            max_visible,
        }
    }

    /// Add a notification to the queue.
    pub fn push(&mut self, notification: Notification) {
        self.notifications.push_back(notification);
        // Remove oldest if we exceed the limit
        while self.notifications.len() > self.max_visible {
            self.notifications.pop_front();
        }
    }

    /// Add an info notification.
    pub fn info(&mut self, message: impl Into<String>) {
        self.push(Notification::info(message));
    }

    /// Add a success notification.
    pub fn success(&mut self, message: impl Into<String>) {
        self.push(Notification::success(message));
    }

    /// Add a warning notification.
    pub fn warning(&mut self, message: impl Into<String>) {
        self.push(Notification::warning(message));
    }

    /// Add an error notification.
    pub fn error(&mut self, message: impl Into<String>) {
        self.push(Notification::error(message));
    }

    /// Remove expired notifications.
    ///
    /// This should be called on each tick/render cycle.
    pub fn tick(&mut self) {
        self.notifications.retain(|n| !n.is_expired());
    }

    /// Clear all notifications.
    pub fn clear(&mut self) {
        self.notifications.clear();
    }

    /// Check if there are any notifications.
    pub fn is_empty(&self) -> bool {
        self.notifications.is_empty()
    }

    /// Get the number of notifications.
    pub fn len(&self) -> usize {
        self.notifications.len()
    }

    /// Get an iterator over the notifications.
    pub fn iter(&self) -> impl Iterator<Item = &Notification> {
        self.notifications.iter()
    }

    /// Render all notifications in the bottom-right corner of the given area.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if self.notifications.is_empty() {
            return;
        }

        // Calculate notification area in bottom-right corner
        let notification_width = 50.min(area.width.saturating_sub(4));
        // Inner width accounts for borders (1 char each side) and icon prefix (2 chars for icon + space)
        let inner_width = notification_width.saturating_sub(4) as usize;

        // Calculate height for each notification based on text wrapping
        let notification_heights: Vec<u16> = self
            .notifications
            .iter()
            .map(|n| {
                let text_len = n.message.len() + 2; // +2 for icon and space
                let lines_needed = if inner_width > 0 {
                    ((text_len + inner_width - 1) / inner_width) as u16 // Ceiling division
                } else {
                    1
                };
                lines_needed + 2 // Add 2 for top and bottom borders
            })
            .collect();

        let total_height = notification_heights
            .iter()
            .sum::<u16>()
            .min(area.height.saturating_sub(2));

        let x = area.x + area.width.saturating_sub(notification_width + 2);
        let y = area.y + area.height.saturating_sub(total_height + 1);

        let notifications_area = Rect::new(x, y, notification_width, total_height);

        // Split into individual notification areas with calculated heights
        let constraints: Vec<Constraint> = notification_heights
            .iter()
            .map(|&h| Constraint::Length(h))
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(notifications_area);

        for (notification, chunk) in self.notifications.iter().zip(chunks.iter()) {
            render_notification(notification, frame, *chunk);
        }
    }
}

/// Render a single notification.
fn render_notification(notification: &Notification, frame: &mut Frame, area: Rect) {
    // Clear the area first
    frame.render_widget(Clear, area);

    let style = notification.notification_type.style();
    let border_style = notification.notification_type.border_style();
    let icon = notification.notification_type.icon();

    // Create the notification text with icon
    let text = Line::from(vec![
        Span::styled(format!("{} ", icon), style.add_modifier(Modifier::BOLD)),
        Span::styled(&notification.message, style),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_type_icon() {
        assert_eq!(NotificationType::Info.icon(), "ℹ");
        assert_eq!(NotificationType::Success.icon(), "✓");
        assert_eq!(NotificationType::Warning.icon(), "⚠");
        assert_eq!(NotificationType::Error.icon(), "✗");
    }

    #[test]
    fn test_notification_type_color() {
        assert_eq!(NotificationType::Info.color(), Color::Blue);
        assert_eq!(NotificationType::Success.color(), Color::Green);
        assert_eq!(NotificationType::Warning.color(), Color::Yellow);
        assert_eq!(NotificationType::Error.color(), Color::Red);
    }

    #[test]
    fn test_notification_info() {
        let n = Notification::info("Test message");
        assert_eq!(n.message, "Test message");
        assert_eq!(n.notification_type, NotificationType::Info);
        assert_eq!(n.duration, Duration::from_secs(3));
    }

    #[test]
    fn test_notification_success() {
        let n = Notification::success("Success!");
        assert_eq!(n.notification_type, NotificationType::Success);
        assert_eq!(n.duration, Duration::from_secs(3));
    }

    #[test]
    fn test_notification_warning() {
        let n = Notification::warning("Warning!");
        assert_eq!(n.notification_type, NotificationType::Warning);
        assert_eq!(n.duration, Duration::from_secs(5));
    }

    #[test]
    fn test_notification_error() {
        let n = Notification::error("Error!");
        assert_eq!(n.notification_type, NotificationType::Error);
        assert_eq!(n.duration, Duration::from_secs(5));
    }

    #[test]
    fn test_notification_with_duration() {
        let n = Notification::info("Test").with_duration(Duration::from_secs(10));
        assert_eq!(n.duration, Duration::from_secs(10));
    }

    #[test]
    fn test_notification_is_expired() {
        let n = Notification::new("Test", NotificationType::Info, Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(5));
        assert!(n.is_expired());
    }

    #[test]
    fn test_notification_not_expired() {
        let n = Notification::info("Test");
        assert!(!n.is_expired());
    }

    #[test]
    fn test_notification_progress() {
        let n = Notification::new("Test", NotificationType::Info, Duration::from_millis(100));
        let progress = n.progress();
        assert!(progress >= 0.0 && progress <= 1.0);
    }

    #[test]
    fn test_notification_manager_new() {
        let manager = NotificationManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_notification_manager_push() {
        let mut manager = NotificationManager::new();
        manager.push(Notification::info("Test"));
        assert!(!manager.is_empty());
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_notification_manager_convenience_methods() {
        let mut manager = NotificationManager::new();
        manager.info("Info");
        manager.success("Success");
        manager.warning("Warning");
        // Only 3 max visible, so adding 4th should remove first
        assert_eq!(manager.len(), 3);
    }

    #[test]
    fn test_notification_manager_max_visible() {
        let mut manager = NotificationManager::with_max_visible(2);
        manager.push(Notification::info("1"));
        manager.push(Notification::info("2"));
        manager.push(Notification::info("3"));
        assert_eq!(manager.len(), 2);
    }

    #[test]
    fn test_notification_manager_tick() {
        let mut manager = NotificationManager::new();
        manager.push(Notification::new(
            "Expires",
            NotificationType::Info,
            Duration::from_millis(1),
        ));
        std::thread::sleep(Duration::from_millis(5));
        manager.tick();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_notification_manager_clear() {
        let mut manager = NotificationManager::new();
        manager.info("Test");
        manager.clear();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_notification_manager_iter() {
        let mut manager = NotificationManager::new();
        manager.info("Test 1");
        manager.info("Test 2");
        let messages: Vec<&str> = manager.iter().map(|n| n.message.as_str()).collect();
        assert_eq!(messages, vec!["Test 1", "Test 2"]);
    }
}
