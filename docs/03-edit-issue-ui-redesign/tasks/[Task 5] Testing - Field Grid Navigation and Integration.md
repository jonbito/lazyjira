# Task 5: Field Grid Navigation and Integration Tests

**Documentation:** [Feature: Edit Issue UI Redesign](../[Feature]%20Edit%20Issue%20UI%20Redesign.md)
**Task Number:** 5
**Area:** Testing
**Estimated Effort:** M (4-6 hours)

## Description

Write comprehensive tests for the field grid navigation system and integration with editors. This includes unit tests for the core types and navigation logic, as well as integration tests for the complete field edit mode flow.

## Acceptance Criteria

- [ ] Unit tests for `EditableField` enum (labels, behaviors)
- [ ] Unit tests for `FieldGrid` creation and structure
- [ ] Unit tests for all navigation methods (move_left/right/up/down)
- [ ] Unit tests for navigation edge cases (boundaries, preferred column)
- [ ] Unit tests for `FieldEditState` (creation, editor state)
- [ ] Integration tests for mode entry/exit
- [ ] Integration tests for field activation triggering correct actions
- [ ] All tests pass with `cargo test`
- [ ] Test coverage for critical paths

## Implementation Details

### Approach

1. **Create test module** in `src/ui/field_grid.rs`:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_editable_field_labels() {
           assert_eq!(EditableField::Summary.label(), "Summary");
           assert_eq!(EditableField::Description.label(), "Description");
           assert_eq!(EditableField::Status.label(), "Status");
           // ... all fields
       }

       #[test]
       fn test_editable_field_behaviors() {
           assert_eq!(EditableField::Summary.edit_behavior(), EditBehavior::InlineText);
           assert_eq!(EditableField::Description.edit_behavior(), EditBehavior::InlineMultiline);
           assert_eq!(EditableField::Status.edit_behavior(), EditBehavior::ModalPicker);
           assert_eq!(EditableField::Labels.edit_behavior(), EditBehavior::Panel);
           // ... all fields
       }

       #[test]
       fn test_field_grid_creation() {
           let grid = FieldGrid::new();
           assert_eq!(grid.row_count(), 7); // Adjust based on actual layout
           assert_eq!(grid.current_field(), EditableField::Summary);
           assert_eq!(grid.position(), (0, 0));
       }

       #[test]
       fn test_field_grid_row_structure() {
           let grid = FieldGrid::new();
           assert_eq!(grid.col_count(0), 1); // Summary row
           assert_eq!(grid.col_count(1), 2); // Assignee, Status row
       }
   }
   ```

2. **Navigation tests**:
   ```rust
   #[test]
   fn test_move_right_single_column_row() {
       let mut grid = FieldGrid::new();
       // Row 0 has only Summary
       assert!(!grid.move_right()); // Should return false at edge
       assert_eq!(grid.position(), (0, 0));
   }

   #[test]
   fn test_move_right_multi_column_row() {
       let mut grid = FieldGrid::new();
       grid.move_down(); // Move to row with Assignee, Status
       assert!(grid.move_right());
       assert_eq!(grid.current_field(), EditableField::Status);
   }

   #[test]
   fn test_move_left_at_edge() {
       let mut grid = FieldGrid::new();
       assert!(!grid.move_left());
       assert_eq!(grid.position(), (0, 0));
   }

   #[test]
   fn test_move_down_to_end() {
       let mut grid = FieldGrid::new();
       let row_count = grid.row_count();
       for _ in 0..row_count {
           grid.move_down();
       }
       assert!(!grid.move_down()); // At last row
   }

   #[test]
   fn test_move_up_at_start() {
       let grid = FieldGrid::new();
       assert!(!grid.move_up());
   }
   ```

3. **Preferred column tests**:
   ```rust
   #[test]
   fn test_preferred_column_preserved() {
       let mut grid = FieldGrid::new();
       // Move to multi-column row and go to second column
       grid.move_down(); // Now at Assignee
       grid.move_right(); // Now at Status, col 1
       assert_eq!(grid.col, 1);

       // Move down to single-column row
       grid.move_down(); // Priority (single column)
       assert_eq!(grid.col, 0); // Clamped to 0
       assert_eq!(grid.preferred_col, 1); // Still remembers 1

       // Move back up
       grid.move_up();
       assert_eq!(grid.col, 1); // Restored to preferred
       assert_eq!(grid.current_field(), EditableField::Status);
   }
   ```

4. **FieldEditState tests**:
   ```rust
   #[test]
   fn test_field_edit_state_creation() {
       let state = FieldEditState::new();
       assert!(!state.is_editing());
       assert_eq!(state.current_field(), EditableField::Summary);
   }

   #[test]
   fn test_field_edit_state_editor_management() {
       let mut state = FieldEditState::new();
       assert!(!state.is_editing());

       // Set active editor (simplified)
       state.active_editor = Some(ActiveEditor::Summary { /* ... */ });
       assert!(state.is_editing());

       state.close_editor();
       assert!(!state.is_editing());
   }
   ```

5. **Integration tests** (in separate test file or integration tests dir):
   ```rust
   #[test]
   fn test_detail_view_enter_field_edit_mode() {
       let mut detail = DetailView::new(/* mock issue */);
       assert!(!detail.is_in_field_edit_mode());

       detail.enter_field_edit_mode();
       assert!(detail.is_in_field_edit_mode());
       assert_eq!(detail.current_field(), Some(EditableField::Summary));
   }

   #[test]
   fn test_detail_view_exit_field_edit_mode() {
       let mut detail = DetailView::new(/* mock issue */);
       detail.enter_field_edit_mode();
       detail.exit_field_edit_mode();
       assert!(!detail.is_in_field_edit_mode());
   }

   #[test]
   fn test_field_activation_returns_correct_action() {
       let mut detail = DetailView::new(/* mock issue */);
       detail.enter_field_edit_mode();

       // Navigate to Status
       detail.field_edit_state.as_mut().unwrap().grid.move_down();

       // Simulate Enter
       let action = detail.handle_input(KeyEvent::from(KeyCode::Enter));
       // Should return action to open transition picker
       assert!(matches!(action, Some(DetailAction::OpenTransitionPicker)));
   }
   ```

### Files to Modify/Create

- `src/ui/field_grid.rs`: Add `#[cfg(test)] mod tests { ... }`
- `src/ui/views/detail.rs`: Add tests for field edit mode
- `tests/field_grid_integration.rs`: Integration tests (if using tests/ dir)

### Test Categories

| Category | Description | Priority |
|----------|-------------|----------|
| Unit: Types | EditableField enum, EditBehavior | High |
| Unit: Grid Structure | FieldGrid creation, row/col counts | High |
| Unit: Navigation | All move_* methods, edge cases | High |
| Unit: Preferred Column | Column memory across rows | Medium |
| Unit: State | FieldEditState, ActiveEditor | High |
| Integration: Mode | Entry/exit field edit mode | High |
| Integration: Actions | Field activation → correct action | High |
| Integration: Flow | Complete edit workflow | Medium |

## Testing Requirements

- [ ] Run `cargo test` - all tests pass
- [ ] Run `cargo test -- --nocapture` to verify test output
- [ ] Check test coverage for core navigation logic
- [ ] Verify no regressions in existing tests

## Dependencies

- **Prerequisite Tasks:** Task 2.1, Task 2.2, Task 3.1 (implementations to test)
- **Blocks Tasks:** None (testing can happen in parallel with Task 4.x and 6)
- **External:** None

## Definition of Done

- [ ] All acceptance criteria met
- [ ] All tests pass
- [ ] Coverage of critical navigation edge cases
- [ ] Tests are well-documented with clear assertions
- [ ] Tests run in reasonable time (< 1 second for unit tests)
- [ ] Code follows project standards
- [ ] Code reviewed and merged
