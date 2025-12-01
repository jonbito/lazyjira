# Task 3.1: DetailView Field Edit Mode Integration

**Documentation:** [Feature: Edit Issue UI Redesign](../[Feature]%20Edit%20Issue%20UI%20Redesign.md)
**Task Number:** 3.1
**Area:** Frontend/UI
**Estimated Effort:** L (6-8 hours)

## Description

Integrate the field edit mode into the DetailView component. This includes adding field edit state management, handling keyboard input for mode entry/exit and hjkl navigation, and updating the DetailAction enum with new actions.

## Acceptance Criteria

- [ ] `field_edit_state: Option<FieldEditState>` added to DetailView
- [ ] `f` key enters field edit mode from normal detail view
- [ ] `Esc` key exits field edit mode (or closes active editor first)
- [ ] `h/j/k/l` keys navigate between fields when in field edit mode
- [ ] `Enter` key triggers appropriate action for focused field
- [ ] `DetailAction` enum updated with new actions
- [ ] State transitions are clean and predictable
- [ ] Existing functionality not regressed

## Implementation Details

### Approach

1. **Update DetailAction enum** in `src/ui/views/detail.rs`:
   ```rust
   pub enum DetailAction {
       // ... existing actions
       EnterFieldEditMode,
       ExitFieldEditMode,
       NavigateField(Direction),
       ActivateField,
   }

   #[derive(Debug, Clone, Copy)]
   pub enum Direction {
       Left,
       Right,
       Up,
       Down,
   }
   ```

2. **Add field edit state to DetailView**:
   ```rust
   pub struct DetailView {
       // ... existing fields
       field_edit_state: Option<FieldEditState>,
   }
   ```

3. **Update handle_input() method**:
   ```rust
   pub fn handle_input(&mut self, key: KeyEvent) -> Option<DetailAction> {
       // If field edit mode is active
       if let Some(ref mut state) = self.field_edit_state {
           // If an inline editor is active, delegate to it
           if state.is_editing() {
               return self.handle_editor_input(key);
           }

           // Handle field navigation
           match key.code {
               KeyCode::Char('h') => {
                   state.grid.move_left();
                   return None;
               }
               KeyCode::Char('j') => {
                   state.grid.move_down();
                   return None;
               }
               KeyCode::Char('k') => {
                   state.grid.move_up();
                   return None;
               }
               KeyCode::Char('l') => {
                   state.grid.move_right();
                   return None;
               }
               KeyCode::Enter => {
                   return Some(DetailAction::ActivateField);
               }
               KeyCode::Esc => {
                   return Some(DetailAction::ExitFieldEditMode);
               }
               _ => {}
           }
           return None;
       }

       // Normal detail view input handling
       match key.code {
           KeyCode::Char('f') => Some(DetailAction::EnterFieldEditMode),
           // ... existing keybindings
           _ => None,
       }
   }
   ```

4. **Implement mode entry/exit**:
   ```rust
   impl DetailView {
       pub fn enter_field_edit_mode(&mut self) {
           self.field_edit_state = Some(FieldEditState::new());
       }

       pub fn exit_field_edit_mode(&mut self) {
           self.field_edit_state = None;
       }

       pub fn is_in_field_edit_mode(&self) -> bool {
           self.field_edit_state.is_some()
       }

       pub fn current_field(&self) -> Option<EditableField> {
           self.field_edit_state.as_ref().map(|s| s.current_field())
       }
   }
   ```

5. **Handle ActivateField action** in App or DetailView:
   ```rust
   fn handle_activate_field(&mut self) {
       if let Some(ref state) = self.field_edit_state {
           match state.current_field() {
               EditableField::Summary => self.open_summary_editor(),
               EditableField::Description => self.open_description_editor(),
               EditableField::Status => self.open_transition_picker(),
               EditableField::Priority => self.open_priority_picker(),
               EditableField::Assignee => self.open_assignee_picker(),
               EditableField::Labels => self.open_label_editor(),
               EditableField::Links => self.open_links_panel(),
               EditableField::Comments => self.open_comments_panel(),
           }
       }
   }
   ```

### Files to Modify/Create

- `src/ui/views/detail.rs`: Main integration point
  - Add `field_edit_state` field
  - Add `Direction` enum
  - Update `DetailAction` enum
  - Update `handle_input()` method
  - Add mode entry/exit methods
- `src/ui/field_grid.rs`: Import types from here
- `src/app.rs`: Handle new DetailAction variants in update loop

### Technical Specifications

- Field edit mode is an overlay on the existing detail view state
- Modal pickers (Status, Priority, Assignee) will transition to their own AppState
- Inline editors (Summary, Description) will be handled within field edit state
- Panel editors (Labels, Links, Comments) may use existing panel infrastructure

## Testing Requirements

- [ ] Test `f` key enters field edit mode
- [ ] Test `Esc` exits field edit mode
- [ ] Test hjkl navigation updates focused field
- [ ] Test `Enter` on each field type triggers correct action
- [ ] Test mode state is correctly tracked
- [ ] Test no regression in existing `e` key edit mode

## Dependencies

- **Prerequisite Tasks:** Task 2.1, Task 2.2 (core types and navigation)
- **Blocks Tasks:** Task 3.2 (rendering), Task 4.1-4.3 (editor integrations)
- **External:** None

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Field edit mode can be entered and exited cleanly
- [ ] Navigation works correctly between all 8 editable fields
- [ ] Enter triggers appropriate placeholder action for each field
- [ ] No regression in existing functionality
- [ ] Code follows project standards
- [ ] Code reviewed and merged
