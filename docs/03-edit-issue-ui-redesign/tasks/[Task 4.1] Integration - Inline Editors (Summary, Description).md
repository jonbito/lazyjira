# Task 4.1: Connect Inline Editors (Summary, Description)

**Documentation:** [Feature: Edit Issue UI Redesign](../[Feature]%20Edit%20Issue%20UI%20Redesign.md)
**Task Number:** 4.1
**Area:** Integration
**Estimated Effort:** M (4-6 hours)

## Description

Integrate inline text editors for Summary and Description fields within the field edit mode. When the user presses Enter on these fields, an inline editor appears replacing the field content, allowing direct text editing without leaving the detail view.

## Acceptance Criteria

- [ ] Pressing Enter on Summary field opens inline single-line TextInput
- [ ] Pressing Enter on Description field opens inline multi-line TextEditor (or triggers external editor)
- [ ] TextInput/TextEditor appears in place of the field value
- [ ] Esc closes the inline editor without saving
- [ ] Enter (in Summary) or Ctrl+S/Esc (in Description) saves changes
- [ ] Changes are sent to JIRA API via `IssueUpdateRequest`
- [ ] Field value updates after successful save
- [ ] Error handling for API failures
- [ ] Existing TextInput and TextEditor components are reused

## Implementation Details

### Approach

1. **Update ActiveEditor enum** with actual editor state:
   ```rust
   pub enum ActiveEditor {
       Summary {
           input: TextInput,
           original_value: String,
       },
       Description {
           editor: TextEditor,
           original_value: String,
       },
   }
   ```

2. **Implement Summary editor activation**:
   ```rust
   impl DetailView {
       pub fn open_summary_editor(&mut self) {
           if let Some(ref mut state) = self.field_edit_state {
               let current_summary = self.issue.fields.summary.clone();
               let mut input = TextInput::new();
               input.set_value(&current_summary);

               state.active_editor = Some(ActiveEditor::Summary {
                   input,
                   original_value: current_summary,
               });
           }
       }
   }
   ```

3. **Implement Description editor activation**:
   ```rust
   pub fn open_description_editor(&mut self) {
       if let Some(ref mut state) = self.field_edit_state {
           let current_desc = self.issue.fields.description.clone().unwrap_or_default();

           // Option 1: Inline multi-line editor
           let mut editor = TextEditor::new();
           editor.set_content(&current_desc);

           state.active_editor = Some(ActiveEditor::Description {
               editor,
               original_value: current_desc,
           });

           // Option 2: External editor (if configured)
           // self.open_external_editor(&current_desc);
       }
   }
   ```

4. **Handle editor input**:
   ```rust
   fn handle_editor_input(&mut self, key: KeyEvent) -> Option<DetailAction> {
       if let Some(ref mut state) = self.field_edit_state {
           match &mut state.active_editor {
               Some(ActiveEditor::Summary { ref mut input, .. }) => {
                   match key.code {
                       KeyCode::Enter => {
                           // Save and close
                           let new_value = input.value().to_string();
                           return Some(DetailAction::SaveSummary(new_value));
                       }
                       KeyCode::Esc => {
                           state.close_editor();
                           return None;
                       }
                       _ => {
                           input.handle_input(key);
                           return None;
                       }
                   }
               }
               Some(ActiveEditor::Description { ref mut editor, .. }) => {
                   match (key.code, key.modifiers) {
                       (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                           let new_value = editor.content().to_string();
                           return Some(DetailAction::SaveDescription(new_value));
                       }
                       (KeyCode::Esc, _) => {
                           state.close_editor();
                           return None;
                       }
                       _ => {
                           editor.handle_input(key);
                           return None;
                       }
                   }
               }
               None => {}
           }
       }
       None
   }
   ```

5. **Add new DetailAction variants**:
   ```rust
   pub enum DetailAction {
       // ... existing
       SaveSummary(String),
       SaveDescription(String),
   }
   ```

6. **Handle save actions in App**:
   ```rust
   // In app.rs or wherever actions are processed
   async fn handle_save_summary(&mut self, new_summary: String) {
       let update = IssueUpdateRequest {
           fields: Some(IssueFieldsUpdate {
               summary: Some(new_summary.clone()),
               ..Default::default()
           }),
           ..Default::default()
       };

       match self.api_client.update_issue(&self.current_issue.key, update).await {
           Ok(_) => {
               self.current_issue.fields.summary = new_summary;
               self.show_notification("Summary updated");
           }
           Err(e) => {
               self.show_error(&format!("Failed to update summary: {}", e));
           }
       }

       // Close editor
       if let Some(ref mut detail) = self.detail_view {
           if let Some(ref mut state) = detail.field_edit_state {
               state.close_editor();
           }
       }
   }
   ```

7. **Render inline editor**:
   ```rust
   fn render_summary_field(&self, frame: &mut Frame, area: Rect) {
       if let Some(ref state) = self.field_edit_state {
           if let Some(ActiveEditor::Summary { ref input, .. }) = state.active_editor {
               // Render TextInput widget in the summary area
               input.render(frame, area);
               return;
           }
       }

       // Normal summary rendering
       let summary = Paragraph::new(&self.issue.fields.summary);
       frame.render_widget(summary, area);
   }
   ```

### Files to Modify/Create

- `src/ui/views/detail.rs`:
  - Add editor activation methods
  - Update handle_input for editor input handling
  - Update render for inline editor display
  - Add SaveSummary, SaveDescription actions
- `src/ui/field_grid.rs`: Update ActiveEditor enum
- `src/app.rs`: Handle new save actions with API calls
- `src/ui/components/input.rs`: Ensure TextInput is compatible
- `src/ui/components/editor.rs`: Ensure TextEditor is compatible (if exists)

### Technical Specifications

- TextInput should support cursor movement, character input, delete, backspace
- TextEditor should support multi-line editing with line wrapping
- API calls should be async and non-blocking
- Original value is stored for cancel/revert functionality
- Notification system used for success/error feedback

## Testing Requirements

- [ ] Test Enter on Summary opens TextInput with current value
- [ ] Test typing in Summary TextInput updates content
- [ ] Test Enter in Summary saves and closes editor
- [ ] Test Esc in Summary closes without saving
- [ ] Test Enter on Description opens TextEditor with current value
- [ ] Test Ctrl+S in Description saves and closes editor
- [ ] Test Esc in Description closes without saving
- [ ] Test API error handling shows error notification
- [ ] Test successful save updates displayed field value

## Dependencies

- **Prerequisite Tasks:** Task 3.1 (DetailView integration), Task 3.2 (rendering)
- **Blocks Tasks:** None
- **External:**
  - `TextInput` component (existing)
  - `TextEditor` component (existing or to be created)
  - JIRA API `update_issue` endpoint

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Both Summary and Description inline editing works
- [ ] API integration complete with error handling
- [ ] No regression in existing edit functionality
- [ ] Code follows project standards
- [ ] Code reviewed and merged
