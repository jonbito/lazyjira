# Task 2.1: FieldGrid and EditableField Types

**Documentation:** [Feature: Edit Issue UI Redesign](../[Feature]%20Edit%20Issue%20UI%20Redesign.md)
**Task Number:** 2.1
**Area:** Core/Backend
**Estimated Effort:** M (4-6 hours)

## Description

Implement the core data types for the field grid system: `EditableField` enum defining all editable fields with their edit behaviors, and `FieldGrid` struct managing the 2D spatial layout and cursor position.

## Acceptance Criteria

- [ ] `EditableField` enum implemented with all 8 editable fields (Summary, Description, Status, Priority, Assignee, Labels, Links, Comments)
- [ ] `FieldGrid` struct implemented with 2D grid storage
- [ ] Grid initialized with correct field positions matching the layout spec
- [ ] `current_field()` method returns the currently focused field
- [ ] Read-only fields are excluded from the grid
- [ ] Unit tests for grid creation and field access
- [ ] Code follows project standards (cargo fmt, cargo clippy)

## Implementation Details

### Approach

1. **Create new module** `src/ui/field_grid.rs` (or within detail view module)

2. **Implement EditableField enum**:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum EditableField {
       Summary,
       Description,
       Status,
       Priority,
       Assignee,
       Labels,
       Links,
       Comments,
   }

   impl EditableField {
       /// Returns the display label for this field
       pub fn label(&self) -> &'static str {
           match self {
               Self::Summary => "Summary",
               Self::Description => "Description",
               Self::Status => "Status",
               Self::Priority => "Priority",
               Self::Assignee => "Assignee",
               Self::Labels => "Labels",
               Self::Links => "Links",
               Self::Comments => "Comments",
           }
       }

       /// Returns the edit behavior type for this field
       pub fn edit_behavior(&self) -> EditBehavior {
           match self {
               Self::Summary => EditBehavior::InlineText,
               Self::Description => EditBehavior::InlineMultiline,
               Self::Status | Self::Priority | Self::Assignee => EditBehavior::ModalPicker,
               Self::Labels | Self::Links | Self::Comments => EditBehavior::Panel,
           }
       }
   }

   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum EditBehavior {
       InlineText,      // Single-line text input
       InlineMultiline, // Multi-line text editor
       ModalPicker,     // Centered modal with options
       Panel,           // Side/bottom panel
   }
   ```

3. **Implement FieldGrid struct**:
   ```rust
   pub struct FieldGrid {
       /// 2D grid of fields, rows may have different column counts
       rows: Vec<Vec<EditableField>>,
       /// Current row position
       row: usize,
       /// Current column position
       col: usize,
       /// Preferred column (remembered when moving between rows of different widths)
       preferred_col: usize,
   }

   impl FieldGrid {
       pub fn new() -> Self {
           // Initialize with the layout from the feature spec
           let rows = vec![
               vec![EditableField::Summary],                              // Row 0: full width
               vec![EditableField::Assignee, EditableField::Status],      // Row 1: 2 editable
               vec![EditableField::Priority],                             // Row 4 in visual, but row 2 in grid
               vec![EditableField::Labels],                               // Row 7 in visual
               vec![EditableField::Links],                                // Row 8 in visual
               vec![EditableField::Description],                          // Row 9 in visual
               vec![EditableField::Comments],                             // Row 10 in visual
           ];

           Self {
               rows,
               row: 0,
               col: 0,
               preferred_col: 0,
           }
       }

       pub fn current_field(&self) -> EditableField {
           self.rows[self.row][self.col]
       }

       pub fn row_count(&self) -> usize {
           self.rows.len()
       }

       pub fn col_count(&self, row: usize) -> usize {
           self.rows.get(row).map(|r| r.len()).unwrap_or(0)
       }
   }
   ```

### Files to Modify/Create

- `src/ui/field_grid.rs` (new): Core FieldGrid and EditableField types
- `src/ui/mod.rs`: Export new field_grid module

### Technical Specifications

- All types should derive `Debug`, `Clone`
- `EditableField` should derive `Copy`, `PartialEq`, `Eq` for easy comparisons
- Grid layout matches the visual spec but only includes editable fields
- No dependencies on external crates beyond std

## Testing Requirements

- [ ] Test `FieldGrid::new()` creates correct initial state
- [ ] Test `current_field()` returns Summary at initialization
- [ ] Test `row_count()` returns correct number of rows
- [ ] Test `col_count()` returns correct columns per row
- [ ] Test `EditableField::label()` returns correct labels
- [ ] Test `EditableField::edit_behavior()` returns correct behaviors

## Dependencies

- **Prerequisite Tasks:** Task 1 (architecture design)
- **Blocks Tasks:** Task 2.2 (navigation logic), Task 3.1 (DetailView integration)
- **External:** None

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Code compiles without warnings
- [ ] `cargo fmt` applied
- [ ] `cargo clippy` passes
- [ ] Unit tests written and passing
- [ ] Code reviewed and merged
