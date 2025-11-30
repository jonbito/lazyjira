# Task 3.5: Comments Viewing and Creation

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 3.5
**Area:** Frontend/UI
**Estimated Effort:** M (6-8 hours)

## Description

Implement comments section in the issue detail view with the ability to view existing comments and add new ones. Comments should display author, timestamp, and formatted content.

## Acceptance Criteria

- [ ] View comments in issue detail view
- [ ] Paginated comment loading for issues with many comments
- [ ] Comment author and timestamp displayed
- [ ] Comment content with basic formatting
- [ ] Add new comment with 'c' key
- [ ] Multi-line comment editor
- [ ] Submit comment with Ctrl+Enter
- [ ] Cancel comment with Escape
- [ ] Success notification after posting
- [ ] Comment appears immediately after posting

## Implementation Details

### Approach

1. Add comments section to detail view
2. Implement comment fetching from API
3. Create comment list component
4. Build comment editor
5. Handle comment submission

### Files to Modify/Create

- `src/ui/views/detail.rs`: Comments section integration
- `src/ui/components/comments.rs`: Comment list and editor
- `src/api/client.rs`: Comment API methods
- `src/api/types.rs`: Comment types

### Technical Specifications

**Comment Types:**
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Comment {
    pub id: String,
    pub body: AtlassianDoc,
    pub author: User,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommentsResponse {
    pub comments: Vec<Comment>,
    #[serde(rename = "startAt")]
    pub start_at: u32,
    #[serde(rename = "maxResults")]
    pub max_results: u32,
    pub total: u32,
}

#[derive(Debug, Serialize)]
pub struct AddCommentRequest {
    pub body: AtlassianDoc,
}
```

**API Methods:**
```rust
impl JiraClient {
    pub async fn get_comments(
        &self,
        issue_key: &str,
        start: u32,
        max: u32,
    ) -> Result<CommentsResponse> {
        let url = format!(
            "{}/rest/api/3/issue/{}/comment?startAt={}&maxResults={}",
            self.base_url, issue_key, start, max
        );
        self.get(&url).await
    }

    pub async fn add_comment(&self, issue_key: &str, body: AtlassianDoc) -> Result<Comment> {
        let url = format!("{}/rest/api/3/issue/{}/comment", self.base_url, issue_key);

        let request = AddCommentRequest { body };

        let response = self.client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: JiraError = response.json().await?;
            Err(ApiError::CommentFailed(error.message()))
        }
    }
}
```

**Comments Section:**
```rust
pub struct CommentsSection {
    comments: Vec<Comment>,
    scroll: usize,
    expanded: bool,
    pagination: CommentPagination,
    editor: Option<CommentEditor>,
}

struct CommentPagination {
    total: u32,
    loaded: u32,
    loading: bool,
}

impl CommentsSection {
    pub fn new() -> Self {
        Self {
            comments: Vec::new(),
            scroll: 0,
            expanded: false,
            pagination: CommentPagination {
                total: 0,
                loaded: 0,
                loading: false,
            },
            editor: None,
        }
    }

    pub fn set_comments(&mut self, response: CommentsResponse) {
        if response.start_at == 0 {
            self.comments = response.comments;
        } else {
            self.comments.extend(response.comments);
        }
        self.pagination.total = response.total;
        self.pagination.loaded = self.comments.len() as u32;
        self.pagination.loading = false;
    }

    pub fn add_comment(&mut self, comment: Comment) {
        self.comments.push(comment);
        self.pagination.total += 1;
        self.pagination.loaded += 1;
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<CommentAction> {
        if let Some(editor) = &mut self.editor {
            return self.handle_editor_input(key);
        }

        match key.code {
            KeyCode::Char('c') => {
                self.editor = Some(CommentEditor::new());
                None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.scroll < self.comments.len().saturating_sub(1) {
                    self.scroll += 1;

                    // Load more if near end
                    if self.scroll >= self.comments.len().saturating_sub(3)
                        && self.pagination.loaded < self.pagination.total
                        && !self.pagination.loading
                    {
                        self.pagination.loading = true;
                        return Some(CommentAction::LoadMore(self.pagination.loaded));
                    }
                }
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
                None
            }
            KeyCode::Tab => {
                self.expanded = !self.expanded;
                None
            }
            _ => None,
        }
    }

    fn handle_editor_input(&mut self, key: KeyEvent) -> Option<CommentAction> {
        let editor = self.editor.as_mut().unwrap();

        match key.code {
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let content = editor.content();
                if !content.trim().is_empty() {
                    let adf = text_to_adf(&content);
                    self.editor = None;
                    return Some(CommentAction::Submit(adf));
                }
                None
            }
            KeyCode::Esc => {
                if editor.has_content() {
                    // Could add confirmation
                }
                self.editor = None;
                None
            }
            _ => {
                editor.handle_input(key);
                None
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(format!("Comments ({}/{})",
                self.pagination.loaded,
                self.pagination.total
            ))
            .borders(Borders::ALL);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if let Some(editor) = &self.editor {
            editor.render(frame, inner);
            return;
        }

        if self.comments.is_empty() {
            let empty = Paragraph::new("No comments yet. Press 'c' to add one.")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(empty, inner);
            return;
        }

        // Calculate visible comments based on expanded state
        let comment_height = if self.expanded { 8 } else { 4 };
        let visible_count = (inner.height as usize / comment_height).max(1);

        let start_idx = self.scroll.saturating_sub(visible_count / 2);
        let end_idx = (start_idx + visible_count).min(self.comments.len());

        let comments_to_show = &self.comments[start_idx..end_idx];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                comments_to_show.iter()
                    .map(|_| Constraint::Length(comment_height as u16))
                    .collect::<Vec<_>>()
            )
            .split(inner);

        for (i, comment) in comments_to_show.iter().enumerate() {
            let is_selected = start_idx + i == self.scroll;
            self.render_comment(frame, comment, chunks[i], is_selected);
        }

        // Loading indicator
        if self.pagination.loading {
            let loading = Paragraph::new("Loading more comments...")
                .style(Style::default().fg(Color::Gray));
            // Render at bottom of area
        }
    }

    fn render_comment(&self, frame: &mut Frame, comment: &Comment, area: Rect, selected: bool) {
        let style = if selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        // Format timestamp
        let timestamp = format_relative_time(&comment.created);

        // Header: Author - Timestamp
        let header = Line::from(vec![
            Span::styled(&comment.author.display_name, Style::default().bold()),
            Span::raw(" - "),
            Span::styled(&timestamp, Style::default().fg(Color::Gray)),
        ]);

        // Body (truncated if not expanded)
        let body_text = comment.body.to_plain_text();
        let body = if self.expanded {
            body_text
        } else {
            body_text.lines().take(2).collect::<Vec<_>>().join("\n")
        };

        let content = vec![header, Line::from(""), Line::from(body)];

        let paragraph = Paragraph::new(content)
            .style(style)
            .block(Block::default().borders(Borders::BOTTOM));

        frame.render_widget(paragraph, area);
    }
}

fn format_relative_time(iso_time: &str) -> String {
    // Parse ISO timestamp and format as relative time
    // "2024-01-15T10:30:00.000+0000" -> "2 hours ago"
    // Simplified implementation
    iso_time.split('T').next().unwrap_or(iso_time).to_string()
}

pub enum CommentAction {
    Submit(AtlassianDoc),
    LoadMore(u32),
}
```

**Comment Editor:**
```rust
pub struct CommentEditor {
    editor: TextEditor,
}

impl CommentEditor {
    pub fn new() -> Self {
        Self {
            editor: TextEditor::new(""),
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) {
        self.editor.handle_input(key);
    }

    pub fn content(&self) -> String {
        self.editor.content()
    }

    pub fn has_content(&self) -> bool {
        !self.content().trim().is_empty()
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(1),
            ])
            .split(area);

        // Editor
        let block = Block::default()
            .title("New Comment (Ctrl+Enter to submit, Esc to cancel)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(chunks[0]);
        frame.render_widget(block, chunks[0]);

        let content = Paragraph::new(self.editor.content())
            .wrap(Wrap { trim: false });
        frame.render_widget(content, inner);

        // Cursor
        let (line, col) = self.editor.cursor_position();
        frame.set_cursor_position(Position::new(
            inner.x + col as u16,
            inner.y + line as u16,
        ));

        // Help text
        let help = Paragraph::new("Ctrl+Enter: Submit | Esc: Cancel")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[1]);
    }
}
```

## Testing Requirements

- [ ] Comments load for issue
- [ ] Comments display author and time
- [ ] 'c' opens comment editor
- [ ] Multi-line editing works
- [ ] Ctrl+Enter submits comment
- [ ] Escape cancels editor
- [ ] New comment appears in list
- [ ] Pagination loads more comments
- [ ] Empty state displays correctly

## Dependencies

- **Prerequisite Tasks:** Task 1.7, Task 3.1, Task 3.2 (text editor)
- **Blocks Tasks:** None
- **External:** JIRA comments API

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Comments are readable
- [ ] Editor is usable
- [ ] Pagination works smoothly
- [ ] Success feedback provided
