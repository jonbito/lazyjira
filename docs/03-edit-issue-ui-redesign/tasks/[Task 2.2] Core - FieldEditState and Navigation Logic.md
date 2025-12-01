# Task 2.2: FieldEditState and Navigation Logic

**Documentation:** [Feature: Edit Issue UI Redesign](../[Feature]%20Edit%20Issue%20UI%20Redesign.md)
**Task Number:** 2.2
**Area:** Core/Backend
**Estimated Effort:** M (4-6 hours)

## Description

Implement the navigation logic for the field grid (hjkl movement methods) and the `FieldEditState` struct that tracks the current field navigation state and active editor.

## Acceptance Criteria

- [ ] `move_left()` method moves cursor left, returns false if at edge
- [ ] `move_right()` method moves cursor right, returns false if at edge
- [ ] `move_up()` method moves cursor up, respects preferred column
- [ ] `move_down()` method moves cursor down, respects preferred column
- [ ] Preferred column is remembered when moving between rows of different widths
- [ ] `FieldEditState` struct tracks grid and active editor
- [ ] `ActiveEditor` enum defines all inline editor variants
- [ ] Unit tests for all navigation edge cases
- [ ] Code follows project standards

## Implementation Details

### Approach

1. **Implement navigation methods on FieldGrid**:
   ```rust
   impl FieldGrid {
       /// Move cursor left. Returns true if moved, false if at edge.
       pub fn move_left(&mut self) -> bool {
           if self.col > 0 {
               self.col -= 1;
               self.preferred_col = self.col;
               true
           } else {
               false
           }
       }

       /// Move cursor right. Returns true if moved, false if at edge.
       pub fn move_right(&mut self) -> bool {
           let max_col = self.rows[self.row].len().saturating_sub(1);
           if self.col < max_col {
               self.col += 1;
               self.preferred_col = self.col;
               true
           } else {
               false
           }
       }

       /// Move cursor up. Returns true if moved, false if at edge.
       /// Attempts to maintain preferred column position.
       pub fn move_up(&mut self) -> bool {
           if self.row > 0 {
               self.row -= 1;
               // Clamp column to new row's width, but remember preferred
               let max_col = self.rows[self.row].len().saturating_sub(1);
               self.col = self.preferred_col.min(max_col);
               true
           } else {
               false
           }
       }

       /// Move cursor down. Returns true if moved, false if at edge.
       /// Attempts to maintain preferred column position.
       pub fn move_down(&mut self) -> bool {
           if self.row + 1 < self.rows.len() {
               self.row += 1;
               // Clamp column to new row's width, but remember preferred
               let max_col = self.rows[self.row].len().saturating_sub(1);
               self.col = self.preferred_col.min(max_col);
               true
           } else {
               false
           }
       }

       /// Reset cursor to first field
       pub fn reset(&mut self) {
           self.row = 0;
           self.col = 0;
           self.preferred_col = 0;
       }

       /// Get current position as (row, col)
       pub fn position(&self) -> (usize, usize) {
           (self.row, self.col)
       }
   }
   ```

2. **Implement FieldEditState**:
   ```rust
   /// State for field edit mode in the detail view
   pub struct FieldEditState {
       /// The field grid managing layout and cursor
       pub grid: FieldGrid,
       /// Currently active inline editor (if any)
       pub active_editor: Option<ActiveEditor>,
   }

   /// Active inline editor variant
   pub enum ActiveEditor {
       /// Single-line text input for Summary
       Summary(TextInputState),
       /// Multi-line text editor for Description
       Description(TextEditorState),
   }

   impl FieldEditState {
       pub fn new() -> Self {
           Self {
               grid: FieldGrid::new(),
               active_editor: None,
           }
       }

       /// Returns the currently focused field
       pub fn current_field(&self) -> EditableField {
           self.grid.current_field()
       }

       /// Returns true if an inline editor is currently active
       pub fn is_editing(&self) -> bool {
           self.active_editor.is_some()
       }

       /// Close any active editor
       pub fn close_editor(&mut self) {
           self.active_editor = None;
       }
   }
   ```

3. **Define placeholder state types** (or use existing):
   ```rust
   // These may already exist or need to be imported from existing components
   pub struct TextInputState {
       // ... existing TextInput state
   }

   pub struct TextEditorState {
       // ... existing TextEditor state
   }
   ```

### Files to Modify/Create

- `src/ui/field_grid.rs`: Add navigation methods and FieldEditState
- May need to reference `src/ui/components/input.rs` for TextInput state type

### Technical Specifications

- Navigation methods return `bool` to allow UI feedback on edge cases
- Preferred column enables intuitive vertical navigation across rows of different widths
- ActiveEditor holds ownership of editor state during inline editing
- Modal/panel editors don't use ActiveEditor (they're handled by AppState transitions)

## Testing Requirements

- [ ] Test `move_left()` at column 0 returns false
- [ ] Test `move_left()` from column 1 moves to column 0
- [ ] Test `move_right()` at last column returns false
- [ ] Test `move_right()` from column 0 moves to column 1 (on multi-column row)
- [ ] Test `move_up()` at row 0 returns false
- [ ] Test `move_down()` at last row returns false
- [ ] Test preferred column: start at col 1, move down to single-col row, move down again restores col 1
- [ ] Test `reset()` moves cursor to (0, 0)
- [ ] Test `FieldEditState::new()` initializes correctly
- [ ] Test `is_editing()` returns false initially, true when editor set

## Dependencies

- **Prerequisite Tasks:** Task 2.1 (FieldGrid and EditableField types)
- **Blocks Tasks:** Task 3.1 (DetailView integration)
- **External:** None

## Definition of Done

- [ ] All acceptance criteria met
- [ ] All navigation edge cases handled correctly
- [ ] Unit tests written and passing
- [ ] Code compiles without warnings
- [ ] `cargo fmt` applied
- [ ] `cargo clippy` passes
- [ ] Code reviewed and merged
