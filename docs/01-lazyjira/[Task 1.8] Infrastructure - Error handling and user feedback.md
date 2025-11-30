# Task 1.8: Error Handling and User Feedback

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 1.8
**Area:** Infrastructure
**Estimated Effort:** M (4-6 hours)

## Description

Implement comprehensive error handling throughout the application with user-friendly error messages and a notification/toast system for operation feedback.

## Acceptance Criteria

- [ ] Centralized error types using thiserror
- [ ] User-friendly error messages (no raw error dumps)
- [ ] Toast/notification component for transient messages
- [ ] Error dialog for critical errors requiring acknowledgment
- [ ] Loading indicators for async operations
- [ ] Success feedback for completed operations
- [ ] Error context preserved for debugging (via tracing)
- [ ] Graceful degradation for non-critical failures

## Implementation Details

### Approach

1. Define error hierarchy with thiserror
2. Create notification/toast component
3. Implement error dialog modal
4. Add loading state management
5. Integrate error handling with API layer
6. Add tracing spans for debugging

### Files to Modify/Create

- `src/error.rs`: Error types and conversions
- `src/ui/components/notification.rs`: Toast/notification widget
- `src/ui/components/modal.rs`: Modal dialog (for errors)
- `src/ui/components/loading.rs`: Loading indicator
- `src/app.rs`: Integrate notifications and error handling

### Technical Specifications

**Error Hierarchy:**
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Terminal error: {0}")]
    Terminal(String),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found. Run 'lazyjira setup' to create one.")]
    NotFound,

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("No profiles configured. Add a profile with 'lazyjira profile add'.")]
    NoProfiles,

    #[error("Profile '{0}' not found")]
    ProfileNotFound(String),
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Authentication failed. Please check your email and API token.")]
    Unauthorized,

    #[error("Access denied. You don't have permission to access this resource.")]
    Forbidden,

    #[error("Issue '{0}' not found")]
    IssueNotFound(String),

    #[error("Connection failed. Please check your internet connection.")]
    ConnectionFailed,

    #[error("JIRA server error. Please try again later.")]
    ServerError,

    #[error("Request timed out. The server took too long to respond.")]
    Timeout,
}
```

**Notification System:**
```rust
#[derive(Debug, Clone)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Notification {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Info,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
        }
    }

    pub fn success(message: impl Into<String>) -> Self { ... }
    pub fn error(message: impl Into<String>) -> Self { ... }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }
}

pub struct NotificationManager {
    notifications: VecDeque<Notification>,
    max_visible: usize,
}

impl NotificationManager {
    pub fn push(&mut self, notification: Notification) {
        self.notifications.push_back(notification);
        if self.notifications.len() > self.max_visible {
            self.notifications.pop_front();
        }
    }

    pub fn tick(&mut self) {
        self.notifications.retain(|n| !n.is_expired());
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Render notifications stacked in corner
    }
}
```

**Toast Rendering:**
```rust
fn render_notification(notification: &Notification, frame: &mut Frame, area: Rect) {
    let (icon, style) = match notification.notification_type {
        NotificationType::Info => ("ℹ", Style::default().fg(Color::Blue)),
        NotificationType::Success => ("✓", Style::default().fg(Color::Green)),
        NotificationType::Warning => ("⚠", Style::default().fg(Color::Yellow)),
        NotificationType::Error => ("✗", Style::default().fg(Color::Red)),
    };

    let text = format!("{} {}", icon, notification.message);
    let paragraph = Paragraph::new(text)
        .style(style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(style));

    frame.render_widget(paragraph, area);
}
```

**Error Dialog:**
```rust
pub struct ErrorDialog {
    title: String,
    message: String,
    details: Option<String>,
    visible: bool,
}

impl ErrorDialog {
    pub fn show(error: &AppError) {
        Self {
            title: "Error".to_string(),
            message: error.to_string(),
            details: Some(format!("{:?}", error)),
            visible: true,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let dialog_area = centered_rect(area, 60, 40);
        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(&self.title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));

        let content = Paragraph::new(&self.message)
            .wrap(Wrap { trim: true })
            .block(block);

        frame.render_widget(content, dialog_area);
    }
}
```

**Loading Indicator:**
```rust
pub struct LoadingIndicator {
    message: String,
    spinner_state: usize,
}

impl LoadingIndicator {
    const SPINNER_FRAMES: &'static [&'static str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    pub fn tick(&mut self) {
        self.spinner_state = (self.spinner_state + 1) % Self::SPINNER_FRAMES.len();
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let spinner = Self::SPINNER_FRAMES[self.spinner_state];
        let text = format!("{} {}", spinner, self.message);
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}
```

## Testing Requirements

- [ ] All error types have user-friendly messages
- [ ] Notifications appear and auto-dismiss
- [ ] Error dialog blocks input until dismissed
- [ ] Loading indicator animates
- [ ] API errors converted to user-friendly messages
- [ ] Config errors guide user to fix
- [ ] No stack traces shown to user

## Dependencies

- **Prerequisite Tasks:** Task 1.2 (app architecture)
- **Blocks Tasks:** All other tasks depend on error handling
- **External:** thiserror, tracing

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Error messages are actionable
- [ ] No technical jargon in user-facing errors
- [ ] Errors logged with full context via tracing
- [ ] Notification system integrated with app
