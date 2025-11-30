# Task 4.5: Linked Issues and Subtasks

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 4.5
**Area:** Frontend/UI
**Estimated Effort:** M (4-6 hours)

## Description

Display linked issues and subtasks in the issue detail view with navigation support. Users should be able to see relationships and quickly jump to related issues.

## Acceptance Criteria

- [ ] Display linked issues section in detail view
- [ ] Show link type (blocks, is blocked by, relates to, etc.)
- [ ] Display subtasks with status indicators
- [ ] Navigate to linked issue with Enter
- [ ] Show parent issue for subtasks
- [ ] Visual distinction for link types
- [ ] Collapse/expand linked issues section
- [ ] Quick preview on hover/focus (nice-to-have)

## Implementation Details

### Approach

1. Parse issue links and subtasks from API response
2. Create linked issues component
3. Add navigation to linked issues
4. Implement expand/collapse

### Files to Modify/Create

- `src/ui/views/detail.rs`: Linked issues section
- `src/ui/components/linked_issues.rs`: Linked issues component
- `src/api/types.rs`: Link types

### Technical Specifications

**Link Types:**
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct IssueLink {
    pub id: String,
    #[serde(rename = "type")]
    pub link_type: IssueLinkType,
    #[serde(rename = "inwardIssue")]
    pub inward_issue: Option<LinkedIssue>,
    #[serde(rename = "outwardIssue")]
    pub outward_issue: Option<LinkedIssue>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueLinkType {
    pub id: String,
    pub name: String,
    pub inward: String,   // e.g., "is blocked by"
    pub outward: String,  // e.g., "blocks"
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinkedIssue {
    pub id: String,
    pub key: String,
    pub fields: LinkedIssueFields,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinkedIssueFields {
    pub summary: String,
    pub status: Status,
    pub priority: Option<Priority>,
    #[serde(rename = "issuetype")]
    pub issue_type: IssueType,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Subtask {
    pub id: String,
    pub key: String,
    pub fields: SubtaskFields,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubtaskFields {
    pub summary: String,
    pub status: Status,
}

// Extended issue fields to include links and subtasks
#[derive(Debug, Clone, Deserialize)]
pub struct IssueFields {
    // ... existing fields ...
    #[serde(default, rename = "issuelinks")]
    pub issue_links: Vec<IssueLink>,
    #[serde(default)]
    pub subtasks: Vec<Subtask>,
    pub parent: Option<ParentIssue>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParentIssue {
    pub id: String,
    pub key: String,
    pub fields: ParentIssueFields,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParentIssueFields {
    pub summary: String,
    pub status: Status,
}
```

**Linked Issues Component:**
```rust
pub struct LinkedIssuesSection {
    links: Vec<DisplayLink>,
    subtasks: Vec<Subtask>,
    parent: Option<ParentIssue>,
    selected: usize,
    expanded: bool,
    focused: bool,
}

struct DisplayLink {
    key: String,
    summary: String,
    status: Status,
    link_description: String, // e.g., "blocks", "is blocked by"
    direction: LinkDirection,
}

enum LinkDirection {
    Inward,  // Other issue affects this one
    Outward, // This issue affects other
}

impl LinkedIssuesSection {
    pub fn new(
        links: &[IssueLink],
        subtasks: &[Subtask],
        parent: Option<ParentIssue>,
    ) -> Self {
        let display_links: Vec<DisplayLink> = links.iter()
            .filter_map(|link| {
                if let Some(inward) = &link.inward_issue {
                    Some(DisplayLink {
                        key: inward.key.clone(),
                        summary: inward.fields.summary.clone(),
                        status: inward.fields.status.clone(),
                        link_description: link.link_type.inward.clone(),
                        direction: LinkDirection::Inward,
                    })
                } else if let Some(outward) = &link.outward_issue {
                    Some(DisplayLink {
                        key: outward.key.clone(),
                        summary: outward.fields.summary.clone(),
                        status: outward.fields.status.clone(),
                        link_description: link.link_type.outward.clone(),
                        direction: LinkDirection::Outward,
                    })
                } else {
                    None
                }
            })
            .collect();

        Self {
            links: display_links,
            subtasks: subtasks.to_vec(),
            parent: parent.clone(),
            selected: 0,
            expanded: true,
            focused: false,
        }
    }

    pub fn total_items(&self) -> usize {
        let parent_count = if self.parent.is_some() { 1 } else { 0 };
        parent_count + self.links.len() + self.subtasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.total_items() == 0
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<LinkAction> {
        if !self.focused {
            return None;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.total_items().saturating_sub(1) {
                    self.selected += 1;
                }
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            KeyCode::Enter => {
                if let Some(key) = self.selected_key() {
                    Some(LinkAction::Navigate(key))
                } else {
                    None
                }
            }
            KeyCode::Tab => {
                self.expanded = !self.expanded;
                None
            }
            _ => None,
        }
    }

    fn selected_key(&self) -> Option<String> {
        let mut index = self.selected;

        // Parent first
        if let Some(parent) = &self.parent {
            if index == 0 {
                return Some(parent.key.clone());
            }
            index -= 1;
        }

        // Then links
        if index < self.links.len() {
            return Some(self.links[index].key.clone());
        }
        index -= self.links.len();

        // Then subtasks
        if index < self.subtasks.len() {
            return Some(self.subtasks[index].key.clone());
        }

        None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if self.is_empty() {
            return;
        }

        let title = if self.expanded {
            format!("Related Issues ▼ ({})", self.total_items())
        } else {
            format!("Related Issues ▶ ({})", self.total_items())
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if self.focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        if !self.expanded {
            frame.render_widget(block, area);
            return;
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();
        let mut item_index = 0;

        // Parent issue
        if let Some(parent) = &self.parent {
            let is_selected = self.focused && item_index == self.selected;
            lines.push(self.render_link_line(
                "↑ Parent",
                &parent.key,
                &parent.fields.summary,
                &parent.fields.status,
                is_selected,
            ));
            item_index += 1;
        }

        // Links
        if !self.links.is_empty() {
            lines.push(Line::from(Span::styled(
                "Linked Issues:",
                Style::default().bold().fg(Color::Gray),
            )));

            for link in &self.links {
                let is_selected = self.focused && item_index == self.selected;
                let prefix = match link.direction {
                    LinkDirection::Inward => "←",
                    LinkDirection::Outward => "→",
                };
                lines.push(self.render_link_line(
                    &format!("{} {}", prefix, link.link_description),
                    &link.key,
                    &link.summary,
                    &link.status,
                    is_selected,
                ));
                item_index += 1;
            }
        }

        // Subtasks
        if !self.subtasks.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Subtasks:",
                Style::default().bold().fg(Color::Gray),
            )));

            for subtask in &self.subtasks {
                let is_selected = self.focused && item_index == self.selected;
                lines.push(self.render_subtask_line(subtask, is_selected));
                item_index += 1;
            }
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    fn render_link_line(
        &self,
        prefix: &str,
        key: &str,
        summary: &str,
        status: &Status,
        selected: bool,
    ) -> Line<'static> {
        let style = if selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let status_style = status_category_style(&status.category);

        Line::from(vec![
            Span::styled(format!("  {} ", prefix), Style::default().fg(Color::Gray)),
            Span::styled(key.to_string(), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::raw(truncate(summary, 40)),
            Span::raw(" ["),
            Span::styled(status.name.clone(), status_style),
            Span::raw("]"),
        ]).style(style)
    }

    fn render_subtask_line(&self, subtask: &Subtask, selected: bool) -> Line<'static> {
        let style = if selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let checkbox = if subtask.fields.status.is_done() {
            "[x]"
        } else {
            "[ ]"
        };

        let status_style = status_category_style(&subtask.fields.status.category);

        Line::from(vec![
            Span::styled(format!("  {} ", checkbox), style),
            Span::styled(subtask.key.clone(), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::raw(truncate(&subtask.fields.summary, 40)),
            Span::raw(" ["),
            Span::styled(subtask.fields.status.name.clone(), status_style),
            Span::raw("]"),
        ]).style(style)
    }
}

impl Status {
    fn is_done(&self) -> bool {
        self.category.as_ref()
            .map(|c| c.key == "done")
            .unwrap_or(false)
    }
}

pub enum LinkAction {
    Navigate(String),
}
```

## Testing Requirements

- [ ] Links display in detail view
- [ ] Subtasks display correctly
- [ ] Parent issue shown for subtasks
- [ ] Enter navigates to linked issue
- [ ] Tab toggles expand/collapse
- [ ] Status indicators correct
- [ ] j/k navigation works
- [ ] Link types displayed

## Dependencies

- **Prerequisite Tasks:** Task 1.7
- **Blocks Tasks:** None
- **External:** JIRA issue links API

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Navigation works smoothly
- [ ] All link types supported
- [ ] Subtasks progress visible
- [ ] Back navigation preserved
