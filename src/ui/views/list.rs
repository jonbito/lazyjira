//! Issue list view.
//!
//! Displays a table of JIRA issues with columns for Key, Summary, Status,
//! Assignee, and Priority. Supports keyboard navigation, column sorting,
//! pagination, and visual indicators for issue priority and type.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::api::types::Issue;
use crate::ui::theme::{issue_type_prefix, priority_style, status_style, truncate};

// ============================================================================
// Sorting Types
// ============================================================================

/// Column that can be sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Key,
    Summary,
    Status,
    Assignee,
    Priority,
}

impl SortColumn {
    /// Convert to JQL field name.
    pub fn to_jql_field(&self) -> &'static str {
        match self {
            SortColumn::Key => "key",
            SortColumn::Summary => "summary",
            SortColumn::Status => "status",
            SortColumn::Assignee => "assignee",
            SortColumn::Priority => "priority",
        }
    }

    /// Get display name for the column.
    pub fn display_name(&self) -> &'static str {
        match self {
            SortColumn::Key => "Key",
            SortColumn::Summary => "Summary",
            SortColumn::Status => "Status",
            SortColumn::Assignee => "Assignee",
            SortColumn::Priority => "Priority",
        }
    }

    /// Get column index (0-based).
    pub fn index(&self) -> usize {
        match self {
            SortColumn::Key => 0,
            SortColumn::Summary => 1,
            SortColumn::Status => 2,
            SortColumn::Assignee => 3,
            SortColumn::Priority => 4,
        }
    }

    /// Get column from index.
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(SortColumn::Key),
            1 => Some(SortColumn::Summary),
            2 => Some(SortColumn::Status),
            3 => Some(SortColumn::Assignee),
            4 => Some(SortColumn::Priority),
            _ => None,
        }
    }
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    /// Toggle to the opposite direction.
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    /// Convert to JQL sort direction.
    pub fn to_jql(&self) -> &'static str {
        match self {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        }
    }

    /// Get display indicator for the sort direction.
    pub fn indicator(&self) -> &'static str {
        match self {
            SortDirection::Ascending => " ▲",
            SortDirection::Descending => " ▼",
        }
    }
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Descending
    }
}

/// Current sort state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortState {
    /// Currently sorted column.
    pub column: SortColumn,
    /// Sort direction.
    pub direction: SortDirection,
}

impl SortState {
    /// Create a new sort state.
    pub fn new(column: SortColumn, direction: SortDirection) -> Self {
        Self { column, direction }
    }

    /// Generate JQL ORDER BY clause.
    pub fn to_jql(&self) -> String {
        format!(
            "ORDER BY {} {}",
            self.column.to_jql_field(),
            self.direction.to_jql()
        )
    }

    /// Toggle sort for a column.
    /// If clicking same column, toggle direction.
    /// If clicking different column, switch to that column with descending.
    pub fn toggle_column(&mut self, column: SortColumn) {
        if self.column == column {
            self.direction = self.direction.toggle();
        } else {
            self.column = column;
            self.direction = SortDirection::Descending;
        }
    }
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            column: SortColumn::Key,
            direction: SortDirection::Descending,
        }
    }
}

// ============================================================================
// Pagination Types
// ============================================================================

/// Pagination state for issue list.
#[derive(Debug, Clone)]
pub struct PaginationState {
    /// Number of issues per page.
    pub page_size: u32,
    /// Current offset (start_at for API).
    pub current_offset: u32,
    /// Total number of issues matching the query.
    pub total: u32,
    /// Whether we're currently loading more issues.
    pub loading: bool,
    /// Whether there are more issues to load.
    pub has_more: bool,
}

impl PaginationState {
    /// Default page size.
    pub const DEFAULT_PAGE_SIZE: u32 = 50;

    /// Create a new pagination state.
    pub fn new() -> Self {
        Self {
            page_size: Self::DEFAULT_PAGE_SIZE,
            current_offset: 0,
            total: 0,
            loading: false,
            has_more: true,
        }
    }

    /// Create pagination state with custom page size.
    pub fn with_page_size(page_size: u32) -> Self {
        Self {
            page_size,
            ..Self::new()
        }
    }

    /// Update state from API response.
    pub fn update_from_response(&mut self, start_at: u32, count: u32, total: u32) {
        self.current_offset = start_at + count;
        self.total = total;
        self.has_more = self.current_offset < total;
        self.loading = false;
    }

    /// Reset pagination state (e.g., when filters change).
    pub fn reset(&mut self) {
        self.current_offset = 0;
        self.total = 0;
        self.loading = false;
        self.has_more = true;
    }

    /// Get the number of issues currently loaded.
    pub fn loaded_count(&self) -> u32 {
        self.current_offset
    }

    /// Get display string for pagination info.
    pub fn display(&self) -> String {
        if self.total == 0 {
            "No issues".to_string()
        } else {
            format!("1-{} of {}", self.loaded_count(), self.total)
        }
    }

    /// Start loading more issues.
    pub fn start_loading(&mut self) {
        self.loading = true;
    }
}

impl Default for PaginationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Action that can be triggered from the list view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListAction {
    /// Open the selected issue for detailed view.
    OpenIssue(String),
    /// Refresh the issue list (triggers full reload with current sort).
    Refresh,
    /// Open the filter panel.
    OpenFilter,
    /// Open the JQL query input.
    OpenJqlInput,
    /// Sort changed - need to refresh issues with new sort order.
    SortChanged,
    /// Load more issues (pagination).
    LoadMore,
}

/// The issue list view state.
pub struct ListView {
    /// The list of issues to display.
    issues: Vec<Issue>,
    /// Currently selected index.
    selected: usize,
    /// Scroll offset for the table.
    scroll_offset: usize,
    /// Table state for ratatui.
    table_state: TableState,
    /// Whether the view is currently loading data.
    loading: bool,
    /// Current profile name (for status bar).
    profile_name: Option<String>,
    /// Pending 'g' key for gg navigation.
    pending_g: bool,
    /// Filter summary to display in the status bar.
    filter_summary: Option<String>,
    /// Current sort state.
    sort: SortState,
    /// Pagination state.
    pagination: PaginationState,
    /// Whether the header row is focused (for sorting).
    header_focused: bool,
    /// Currently focused column index (when header is focused).
    focused_column: usize,
}

impl ListView {
    /// Create a new list view.
    pub fn new() -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Self {
            issues: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            table_state,
            loading: false,
            profile_name: None,
            pending_g: false,
            filter_summary: None,
            sort: SortState::default(),
            pagination: PaginationState::new(),
            header_focused: false,
            focused_column: 0,
        }
    }

    /// Set the list of issues to display.
    pub fn set_issues(&mut self, issues: Vec<Issue>) {
        self.issues = issues;
        self.selected = 0;
        self.scroll_offset = 0;
        self.table_state.select(Some(0));
        self.loading = false;
    }

    /// Set the loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Check if the view is loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Set the current profile name.
    pub fn set_profile_name(&mut self, name: Option<String>) {
        self.profile_name = name;
    }

    /// Set the filter summary to display.
    pub fn set_filter_summary(&mut self, summary: Option<String>) {
        self.filter_summary = summary;
    }

    /// Get the current filter summary.
    pub fn filter_summary(&self) -> Option<&str> {
        self.filter_summary.as_deref()
    }

    /// Get the currently selected issue.
    pub fn selected_issue(&self) -> Option<&Issue> {
        self.issues.get(self.selected)
    }

    /// Get the selected index.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Get the number of issues.
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    // ========================================================================
    // Sort Methods
    // ========================================================================

    /// Get the current sort state.
    pub fn sort(&self) -> &SortState {
        &self.sort
    }

    /// Get a mutable reference to the sort state.
    pub fn sort_mut(&mut self) -> &mut SortState {
        &mut self.sort
    }

    /// Set the sort state.
    pub fn set_sort(&mut self, sort: SortState) {
        self.sort = sort;
    }

    /// Check if header is currently focused for sorting.
    pub fn is_header_focused(&self) -> bool {
        self.header_focused
    }

    /// Enter header focus mode for sorting.
    pub fn enter_header_mode(&mut self) {
        self.header_focused = true;
        self.focused_column = self.sort.column.index();
    }

    /// Exit header focus mode.
    pub fn exit_header_mode(&mut self) {
        self.header_focused = false;
    }

    // ========================================================================
    // Pagination Methods
    // ========================================================================

    /// Get the current pagination state.
    pub fn pagination(&self) -> &PaginationState {
        &self.pagination
    }

    /// Get a mutable reference to the pagination state.
    pub fn pagination_mut(&mut self) -> &mut PaginationState {
        &mut self.pagination
    }

    /// Append issues to the existing list (for pagination).
    pub fn append_issues(&mut self, new_issues: Vec<Issue>) {
        self.issues.extend(new_issues);
        self.pagination.loading = false;
    }

    /// Update pagination state from API response.
    pub fn update_pagination(&mut self, start_at: u32, count: u32, total: u32) {
        self.pagination.update_from_response(start_at, count, total);
    }

    /// Reset for a new query (clears issues and pagination).
    pub fn reset_for_new_query(&mut self) {
        self.issues.clear();
        self.selected = 0;
        self.scroll_offset = 0;
        self.table_state.select(Some(0));
        self.pagination.reset();
    }

    /// Check if we should load more issues (near end of list).
    fn check_load_more(&self) -> Option<ListAction> {
        // Load more if within 5 items of end
        let threshold = 5;
        if self.selected + threshold >= self.issues.len()
            && self.pagination.has_more
            && !self.pagination.loading
        {
            Some(ListAction::LoadMore)
        } else {
            None
        }
    }

    /// Handle keyboard input.
    ///
    /// Returns an optional action to be handled by the application.
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<ListAction> {
        // Handle header mode for sorting
        if self.header_focused {
            return self.handle_header_input(key);
        }

        // Handle pending 'g' for gg navigation
        if self.pending_g {
            self.pending_g = false;
            if key.code == KeyCode::Char('g') && key.modifiers == KeyModifiers::NONE {
                self.move_to_start();
                return None;
            }
            // If not 'g', fall through to normal handling
        }

        match (key.code, key.modifiers) {
            // Navigation
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                self.move_down();
                return self.check_load_more();
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                self.move_up();
            }
            (KeyCode::Char('g'), KeyModifiers::NONE) => {
                // Wait for second 'g'
                self.pending_g = true;
            }
            (KeyCode::Char('G'), KeyModifiers::SHIFT) | (KeyCode::Char('G'), KeyModifiers::NONE) => {
                self.move_to_end();
                return self.check_load_more();
            }
            (KeyCode::Home, _) => {
                self.move_to_start();
            }
            (KeyCode::End, _) => {
                self.move_to_end();
                return self.check_load_more();
            }
            (KeyCode::PageDown, _) | (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                self.page_down();
                return self.check_load_more();
            }
            (KeyCode::PageUp, _) | (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                self.page_up();
            }
            // Enter sort/header mode
            (KeyCode::Char('s'), KeyModifiers::NONE) => {
                self.enter_header_mode();
            }
            // Actions
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(issue) = self.selected_issue() {
                    return Some(ListAction::OpenIssue(issue.key.clone()));
                }
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                return Some(ListAction::Refresh);
            }
            (KeyCode::Char('f'), KeyModifiers::NONE) => {
                return Some(ListAction::OpenFilter);
            }
            // JQL input
            (KeyCode::Char(':'), KeyModifiers::NONE)
            | (KeyCode::Char(':'), KeyModifiers::SHIFT)
            | (KeyCode::Char('/'), KeyModifiers::NONE) => {
                return Some(ListAction::OpenJqlInput);
            }
            _ => {}
        }
        None
    }

    /// Handle keyboard input when header is focused for sorting.
    fn handle_header_input(&mut self, key: KeyEvent) -> Option<ListAction> {
        match (key.code, key.modifiers) {
            // Navigate left in header
            (KeyCode::Left, _) | (KeyCode::Char('h'), KeyModifiers::NONE) => {
                if self.focused_column > 0 {
                    self.focused_column -= 1;
                }
                None
            }
            // Navigate right in header
            (KeyCode::Right, _) | (KeyCode::Char('l'), KeyModifiers::NONE) => {
                if self.focused_column < 4 {
                    self.focused_column += 1;
                }
                None
            }
            // Select column to sort
            (KeyCode::Enter, KeyModifiers::NONE) | (KeyCode::Char(' '), KeyModifiers::NONE) => {
                if let Some(column) = SortColumn::from_index(self.focused_column) {
                    self.sort.toggle_column(column);
                    self.header_focused = false;
                    Some(ListAction::SortChanged)
                } else {
                    None
                }
            }
            // Cancel header mode
            (KeyCode::Esc, _) | (KeyCode::Char('s'), KeyModifiers::NONE) => {
                self.header_focused = false;
                None
            }
            _ => None,
        }
    }

    /// Move selection down by one.
    fn move_down(&mut self) {
        if self.issues.is_empty() {
            return;
        }
        if self.selected < self.issues.len() - 1 {
            self.selected += 1;
            self.table_state.select(Some(self.selected));
        }
    }

    /// Move selection up by one.
    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.table_state.select(Some(self.selected));
        }
    }

    /// Move selection to the first item.
    fn move_to_start(&mut self) {
        self.selected = 0;
        self.table_state.select(Some(0));
    }

    /// Move selection to the last item.
    fn move_to_end(&mut self) {
        if !self.issues.is_empty() {
            self.selected = self.issues.len() - 1;
            self.table_state.select(Some(self.selected));
        }
    }

    /// Move selection down by a page (10 items).
    fn page_down(&mut self) {
        if self.issues.is_empty() {
            return;
        }
        let page_size = 10;
        self.selected = (self.selected + page_size).min(self.issues.len() - 1);
        self.table_state.select(Some(self.selected));
    }

    /// Move selection up by a page (10 items).
    fn page_up(&mut self) {
        let page_size = 10;
        self.selected = self.selected.saturating_sub(page_size);
        self.table_state.select(Some(self.selected));
    }

    /// Render the list view.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.loading {
            self.render_loading(frame, area);
        } else if self.issues.is_empty() {
            self.render_empty(frame, area);
        } else {
            self.render_table(frame, area);
        }
    }

    /// Render the loading state.
    fn render_loading(&self, frame: &mut Frame, area: Rect) {
        let loading = Paragraph::new("Loading issues...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));

        // Center the loading message vertically
        let vertical_center = area.y + area.height / 2;
        let centered_area = Rect {
            x: area.x,
            y: vertical_center.saturating_sub(1),
            width: area.width,
            height: 3,
        };

        frame.render_widget(loading, centered_area);
    }

    /// Render the empty state.
    fn render_empty(&self, frame: &mut Frame, area: Rect) {
        let message = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No issues found",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'f' to change filters",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "Press 'r' to refresh",
                Style::default().fg(Color::Gray),
            )),
        ];

        let paragraph = Paragraph::new(message).alignment(Alignment::Center);

        // Center vertically
        let vertical_center = area.y + area.height / 2;
        let centered_area = Rect {
            x: area.x,
            y: vertical_center.saturating_sub(3),
            width: area.width,
            height: 6,
        };

        frame.render_widget(paragraph, centered_area);
    }

    /// Render the issue table.
    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        // Calculate column widths based on available space
        let key_width = 14; // PROJ-12345
        let status_width = 15;
        let assignee_width = 20;
        let priority_width = 10;
        let min_summary_width = 30;

        // Summary gets remaining space
        let fixed_width = key_width + status_width + assignee_width + priority_width + 8; // 8 for borders/spacing
        let summary_width = if area.width as usize > fixed_width + min_summary_width {
            area.width as usize - fixed_width
        } else {
            min_summary_width
        };

        // Create header with sort indicators
        let columns = [
            (SortColumn::Key, "Key"),
            (SortColumn::Summary, "Summary"),
            (SortColumn::Status, "Status"),
            (SortColumn::Assignee, "Assignee"),
            (SortColumn::Priority, "Priority"),
        ];

        let header_cells: Vec<Cell> = columns
            .iter()
            .enumerate()
            .map(|(i, (col, name))| {
                // Add sort indicator if this is the sorted column
                let indicator = if self.sort.column == *col {
                    self.sort.direction.indicator()
                } else {
                    ""
                };
                let text = format!("{}{}", name, indicator);

                // Style: highlight focused column when in header mode
                let style = if self.header_focused && i == self.focused_column {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                };

                Cell::from(text).style(style)
            })
            .collect();

        let header = Row::new(header_cells).height(1).bottom_margin(1);

        // Create rows
        let rows: Vec<Row> = self
            .issues
            .iter()
            .map(|issue| {
                // Key with type prefix
                let type_prefix = issue_type_prefix(&issue.fields.issuetype.name);
                let key_text = format!("{} {}", type_prefix, issue.key);

                // Truncate summary to fit
                let summary = truncate(&issue.fields.summary, summary_width);

                // Status with color
                let status_style = status_style(&issue.fields.status);

                // Priority with color
                let priority_style = priority_style(issue.fields.priority.as_ref());

                Row::new(vec![
                    Cell::from(key_text),
                    Cell::from(summary),
                    Cell::from(issue.fields.status.name.clone()).style(status_style),
                    Cell::from(issue.assignee_name().to_string()),
                    Cell::from(issue.priority_name().to_string()).style(priority_style),
                ])
            })
            .collect();

        // Define column constraints
        let widths = [
            Constraint::Length(key_width as u16),
            Constraint::Min(min_summary_width as u16),
            Constraint::Length(status_width as u16),
            Constraint::Length(assignee_width as u16),
            Constraint::Length(priority_width as u16),
        ];

        // Create table
        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::NONE))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    /// Render the status bar.
    pub fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let profile_text = self
            .profile_name
            .as_deref()
            .unwrap_or("No profile");

        // Use pagination display if we have pagination info, otherwise fall back to issue count
        let issue_count_text = if self.loading || self.pagination.loading {
            "Loading...".to_string()
        } else if self.pagination.total > 0 {
            self.pagination.display()
        } else {
            format!("{} issues", self.issues.len())
        };

        let selected_text = if !self.issues.is_empty() {
            format!(" [{}/{}]", self.selected + 1, self.issues.len())
        } else {
            String::new()
        };

        let mut spans = vec![
            Span::styled(
                format!(" {} ", profile_text),
                Style::default().fg(Color::Black).bg(Color::Cyan),
            ),
            Span::raw(" "),
            Span::styled(issue_count_text, Style::default().fg(Color::Gray)),
            Span::styled(selected_text, Style::default().fg(Color::DarkGray)),
        ];

        // Add sort info
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!(
                "[Sort: {}{}]",
                self.sort.column.display_name(),
                self.sort.direction.indicator()
            ),
            Style::default().fg(Color::Cyan),
        ));

        // Add filter summary if active
        if let Some(summary) = &self.filter_summary {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("[Filter: {}]", summary),
                Style::default().fg(Color::Yellow),
            ));
        }

        spans.push(Span::raw(" | "));

        // Show different help text when in header mode
        let help_text = if self.header_focused {
            "h/l:select column  Enter:sort  Esc:cancel"
        } else {
            "j/k:nav  s:sort  r:refresh  f:filter  :jql  ?:help"
        };
        spans.push(Span::styled(help_text, Style::default().fg(Color::DarkGray)));

        let status_line = Line::from(spans);
        let paragraph = Paragraph::new(status_line);
        frame.render_widget(paragraph, area);
    }
}

impl Default for ListView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{IssueFields, IssueType, Status};

    fn create_test_issue(key: &str, summary: &str) -> Issue {
        Issue {
            id: "1".to_string(),
            key: key.to_string(),
            self_url: "https://example.com".to_string(),
            fields: IssueFields {
                summary: summary.to_string(),
                description: None,
                status: Status {
                    id: "1".to_string(),
                    name: "Open".to_string(),
                    status_category: None,
                },
                issuetype: IssueType {
                    id: "1".to_string(),
                    name: "Bug".to_string(),
                    subtask: false,
                    description: None,
                    icon_url: None,
                },
                priority: None,
                assignee: None,
                reporter: None,
                project: None,
                labels: vec![],
                components: vec![],
                created: None,
                updated: None,
                duedate: None,
                story_points: None,
            },
        }
    }

    #[test]
    fn test_new_list_view() {
        let view = ListView::new();
        assert_eq!(view.selected, 0);
        assert!(view.issues.is_empty());
        assert!(!view.loading);
    }

    #[test]
    fn test_set_issues() {
        let mut view = ListView::new();
        let issues = vec![
            create_test_issue("TEST-1", "First issue"),
            create_test_issue("TEST-2", "Second issue"),
        ];
        view.set_issues(issues);

        assert_eq!(view.issue_count(), 2);
        assert_eq!(view.selected, 0);
        assert!(!view.loading);
    }

    #[test]
    fn test_move_down() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
            create_test_issue("TEST-3", "Third"),
        ]);

        assert_eq!(view.selected, 0);
        view.move_down();
        assert_eq!(view.selected, 1);
        view.move_down();
        assert_eq!(view.selected, 2);
        // Should not go past the end
        view.move_down();
        assert_eq!(view.selected, 2);
    }

    #[test]
    fn test_move_up() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
        ]);

        view.selected = 1;
        view.move_up();
        assert_eq!(view.selected, 0);
        // Should not go below 0
        view.move_up();
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_move_to_start_and_end() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
            create_test_issue("TEST-3", "Third"),
        ]);

        view.move_to_end();
        assert_eq!(view.selected, 2);

        view.move_to_start();
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_page_navigation() {
        let mut view = ListView::new();
        let issues: Vec<Issue> = (0..25)
            .map(|i| create_test_issue(&format!("TEST-{}", i), &format!("Issue {}", i)))
            .collect();
        view.set_issues(issues);

        // Start at 0
        assert_eq!(view.selected, 0);

        // Page down (10 items)
        view.page_down();
        assert_eq!(view.selected, 10);

        // Page down again
        view.page_down();
        assert_eq!(view.selected, 20);

        // Page down - should stop at last item (24)
        view.page_down();
        assert_eq!(view.selected, 24);

        // Page up
        view.page_up();
        assert_eq!(view.selected, 14);
    }

    #[test]
    fn test_handle_input_navigation() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
        ]);
        // Mark as no more pages so navigation doesn't trigger LoadMore
        view.pagination.has_more = false;

        // j key moves down
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert_eq!(view.selected, 1);

        // k key moves up
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert_eq!(view.selected, 0);

        // Down arrow moves down
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert_eq!(view.selected, 1);
    }

    #[test]
    fn test_handle_input_gg_navigation() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
            create_test_issue("TEST-3", "Third"),
        ]);

        // Move to end
        view.move_to_end();
        assert_eq!(view.selected, 2);

        // First 'g' should set pending
        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert!(view.pending_g);

        // Second 'g' should move to start
        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert_eq!(view.selected, 0);
        assert!(!view.pending_g);
    }

    #[test]
    fn test_handle_input_shift_g() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
            create_test_issue("TEST-3", "Third"),
        ]);
        // Mark as no more pages so navigation doesn't trigger LoadMore
        view.pagination.has_more = false;

        // G (shift+g) should move to end
        let key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert_eq!(view.selected, 2);
    }

    #[test]
    fn test_handle_input_enter() {
        let mut view = ListView::new();
        view.set_issues(vec![create_test_issue("TEST-1", "First issue")]);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert_eq!(action, Some(ListAction::OpenIssue("TEST-1".to_string())));
    }

    #[test]
    fn test_handle_input_refresh() {
        let mut view = ListView::new();

        let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert_eq!(action, Some(ListAction::Refresh));
    }

    #[test]
    fn test_handle_input_filter() {
        let mut view = ListView::new();

        let key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert_eq!(action, Some(ListAction::OpenFilter));
    }

    #[test]
    fn test_selected_issue() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
        ]);

        assert_eq!(view.selected_issue().unwrap().key, "TEST-1");

        view.move_down();
        assert_eq!(view.selected_issue().unwrap().key, "TEST-2");
    }

    #[test]
    fn test_loading_state() {
        let mut view = ListView::new();
        assert!(!view.is_loading());

        view.set_loading(true);
        assert!(view.is_loading());

        view.set_loading(false);
        assert!(!view.is_loading());
    }

    #[test]
    fn test_profile_name() {
        let mut view = ListView::new();
        assert!(view.profile_name.is_none());

        view.set_profile_name(Some("work".to_string()));
        assert_eq!(view.profile_name, Some("work".to_string()));
    }

    #[test]
    fn test_empty_list_navigation() {
        let mut view = ListView::new();
        // These should not panic on empty list
        view.move_down();
        view.move_up();
        view.move_to_start();
        view.move_to_end();
        view.page_down();
        view.page_up();
        assert_eq!(view.selected, 0);
    }

    #[test]
    fn test_handle_input_jql_colon() {
        let mut view = ListView::new();

        // ':' key opens JQL input
        let key = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert_eq!(action, Some(ListAction::OpenJqlInput));
    }

    #[test]
    fn test_handle_input_jql_slash() {
        let mut view = ListView::new();

        // '/' key opens JQL input
        let key = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert_eq!(action, Some(ListAction::OpenJqlInput));
    }

    // ========================================================================
    // Sort Tests
    // ========================================================================

    #[test]
    fn test_sort_column_to_jql_field() {
        assert_eq!(SortColumn::Key.to_jql_field(), "key");
        assert_eq!(SortColumn::Summary.to_jql_field(), "summary");
        assert_eq!(SortColumn::Status.to_jql_field(), "status");
        assert_eq!(SortColumn::Assignee.to_jql_field(), "assignee");
        assert_eq!(SortColumn::Priority.to_jql_field(), "priority");
    }

    #[test]
    fn test_sort_column_display_name() {
        assert_eq!(SortColumn::Key.display_name(), "Key");
        assert_eq!(SortColumn::Summary.display_name(), "Summary");
        assert_eq!(SortColumn::Status.display_name(), "Status");
        assert_eq!(SortColumn::Assignee.display_name(), "Assignee");
        assert_eq!(SortColumn::Priority.display_name(), "Priority");
    }

    #[test]
    fn test_sort_column_index() {
        assert_eq!(SortColumn::Key.index(), 0);
        assert_eq!(SortColumn::Summary.index(), 1);
        assert_eq!(SortColumn::Status.index(), 2);
        assert_eq!(SortColumn::Assignee.index(), 3);
        assert_eq!(SortColumn::Priority.index(), 4);
    }

    #[test]
    fn test_sort_column_from_index() {
        assert_eq!(SortColumn::from_index(0), Some(SortColumn::Key));
        assert_eq!(SortColumn::from_index(1), Some(SortColumn::Summary));
        assert_eq!(SortColumn::from_index(2), Some(SortColumn::Status));
        assert_eq!(SortColumn::from_index(3), Some(SortColumn::Assignee));
        assert_eq!(SortColumn::from_index(4), Some(SortColumn::Priority));
        assert_eq!(SortColumn::from_index(5), None);
    }

    #[test]
    fn test_sort_direction_toggle() {
        let asc = SortDirection::Ascending;
        assert_eq!(asc.toggle(), SortDirection::Descending);

        let desc = SortDirection::Descending;
        assert_eq!(desc.toggle(), SortDirection::Ascending);
    }

    #[test]
    fn test_sort_direction_to_jql() {
        assert_eq!(SortDirection::Ascending.to_jql(), "ASC");
        assert_eq!(SortDirection::Descending.to_jql(), "DESC");
    }

    #[test]
    fn test_sort_direction_indicator() {
        assert_eq!(SortDirection::Ascending.indicator(), " ▲");
        assert_eq!(SortDirection::Descending.indicator(), " ▼");
    }

    #[test]
    fn test_sort_state_default() {
        let state = SortState::default();
        assert_eq!(state.column, SortColumn::Key);
        assert_eq!(state.direction, SortDirection::Descending);
    }

    #[test]
    fn test_sort_state_to_jql() {
        let state = SortState::new(SortColumn::Key, SortDirection::Descending);
        assert_eq!(state.to_jql(), "ORDER BY key DESC");

        let state = SortState::new(SortColumn::Status, SortDirection::Ascending);
        assert_eq!(state.to_jql(), "ORDER BY status ASC");
    }

    #[test]
    fn test_sort_state_toggle_column_same() {
        let mut state = SortState::new(SortColumn::Key, SortDirection::Descending);
        state.toggle_column(SortColumn::Key);

        assert_eq!(state.column, SortColumn::Key);
        assert_eq!(state.direction, SortDirection::Ascending);
    }

    #[test]
    fn test_sort_state_toggle_column_different() {
        let mut state = SortState::new(SortColumn::Key, SortDirection::Ascending);
        state.toggle_column(SortColumn::Status);

        assert_eq!(state.column, SortColumn::Status);
        assert_eq!(state.direction, SortDirection::Descending);
    }

    #[test]
    fn test_handle_input_enter_sort_mode() {
        let mut view = ListView::new();
        view.set_issues(vec![create_test_issue("TEST-1", "First")]);

        // 's' key enters header/sort mode
        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        let action = view.handle_input(key);
        assert!(action.is_none());
        assert!(view.is_header_focused());
    }

    #[test]
    fn test_handle_input_header_mode_navigation() {
        let mut view = ListView::new();
        view.set_issues(vec![create_test_issue("TEST-1", "First")]);
        view.enter_header_mode();

        // Start at current sort column (Key = 0)
        assert_eq!(view.focused_column, 0);

        // 'l' moves right
        let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
        view.handle_input(key);
        assert_eq!(view.focused_column, 1);

        // 'h' moves left
        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        view.handle_input(key);
        assert_eq!(view.focused_column, 0);

        // 'h' at 0 stays at 0
        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        view.handle_input(key);
        assert_eq!(view.focused_column, 0);
    }

    #[test]
    fn test_handle_input_header_mode_select() {
        let mut view = ListView::new();
        view.set_issues(vec![create_test_issue("TEST-1", "First")]);
        view.enter_header_mode();

        // Move to Status column (index 2)
        view.focused_column = 2;

        // Enter selects the column
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert_eq!(action, Some(ListAction::SortChanged));
        assert_eq!(view.sort.column, SortColumn::Status);
        assert!(!view.is_header_focused());
    }

    #[test]
    fn test_handle_input_header_mode_cancel() {
        let mut view = ListView::new();
        view.set_issues(vec![create_test_issue("TEST-1", "First")]);
        view.enter_header_mode();

        // Esc cancels
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = view.handle_input(key);

        assert!(action.is_none());
        assert!(!view.is_header_focused());
    }

    // ========================================================================
    // Pagination Tests
    // ========================================================================

    #[test]
    fn test_pagination_state_default() {
        let state = PaginationState::new();
        assert_eq!(state.page_size, PaginationState::DEFAULT_PAGE_SIZE);
        assert_eq!(state.current_offset, 0);
        assert_eq!(state.total, 0);
        assert!(!state.loading);
        assert!(state.has_more);
    }

    #[test]
    fn test_pagination_state_with_page_size() {
        let state = PaginationState::with_page_size(25);
        assert_eq!(state.page_size, 25);
    }

    #[test]
    fn test_pagination_state_update_from_response() {
        let mut state = PaginationState::new();
        state.update_from_response(0, 50, 100);

        assert_eq!(state.current_offset, 50);
        assert_eq!(state.total, 100);
        assert!(state.has_more);
        assert!(!state.loading);
    }

    #[test]
    fn test_pagination_state_update_last_page() {
        let mut state = PaginationState::new();
        state.update_from_response(50, 50, 100);

        assert_eq!(state.current_offset, 100);
        assert_eq!(state.total, 100);
        assert!(!state.has_more);
    }

    #[test]
    fn test_pagination_state_reset() {
        let mut state = PaginationState::new();
        state.update_from_response(50, 50, 100);
        state.reset();

        assert_eq!(state.current_offset, 0);
        assert_eq!(state.total, 0);
        assert!(state.has_more);
    }

    #[test]
    fn test_pagination_state_display() {
        let mut state = PaginationState::new();
        assert_eq!(state.display(), "No issues");

        state.update_from_response(0, 50, 100);
        assert_eq!(state.display(), "1-50 of 100");
    }

    #[test]
    fn test_check_load_more_not_near_end() {
        let mut view = ListView::new();
        let issues: Vec<Issue> = (0..50)
            .map(|i| create_test_issue(&format!("TEST-{}", i), &format!("Issue {}", i)))
            .collect();
        view.set_issues(issues);
        view.pagination.update_from_response(0, 50, 100);

        // At the start, no need to load more
        view.selected = 0;
        let action = view.check_load_more();
        assert!(action.is_none());
    }

    #[test]
    fn test_check_load_more_near_end() {
        let mut view = ListView::new();
        let issues: Vec<Issue> = (0..50)
            .map(|i| create_test_issue(&format!("TEST-{}", i), &format!("Issue {}", i)))
            .collect();
        view.set_issues(issues);
        view.pagination.update_from_response(0, 50, 100);

        // Near the end, should trigger load more
        view.selected = 46; // Within 5 of 50
        let action = view.check_load_more();
        assert_eq!(action, Some(ListAction::LoadMore));
    }

    #[test]
    fn test_check_load_more_no_more_pages() {
        let mut view = ListView::new();
        let issues: Vec<Issue> = (0..50)
            .map(|i| create_test_issue(&format!("TEST-{}", i), &format!("Issue {}", i)))
            .collect();
        view.set_issues(issues);
        view.pagination.update_from_response(0, 50, 50); // Last page

        view.selected = 48;
        let action = view.check_load_more();
        assert!(action.is_none());
    }

    #[test]
    fn test_check_load_more_already_loading() {
        let mut view = ListView::new();
        let issues: Vec<Issue> = (0..50)
            .map(|i| create_test_issue(&format!("TEST-{}", i), &format!("Issue {}", i)))
            .collect();
        view.set_issues(issues);
        view.pagination.update_from_response(0, 50, 100);
        view.pagination.loading = true;

        view.selected = 48;
        let action = view.check_load_more();
        assert!(action.is_none());
    }

    #[test]
    fn test_append_issues() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
        ]);
        view.pagination.start_loading();

        view.append_issues(vec![
            create_test_issue("TEST-3", "Third"),
            create_test_issue("TEST-4", "Fourth"),
        ]);

        assert_eq!(view.issue_count(), 4);
        assert!(!view.pagination.loading);
    }

    #[test]
    fn test_reset_for_new_query() {
        let mut view = ListView::new();
        view.set_issues(vec![
            create_test_issue("TEST-1", "First"),
            create_test_issue("TEST-2", "Second"),
        ]);
        view.selected = 1;
        view.pagination.update_from_response(0, 50, 100);

        view.reset_for_new_query();

        assert!(view.issues.is_empty());
        assert_eq!(view.selected, 0);
        assert_eq!(view.pagination.current_offset, 0);
        assert_eq!(view.pagination.total, 0);
    }
}
