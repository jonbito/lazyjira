# Task 4.2: Connect Modal Pickers (Status, Priority, Assignee)

**Documentation:** [Feature: Edit Issue UI Redesign](../[Feature]%20Edit%20Issue%20UI%20Redesign.md)
**Task Number:** 4.2
**Area:** Integration
**Estimated Effort:** M (4-6 hours)

## Description

Connect the existing modal picker components (TransitionPicker, PriorityPicker, AssigneePicker) to the field edit mode. When the user presses Enter on Status, Priority, or Assignee fields, the appropriate modal picker opens.

## Acceptance Criteria

- [ ] Pressing Enter on Status field opens TransitionPicker modal
- [ ] Pressing Enter on Priority field opens PriorityPicker modal
- [ ] Pressing Enter on Assignee field opens AssigneePicker modal
- [ ] Modal pickers overlay the detail view (centered)
- [ ] Selecting an option in picker updates the field and calls API
- [ ] Esc in picker closes it and returns to field edit mode
- [ ] Existing picker components are reused without modification (if possible)
- [ ] Field grid cursor position is preserved after picker closes

## Implementation Details

### Approach

1. **Add picker-related DetailActions**:
   ```rust
   pub enum DetailAction {
       // ... existing
       OpenTransitionPicker,
       OpenPriorityPicker,
       OpenAssigneePicker,
       // These may already exist from current implementation
   }
   ```

2. **Map field activation to picker actions**:
   ```rust
   fn handle_activate_field(&mut self) -> Option<DetailAction> {
       if let Some(ref state) = self.field_edit_state {
           match state.current_field() {
               EditableField::Status => Some(DetailAction::OpenTransitionPicker),
               EditableField::Priority => Some(DetailAction::OpenPriorityPicker),
               EditableField::Assignee => Some(DetailAction::OpenAssigneePicker),
               // ... other fields handled in other tasks
               _ => None,
           }
       } else {
           None
       }
   }
   ```

3. **Handle picker opening in App**:
   ```rust
   // In app.rs update loop
   fn handle_detail_action(&mut self, action: DetailAction) {
       match action {
           DetailAction::OpenTransitionPicker => {
               // Fetch available transitions
               self.fetch_transitions();
               self.show_transition_picker = true;
           }
           DetailAction::OpenPriorityPicker => {
               self.show_priority_picker = true;
           }
           DetailAction::OpenAssigneePicker => {
               // Fetch assignable users
               self.fetch_assignable_users();
               self.show_assignee_picker = true;
           }
           // ... other actions
       }
   }
   ```

4. **Ensure picker close returns to field edit mode**:
   ```rust
   fn handle_transition_selected(&mut self, transition: Transition) {
       // Perform transition via API
       self.perform_transition(transition);
       // Close picker
       self.show_transition_picker = false;
       // Field edit mode should still be active
       // (field_edit_state is preserved)
   }

   fn handle_picker_cancelled(&mut self) {
       self.show_transition_picker = false;
       self.show_priority_picker = false;
       self.show_assignee_picker = false;
       // Return to field edit mode navigation
   }
   ```

5. **Render picker overlay**:
   ```rust
   fn render(&self, frame: &mut Frame, area: Rect) {
       // Render detail view (with field edit mode)
       self.detail_view.render(frame, area);

       // Render modal picker on top if active
       if self.show_transition_picker {
           let picker_area = centered_rect(60, 50, area);
           self.transition_picker.render(frame, picker_area);
       }
       // Similar for other pickers
   }
   ```

6. **Priority of input handling**:
   ```rust
   fn handle_input(&mut self, key: KeyEvent) {
       // Pickers take priority
       if self.show_transition_picker {
           self.handle_transition_picker_input(key);
           return;
       }
       if self.show_priority_picker {
           self.handle_priority_picker_input(key);
           return;
       }
       if self.show_assignee_picker {
           self.handle_assignee_picker_input(key);
           return;
       }

       // Then field edit mode
       if let Some(action) = self.detail_view.handle_input(key) {
           self.handle_detail_action(action);
       }
   }
   ```

### Files to Modify/Create

- `src/ui/views/detail.rs`:
  - Map Status/Priority/Assignee to picker actions in ActivateField handler
- `src/app.rs`:
  - Handle picker opening actions
  - Manage picker state (show/hide)
  - Coordinate input priority
- `src/ui/components/transition_picker.rs`: Ensure compatible (likely no changes)
- `src/ui/components/priority_picker.rs`: Ensure compatible (likely no changes)
- `src/ui/components/assignee_picker.rs`: Ensure compatible (likely no changes)

### Technical Specifications

- Pickers are rendered as modal overlays (centered, with backdrop optional)
- Picker state is managed at App level, not DetailView level
- Field edit state persists while picker is open
- API calls for fetching options and applying changes are async
- Existing picker keybindings (j/k to navigate, Enter to select, Esc to close) preserved

## Testing Requirements

- [ ] Test Enter on Status opens TransitionPicker
- [ ] Test selecting transition updates status and calls API
- [ ] Test Esc in TransitionPicker returns to field edit mode
- [ ] Test Enter on Priority opens PriorityPicker
- [ ] Test selecting priority updates field and calls API
- [ ] Test Enter on Assignee opens AssigneePicker
- [ ] Test selecting assignee updates field and calls API
- [ ] Test cursor position preserved after picker interaction
- [ ] Test multiple picker open/close cycles work correctly

## Dependencies

- **Prerequisite Tasks:** Task 3.1 (DetailView integration)
- **Blocks Tasks:** None
- **External:**
  - `TransitionPicker` component (existing)
  - `PriorityPicker` component (existing)
  - `AssigneePicker` component (existing)
  - JIRA API endpoints for transitions, priorities, users

## Definition of Done

- [ ] All acceptance criteria met
- [ ] All three pickers integrate with field edit mode
- [ ] Smooth user experience with picker open/close
- [ ] API integration works correctly
- [ ] No regression in existing picker functionality
- [ ] Code follows project standards
- [ ] Code reviewed and merged
