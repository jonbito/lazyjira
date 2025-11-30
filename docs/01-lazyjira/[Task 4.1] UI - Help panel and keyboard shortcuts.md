# Task 4.1: Help Panel and Keyboard Shortcuts

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 4.1
**Area:** Frontend/UI
**Estimated Effort:** S (2-4 hours)

## Description

Implement a global help panel accessible via '?' that displays all available keyboard shortcuts organized by context. Provide contextual hints and discoverability for available actions.

## Acceptance Criteria

- [ ] Help panel opens with '?' key from any view
- [ ] Shortcuts organized by context (global, list, detail, etc.)
- [ ] Scrollable if content exceeds screen
- [ ] Close with '?' or 'q' or Escape
- [ ] Contextual hints shown in status bar
- [ ] Searchable shortcuts (nice-to-have)
- [ ] Visual key representations

## Implementation Details

### Approach

1. Define keybinding registry
2. Create help panel component
3. Organize shortcuts by category
4. Add contextual hints to views
5. Implement search/filter

### Files to Modify/Create

- `src/events/keys.rs`: Keybinding definitions
- `src/ui/views/help.rs`: Help panel view
- `src/ui/components/help_bar.rs`: Contextual help bar

### Technical Specifications

**Keybinding Definitions:**
```rust
#[derive(Debug, Clone)]
pub struct Keybinding {
    pub key: String,
    pub action: String,
    pub description: String,
    pub context: KeyContext,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyContext {
    Global,
    IssueList,
    IssueDetail,
    ProfileManagement,
    FilterPanel,
    Editor,
}

impl KeyContext {
    fn display(&self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::IssueList => "Issue List",
            Self::IssueDetail => "Issue Detail",
            Self::ProfileManagement => "Profiles",
            Self::FilterPanel => "Filters",
            Self::Editor => "Editor",
        }
    }
}

pub fn get_keybindings() -> Vec<Keybinding> {
    vec![
        // Global
        Keybinding {
            key: "?".to_string(),
            action: "help".to_string(),
            description: "Show this help panel".to_string(),
            context: KeyContext::Global,
        },
        Keybinding {
            key: "q".to_string(),
            action: "quit".to_string(),
            description: "Go back / Quit application".to_string(),
            context: KeyContext::Global,
        },
        Keybinding {
            key: "p".to_string(),
            action: "switch_profile".to_string(),
            description: "Switch JIRA profile".to_string(),
            context: KeyContext::Global,
        },
        Keybinding {
            key: "r".to_string(),
            action: "refresh".to_string(),
            description: "Refresh current view".to_string(),
            context: KeyContext::Global,
        },

        // Issue List
        Keybinding {
            key: "j / ↓".to_string(),
            action: "move_down".to_string(),
            description: "Move down".to_string(),
            context: KeyContext::IssueList,
        },
        Keybinding {
            key: "k / ↑".to_string(),
            action: "move_up".to_string(),
            description: "Move up".to_string(),
            context: KeyContext::IssueList,
        },
        Keybinding {
            key: "Enter".to_string(),
            action: "open_issue".to_string(),
            description: "Open issue detail".to_string(),
            context: KeyContext::IssueList,
        },
        Keybinding {
            key: "f".to_string(),
            action: "filter".to_string(),
            description: "Open filter panel".to_string(),
            context: KeyContext::IssueList,
        },
        Keybinding {
            key: "/".to_string(),
            action: "search".to_string(),
            description: "Quick search in loaded issues".to_string(),
            context: KeyContext::IssueList,
        },
        Keybinding {
            key: ":".to_string(),
            action: "jql".to_string(),
            description: "Enter JQL query".to_string(),
            context: KeyContext::IssueList,
        },
        Keybinding {
            key: "gg".to_string(),
            action: "go_top".to_string(),
            description: "Go to first issue".to_string(),
            context: KeyContext::IssueList,
        },
        Keybinding {
            key: "G".to_string(),
            action: "go_bottom".to_string(),
            description: "Go to last issue".to_string(),
            context: KeyContext::IssueList,
        },

        // Issue Detail
        Keybinding {
            key: "e".to_string(),
            action: "edit".to_string(),
            description: "Edit issue".to_string(),
            context: KeyContext::IssueDetail,
        },
        Keybinding {
            key: "c".to_string(),
            action: "comment".to_string(),
            description: "Add comment".to_string(),
            context: KeyContext::IssueDetail,
        },
        Keybinding {
            key: "t".to_string(),
            action: "transition".to_string(),
            description: "Change status".to_string(),
            context: KeyContext::IssueDetail,
        },
        Keybinding {
            key: "a".to_string(),
            action: "assign".to_string(),
            description: "Change assignee".to_string(),
            context: KeyContext::IssueDetail,
        },
        Keybinding {
            key: "P".to_string(),
            action: "priority".to_string(),
            description: "Change priority".to_string(),
            context: KeyContext::IssueDetail,
        },
        Keybinding {
            key: "l".to_string(),
            action: "labels".to_string(),
            description: "Edit labels".to_string(),
            context: KeyContext::IssueDetail,
        },

        // Editor
        Keybinding {
            key: "Ctrl+S".to_string(),
            action: "save".to_string(),
            description: "Save changes".to_string(),
            context: KeyContext::Editor,
        },
        Keybinding {
            key: "Esc".to_string(),
            action: "cancel".to_string(),
            description: "Cancel editing".to_string(),
            context: KeyContext::Editor,
        },
    ]
}
```

**Help Panel View:**
```rust
pub struct HelpView {
    keybindings: Vec<Keybinding>,
    scroll: usize,
    visible: bool,
    search: String,
}

impl HelpView {
    pub fn new() -> Self {
        Self {
            keybindings: get_keybindings(),
            scroll: 0,
            visible: false,
            search: String::new(),
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        self.scroll = 0;
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        if !self.visible {
            return false;
        }

        match key.code {
            KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc => {
                self.visible = false;
                true
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll = self.scroll.saturating_add(1);
                true
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
                true
            }
            KeyCode::Char('g') => {
                self.scroll = 0;
                true
            }
            KeyCode::Char('G') => {
                self.scroll = self.max_scroll();
                true
            }
            _ => true, // Consume all input when help is open
        }
    }

    fn max_scroll(&self) -> usize {
        // Calculate based on grouped keybindings
        self.keybindings.len().saturating_sub(10)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        frame.render_widget(Clear, area);

        let block = Block::default()
            .title("Help - Keyboard Shortcuts")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Group keybindings by context
        let grouped = self.group_by_context();

        let mut lines: Vec<Line> = Vec::new();

        for (context, bindings) in grouped {
            // Context header
            lines.push(Line::from(vec![
                Span::styled(
                    format!("── {} ──", context.display()),
                    Style::default().bold().fg(Color::Yellow),
                ),
            ]));
            lines.push(Line::from(""));

            // Keybindings in this context
            for binding in bindings {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{:>12}", binding.key),
                        Style::default().fg(Color::Green).bold(),
                    ),
                    Span::raw("  "),
                    Span::raw(&binding.description),
                ]));
            }

            lines.push(Line::from(""));
        }

        let paragraph = Paragraph::new(lines)
            .scroll((self.scroll as u16, 0));

        frame.render_widget(paragraph, inner);

        // Scroll indicator
        let total_lines = self.keybindings.len() + grouped.len() * 3;
        let visible_lines = inner.height as usize;
        if total_lines > visible_lines {
            let indicator = format!(
                "↑↓ scroll | {}/{}",
                self.scroll + 1,
                total_lines.saturating_sub(visible_lines) + 1
            );
            let indicator_widget = Paragraph::new(indicator)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Right);
            frame.render_widget(indicator_widget, Rect::new(
                inner.x,
                area.y + area.height - 1,
                inner.width,
                1,
            ));
        }
    }

    fn group_by_context(&self) -> Vec<(KeyContext, Vec<&Keybinding>)> {
        let contexts = [
            KeyContext::Global,
            KeyContext::IssueList,
            KeyContext::IssueDetail,
            KeyContext::Editor,
            KeyContext::FilterPanel,
            KeyContext::ProfileManagement,
        ];

        contexts.iter()
            .filter_map(|ctx| {
                let bindings: Vec<&Keybinding> = self.keybindings.iter()
                    .filter(|b| &b.context == ctx)
                    .collect();
                if bindings.is_empty() {
                    None
                } else {
                    Some((ctx.clone(), bindings))
                }
            })
            .collect()
    }
}
```

**Contextual Help Bar:**
```rust
pub fn render_context_help(frame: &mut Frame, area: Rect, context: KeyContext) {
    let hints = match context {
        KeyContext::IssueList => "[j/k] navigate  [Enter] open  [f] filter  [/] search  [?] help",
        KeyContext::IssueDetail => "[e] edit  [c] comment  [t] status  [a] assign  [q] back  [?] help",
        KeyContext::FilterPanel => "[Space] toggle  [Tab] section  [Enter] apply  [c] clear  [Esc] close",
        KeyContext::Editor => "[Ctrl+S] save  [Esc] cancel",
        _ => "[?] help",
    };

    let widget = Paragraph::new(hints)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(widget, area);
}
```

## Testing Requirements

- [ ] '?' opens help panel
- [ ] Help displays all shortcuts
- [ ] Scrolling works in help
- [ ] '?', 'q', Esc closes help
- [ ] Shortcuts grouped by context
- [ ] Contextual hints update per view

## Dependencies

- **Prerequisite Tasks:** Task 1.2
- **Blocks Tasks:** None
- **External:** None

## Definition of Done

- [ ] All acceptance criteria met
- [ ] All shortcuts documented
- [ ] Help is discoverable
- [ ] Contextual hints helpful
- [ ] Scrolling smooth
