# Task 4.4: Issue History and Changelog

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 4.4
**Area:** Frontend/UI
**Estimated Effort:** M (4-6 hours)

## Description

Implement a view for displaying issue history and changelog, showing all changes made to an issue over time including field changes, status transitions, and user actions.

## Acceptance Criteria

- [ ] History view accessible from issue detail ('h' key)
- [ ] Display all field changes with before/after values
- [ ] Show user who made the change
- [ ] Display timestamp for each change
- [ ] Group changes by date
- [ ] Scrollable list for long histories
- [ ] Visual indicators for different change types
- [ ] Return to detail view with 'q' or Escape

## Implementation Details

### Approach

1. Add changelog API endpoint
2. Parse JIRA changelog format
3. Create history view component
4. Format changes for display
5. Add navigation from detail view

### Files to Modify/Create

- `src/ui/views/history.rs`: History view
- `src/api/client.rs`: Changelog API
- `src/api/types.rs`: Changelog types

### Technical Specifications

**Changelog Types:**
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Changelog {
    pub histories: Vec<ChangeHistory>,
    #[serde(rename = "startAt")]
    pub start_at: u32,
    #[serde(rename = "maxResults")]
    pub max_results: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChangeHistory {
    pub id: String,
    pub author: User,
    pub created: String,
    pub items: Vec<ChangeItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChangeItem {
    pub field: String,
    #[serde(rename = "fieldtype")]
    pub field_type: String,
    #[serde(rename = "from")]
    pub from_value: Option<String>,
    #[serde(rename = "fromString")]
    pub from_string: Option<String>,
    #[serde(rename = "to")]
    pub to_value: Option<String>,
    #[serde(rename = "toString")]
    pub to_string: Option<String>,
}

impl ChangeItem {
    pub fn display_from(&self) -> &str {
        self.from_string.as_deref()
            .or(self.from_value.as_deref())
            .unwrap_or("(none)")
    }

    pub fn display_to(&self) -> &str {
        self.to_string.as_deref()
            .or(self.to_value.as_deref())
            .unwrap_or("(none)")
    }

    pub fn change_type(&self) -> ChangeType {
        match self.field.to_lowercase().as_str() {
            "status" => ChangeType::Status,
            "assignee" => ChangeType::Assignee,
            "priority" => ChangeType::Priority,
            "summary" | "description" => ChangeType::Content,
            "labels" | "component" => ChangeType::Tags,
            "sprint" => ChangeType::Sprint,
            _ => ChangeType::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChangeType {
    Status,
    Assignee,
    Priority,
    Content,
    Tags,
    Sprint,
    Other,
}

impl ChangeType {
    fn icon(&self) -> &'static str {
        match self {
            Self::Status => "◉",
            Self::Assignee => "👤",
            Self::Priority => "⬆",
            Self::Content => "✎",
            Self::Tags => "🏷",
            Self::Sprint => "🏃",
            Self::Other => "•",
        }
    }

    fn color(&self, theme: &Theme) -> Color {
        match self {
            Self::Status => theme.status_in_progress,
            Self::Assignee => theme.accent,
            Self::Priority => theme.warning,
            Self::Content => theme.info,
            Self::Tags => theme.success,
            Self::Sprint => theme.accent,
            Self::Other => theme.foreground_dim,
        }
    }
}
```

**API Method:**
```rust
impl JiraClient {
    pub async fn get_changelog(
        &self,
        issue_key: &str,
        start: u32,
        max: u32,
    ) -> Result<Changelog> {
        let url = format!(
            "{}/rest/api/3/issue/{}/changelog?startAt={}&maxResults={}",
            self.base_url, issue_key, start, max
        );
        self.get(&url).await
    }
}
```

**History View:**
```rust
pub struct HistoryView {
    issue_key: String,
    histories: Vec<ChangeHistory>,
    scroll: usize,
    loading: bool,
    pagination: HistoryPagination,
}

struct HistoryPagination {
    total: u32,
    loaded: u32,
    has_more: bool,
}

impl HistoryView {
    pub fn new(issue_key: &str) -> Self {
        Self {
            issue_key: issue_key.to_string(),
            histories: Vec::new(),
            scroll: 0,
            loading: true,
            pagination: HistoryPagination {
                total: 0,
                loaded: 0,
                has_more: true,
            },
        }
    }

    pub fn set_changelog(&mut self, changelog: Changelog) {
        if changelog.start_at == 0 {
            self.histories = changelog.histories;
        } else {
            self.histories.extend(changelog.histories);
        }
        self.pagination.total = changelog.total;
        self.pagination.loaded = self.histories.len() as u32;
        self.pagination.has_more = self.pagination.loaded < changelog.total;
        self.loading = false;
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<HistoryAction> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                Some(HistoryAction::Close)
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_down();
                self.check_load_more()
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_up();
                None
            }
            KeyCode::Char('g') => {
                self.scroll = 0;
                None
            }
            KeyCode::Char('G') => {
                self.scroll_to_end();
                self.check_load_more()
            }
            _ => None,
        }
    }

    fn check_load_more(&mut self) -> Option<HistoryAction> {
        if self.scroll + 5 >= self.histories.len()
            && self.pagination.has_more
            && !self.loading
        {
            self.loading = true;
            Some(HistoryAction::LoadMore(self.pagination.loaded))
        } else {
            None
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(format!("History - {}", self.issue_key))
            .borders(Borders::ALL);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.loading && self.histories.is_empty() {
            let loading = Paragraph::new("Loading history...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(loading, inner);
            return;
        }

        if self.histories.is_empty() {
            let empty = Paragraph::new("No history available")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(empty, inner);
            return;
        }

        // Group histories by date
        let grouped = self.group_by_date();

        let mut lines: Vec<Line> = Vec::new();

        for (date, histories) in grouped.iter().skip(self.scroll) {
            // Date header
            lines.push(Line::from(vec![
                Span::styled(
                    format!("── {} ──", date),
                    Style::default().bold().fg(Color::Yellow),
                ),
            ]));

            for history in histories {
                // Author and time
                let time = format_time(&history.created);
                lines.push(Line::from(vec![
                    Span::styled(&history.author.display_name, Style::default().bold()),
                    Span::raw(" at "),
                    Span::styled(&time, Style::default().fg(Color::Gray)),
                ]));

                // Changes
                for item in &history.items {
                    let icon = item.change_type().icon();
                    let color = item.change_type().color(theme());

                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(icon, Style::default().fg(color)),
                        Span::raw(" "),
                        Span::styled(&item.field, Style::default().bold()),
                        Span::raw(": "),
                        Span::styled(item.display_from(), Style::default().fg(Color::Red)),
                        Span::raw(" → "),
                        Span::styled(item.display_to(), Style::default().fg(Color::Green)),
                    ]));
                }

                lines.push(Line::from(""));
            }
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);

        // Scroll indicator
        if self.loading {
            let loading = Paragraph::new("Loading more...")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Right);
            // Render at bottom
        }
    }

    fn group_by_date(&self) -> Vec<(String, Vec<&ChangeHistory>)> {
        let mut groups: std::collections::HashMap<String, Vec<&ChangeHistory>> =
            std::collections::HashMap::new();

        for history in &self.histories {
            let date = history.created.split('T').next()
                .unwrap_or(&history.created)
                .to_string();
            groups.entry(date).or_default().push(history);
        }

        let mut sorted: Vec<_> = groups.into_iter().collect();
        sorted.sort_by(|a, b| b.0.cmp(&a.0)); // Newest first
        sorted
    }
}

fn format_time(iso: &str) -> String {
    // "2024-01-15T10:30:00.000+0000" -> "10:30"
    iso.split('T')
        .nth(1)
        .and_then(|t| t.split('.').next())
        .and_then(|t| t.rsplit_once(':').map(|(h, _)| h))
        .unwrap_or(iso)
        .to_string()
}

pub enum HistoryAction {
    Close,
    LoadMore(u32),
}
```

## Testing Requirements

- [ ] 'h' opens history view
- [ ] History loads for issue
- [ ] Changes displayed correctly
- [ ] User and timestamp shown
- [ ] Scrolling works
- [ ] Pagination loads more
- [ ] 'q'/Esc returns to detail
- [ ] Different change types styled

## Dependencies

- **Prerequisite Tasks:** Task 1.7, Task 4.2 (theme)
- **Blocks Tasks:** None
- **External:** JIRA changelog API

## Definition of Done

- [ ] All acceptance criteria met
- [ ] History is readable
- [ ] All change types displayed
- [ ] Pagination works smoothly
- [ ] Visual indicators clear
