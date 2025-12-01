# Task 4.3: Connect Panel Editors (Labels, Links, Comments)

**Documentation:** [Feature: Edit Issue UI Redesign](../[Feature]%20Edit%20Issue%20UI%20Redesign.md)
**Task Number:** 4.3
**Area:** Integration
**Estimated Effort:** M (4-6 hours)

## Description

Connect the panel-based editors for Labels, Links, and Comments to the field edit mode. When the user presses Enter on these fields, a panel opens (side or bottom) allowing interaction with the multi-value or list-based content.

## Acceptance Criteria

- [ ] Pressing Enter on Labels field opens tag editor panel
- [ ] Pressing Enter on Links field opens linked issues panel
- [ ] Pressing Enter on Comments field opens comments panel
- [ ] Panels overlay or split the detail view appropriately
- [ ] Esc in panel closes it and returns to field edit mode
- [ ] Changes in panels are reflected in the detail view
- [ ] Existing panel components are reused
- [ ] Field grid cursor position is preserved after panel closes

## Implementation Details

### Approach

1. **Add panel-related DetailActions**:
   ```rust
   pub enum DetailAction {
       // ... existing
       OpenLabelEditor,
       OpenLinksPanel,
       OpenCommentsPanel,
   }
   ```

2. **Map field activation to panel actions**:
   ```rust
   fn handle_activate_field(&mut self) -> Option<DetailAction> {
       if let Some(ref state) = self.field_edit_state {
           match state.current_field() {
               EditableField::Labels => Some(DetailAction::OpenLabelEditor),
               EditableField::Links => Some(DetailAction::OpenLinksPanel),
               EditableField::Comments => Some(DetailAction::OpenCommentsPanel),
               // ... other fields handled elsewhere
               _ => None,
           }
       } else {
           None
       }
   }
   ```

3. **Panel state management**:
   ```rust
   pub enum ActivePanel {
       Labels(LabelEditor),
       Links(LinkedIssuesPanel),
       Comments(CommentsPanel),
   }

   // In App or DetailView
   active_panel: Option<ActivePanel>,
   ```

4. **Handle Labels panel**:
   ```rust
   fn open_label_editor(&mut self) {
       // Fetch available labels
       let available_labels = self.fetch_labels().await;
       let current_labels = self.current_issue.fields.labels.clone();

       self.active_panel = Some(ActivePanel::Labels(
           LabelEditor::new(current_labels, available_labels)
       ));
   }

   fn handle_label_editor_result(&mut self, labels: Vec<String>) {
       // Update via API
       self.update_issue_labels(labels).await;
       self.active_panel = None;
   }
   ```

5. **Handle Links panel**:
   ```rust
   fn open_links_panel(&mut self) {
       let links = self.current_issue.fields.issue_links.clone();

       self.active_panel = Some(ActivePanel::Links(
           LinkedIssuesPanel::new(links)
       ));
   }

   // Links panel allows:
   // - View existing links
   // - Add new link (opens link type picker, then issue search)
   // - Remove existing link
   ```

6. **Handle Comments panel**:
   ```rust
   fn open_comments_panel(&mut self) {
       // Fetch comments (may already be loaded)
       let comments = self.fetch_comments().await;

       self.active_panel = Some(ActivePanel::Comments(
           CommentsPanel::new(comments)
       ));
   }

   // Comments panel allows:
   // - View existing comments
   // - Add new comment
   // - Edit own comments (if supported)
   ```

7. **Panel rendering layout**:
   ```rust
   fn render(&self, frame: &mut Frame, area: Rect) {
       if let Some(ref panel) = self.active_panel {
           // Split layout: detail on left/top, panel on right/bottom
           let chunks = Layout::default()
               .direction(Direction::Horizontal)
               .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
               .split(area);

           self.detail_view.render(frame, chunks[0]);
           panel.render(frame, chunks[1]);
       } else {
           self.detail_view.render(frame, area);
       }
   }
   ```

8. **Panel input handling**:
   ```rust
   fn handle_input(&mut self, key: KeyEvent) {
       if let Some(ref mut panel) = self.active_panel {
           match key.code {
               KeyCode::Esc => {
                   self.active_panel = None;
                   return;
               }
               _ => {
                   if let Some(result) = panel.handle_input(key) {
                       self.handle_panel_result(result);
                   }
                   return;
               }
           }
       }

       // Handle detail view input
       if let Some(action) = self.detail_view.handle_input(key) {
           self.handle_detail_action(action);
       }
   }
   ```

### Files to Modify/Create

- `src/ui/views/detail.rs`:
  - Map Labels/Links/Comments to panel actions in ActivateField handler
- `src/app.rs`:
  - Add `active_panel: Option<ActivePanel>` state
  - Handle panel opening actions
  - Manage panel input priority
  - Handle panel results (label updates, link changes, new comments)
- `src/ui/components/label_editor.rs`: Ensure compatible or create tag editor
- `src/ui/components/linked_issues.rs`: Ensure compatible or enhance
- `src/ui/components/comments_panel.rs`: Ensure compatible or enhance

### Technical Specifications

- Panels render alongside detail view (split layout) or as overlay
- Panel state is managed at App level
- Field edit state persists while panel is open
- API calls for saving changes are async
- Each panel type has its own input handling logic

### Panel-Specific Features

**LabelEditor:**
- Display current labels as chips/tags
- Fuzzy search for adding new labels
- Remove labels with `x` or Delete key
- Enter or Ctrl+S to save changes

**LinkedIssuesPanel:**
- List of current links with issue key, summary, link type
- `a` to add new link (opens sub-modal for link type and issue)
- `d` to remove selected link
- Navigation with j/k

**CommentsPanel:**
- Scrollable list of comments with author and date
- `a` to add new comment (opens text editor)
- `e` to edit own comment
- Navigation with j/k

## Testing Requirements

- [ ] Test Enter on Labels opens label editor
- [ ] Test adding/removing labels works
- [ ] Test Esc in label editor returns to field edit mode
- [ ] Test Enter on Links opens linked issues panel
- [ ] Test viewing/adding/removing links works
- [ ] Test Enter on Comments opens comments panel
- [ ] Test viewing/adding comments works
- [ ] Test cursor position preserved after panel interaction
- [ ] Test panel state doesn't interfere with field edit state

## Dependencies

- **Prerequisite Tasks:** Task 3.1 (DetailView integration)
- **Blocks Tasks:** None
- **External:**
  - `LabelEditor` / `TagEditor` component (may need creation)
  - `LinkedIssuesPanel` component (existing or enhance)
  - `CommentsPanel` component (existing or enhance)
  - JIRA API for labels, links, comments

## Definition of Done

- [ ] All acceptance criteria met
- [ ] All three panel types integrate with field edit mode
- [ ] Panel layout is visually appropriate
- [ ] API integration works correctly
- [ ] Smooth UX with panel open/close transitions
- [ ] Code follows project standards
- [ ] Code reviewed and merged
