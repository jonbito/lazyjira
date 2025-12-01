# Task 1: Field Grid System Architecture

**Documentation:** [Feature: Edit Issue UI Redesign](../[Feature]%20Edit%20Issue%20UI%20Redesign.md)
**Task Number:** 1
**Area:** Research/Architecture
**Estimated Effort:** S (2-4 hours)

## Description

Research and design the field grid system architecture that will enable spatial navigation between editable fields in the issue detail view. This task establishes the foundational design patterns and data structures before implementation begins.

## Acceptance Criteria

- [x] Document the field grid layout structure (rows, columns, field positions)
- [x] Define the navigation algorithm for hjkl movement (including read-only field skipping)
- [x] Identify integration points with existing DetailView and editor components
- [x] Create type definitions for FieldGrid, EditableField, and FieldEditState
- [x] Document state transition diagram for field edit mode
- [x] Review existing picker/editor components for compatibility

## Implementation Details

### Approach

1. **Analyze current DetailView structure**
   - Review `src/ui/views/detail.rs` for current layout and state management
   - Identify how existing edit mode (`e` key) works
   - Map out current keyboard handling flow

2. **Design grid layout**
   - Define the 2D grid structure based on feature spec
   - Determine which fields are editable vs read-only
   - Plan column spanning for full-width fields

3. **Design navigation algorithm**
   - Implement "skip read-only" logic
   - Handle row transitions with varying column counts
   - Define edge behavior (wrap vs stop)

4. **Plan state management**
   - Design FieldEditState struct
   - Define ActiveEditor enum variants
   - Plan state transitions and Esc handling

5. **Review existing components**
   - Audit TextInput, TextEditor for inline editing
   - Audit TransitionPicker, PriorityPicker, AssigneePicker for modal integration
   - Audit TagEditor, LinkedIssues, CommentsPanel for panel integration

### Files to Review

- `src/ui/views/detail.rs`: Current detail view implementation
- `src/ui/components/`: All existing editor/picker components
- `src/events/keys.rs`: Current keybinding definitions
- `src/app.rs`: AppState and state transitions

### Deliverables

1. **Architecture Document** (can be added to this task file or separate doc):
   - Grid layout diagram
   - Navigation algorithm pseudocode
   - State machine diagram
   - Type definitions (Rust code blocks)
   - Component integration plan

## Testing Requirements

- [ ] N/A for research task - testing criteria defined for implementation tasks

## Dependencies

- **Prerequisite Tasks:** None (this is the first task)
- **Blocks Tasks:** Task 2.1, Task 2.2, Task 3.1, Task 3.2
- **External:** None

## Definition of Done

- [x] Architecture document completed
- [x] Navigation algorithm defined and edge cases documented
- [x] Type definitions reviewed and approved
- [x] Integration points identified for all 8 editable fields
- [x] State transition diagram completed
- [x] Ready for implementation to begin

## Notes

Key design decisions to document:

1. **Grid representation**: Should use a sparse representation since rows have varying column counts
2. **Field skipping**: Navigation should skip read-only fields automatically
3. **Column memory**: When moving vertically between rows of different widths, remember the original column preference
4. **Mode entry**: Use `f` key to enter field edit mode (not conflicting with existing bindings)
5. **Esc behavior**: First Esc closes active editor, second Esc exits field edit mode

---

## Architecture Document

### 1. Grid Layout Structure

The field grid organizes issue fields into a 2D spatial layout enabling hjkl-style navigation. Fields are arranged in rows with varying column counts.

```
┌─────────────────────────────────────────────────────────────┐
│ Row 0: [Summary]                                   (1 col)  │
├─────────────────────────────────────────────────────────────┤
│ Row 1: [Status]    [Priority]    [Assignee]       (3 cols)  │
├─────────────────────────────────────────────────────────────┤
│ Row 2: [Labels]    [Components]                   (2 cols)  │
├─────────────────────────────────────────────────────────────┤
│ Row 3: [Description]                               (1 col)  │
├─────────────────────────────────────────────────────────────┤
│ Row 4: [Comments]  [Linked Issues]                (2 cols)  │
└─────────────────────────────────────────────────────────────┘
```

**Field Properties:**

| Field         | Row | Col | Editable | Editor Type        | Span  |
|---------------|-----|-----|----------|-------------------|-------|
| Summary       | 0   | 0   | Yes      | Inline TextInput  | Full  |
| Status        | 1   | 0   | Yes      | Modal Picker      | 1/3   |
| Priority      | 1   | 1   | Yes      | Modal Picker      | 1/3   |
| Assignee      | 1   | 2   | Yes      | Modal Picker      | 1/3   |
| Labels        | 2   | 0   | Yes      | Modal TagEditor   | 1/2   |
| Components    | 2   | 1   | Yes      | Modal TagEditor   | 1/2   |
| Description   | 3   | 0   | Yes      | Overlay TextEditor| Full  |
| Comments      | 4   | 0   | Yes*     | Panel             | 1/2   |
| Linked Issues | 4   | 1   | Yes*     | Panel             | 1/2   |

*Comments and Linked Issues open side panels rather than inline editors.

### 2. Type Definitions

```rust
/// Represents a position in the field grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GridPosition {
    pub row: usize,
    pub col: usize,
}

impl GridPosition {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

/// Identifies which field is being referenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldId {
    Summary,
    Status,
    Priority,
    Assignee,
    Labels,
    Components,
    Description,
    Comments,
    LinkedIssues,
}

/// Type of editor used for a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorType {
    /// Single-line inline text input.
    InlineText,
    /// Multi-line text editor overlay.
    TextEditor,
    /// Modal picker dialog (status, priority, assignee).
    ModalPicker,
    /// Modal tag editor (labels, components).
    TagEditor,
    /// Side panel (comments, linked issues).
    Panel,
}

/// Metadata for a single editable field.
#[derive(Debug, Clone)]
pub struct EditableField {
    /// Unique identifier for this field.
    pub id: FieldId,
    /// Display label shown in the UI.
    pub label: &'static str,
    /// Position in the grid.
    pub position: GridPosition,
    /// Whether this field is editable (some may be read-only based on permissions).
    pub editable: bool,
    /// Type of editor to use.
    pub editor_type: EditorType,
    /// Number of columns this field spans (1, 2, or 3).
    pub col_span: usize,
}

impl EditableField {
    pub const fn new(
        id: FieldId,
        label: &'static str,
        row: usize,
        col: usize,
        editor_type: EditorType,
        col_span: usize,
    ) -> Self {
        Self {
            id,
            label,
            position: GridPosition { row, col },
            editable: true, // Can be set dynamically based on permissions
            col_span,
            editor_type,
        }
    }
}

/// The currently active editor for a field.
#[derive(Debug, Clone, PartialEq)]
pub enum ActiveEditor {
    /// No editor is active.
    None,
    /// Inline text input for summary.
    SummaryInput(String),
    /// Transition picker for status.
    StatusPicker,
    /// Priority picker.
    PriorityPicker,
    /// Assignee picker.
    AssigneePicker,
    /// Labels tag editor.
    LabelsEditor,
    /// Components tag editor.
    ComponentsEditor,
    /// Description text editor overlay.
    DescriptionEditor,
    /// Comments panel.
    CommentsPanel,
    /// Linked issues panel.
    LinkedIssuesPanel,
}

impl ActiveEditor {
    /// Check if any editor is currently active.
    pub fn is_active(&self) -> bool {
        !matches!(self, ActiveEditor::None)
    }

    /// Get the field ID for the active editor.
    pub fn field_id(&self) -> Option<FieldId> {
        match self {
            ActiveEditor::None => None,
            ActiveEditor::SummaryInput(_) => Some(FieldId::Summary),
            ActiveEditor::StatusPicker => Some(FieldId::Status),
            ActiveEditor::PriorityPicker => Some(FieldId::Priority),
            ActiveEditor::AssigneePicker => Some(FieldId::Assignee),
            ActiveEditor::LabelsEditor => Some(FieldId::Labels),
            ActiveEditor::ComponentsEditor => Some(FieldId::Components),
            ActiveEditor::DescriptionEditor => Some(FieldId::Description),
            ActiveEditor::CommentsPanel => Some(FieldId::Comments),
            ActiveEditor::LinkedIssuesPanel => Some(FieldId::LinkedIssues),
        }
    }
}

/// State for field-level editing mode.
#[derive(Debug, Clone)]
pub struct FieldEditState {
    /// Whether field edit mode is active (f key was pressed).
    pub active: bool,
    /// Currently focused field position.
    pub focused: GridPosition,
    /// Preferred column when navigating vertically (column memory).
    pub preferred_col: usize,
    /// Currently active editor (if any).
    pub editor: ActiveEditor,
}

impl Default for FieldEditState {
    fn default() -> Self {
        Self {
            active: false,
            focused: GridPosition::default(),
            preferred_col: 0,
            editor: ActiveEditor::None,
        }
    }
}

impl FieldEditState {
    /// Enter field edit mode.
    pub fn enter(&mut self) {
        self.active = true;
        self.focused = GridPosition::new(0, 0);
        self.preferred_col = 0;
        self.editor = ActiveEditor::None;
    }

    /// Exit field edit mode.
    pub fn exit(&mut self) {
        self.active = false;
        self.editor = ActiveEditor::None;
    }

    /// Check if an editor is currently open.
    pub fn has_active_editor(&self) -> bool {
        self.editor.is_active()
    }

    /// Close the active editor, returning whether one was closed.
    pub fn close_editor(&mut self) -> bool {
        if self.editor.is_active() {
            self.editor = ActiveEditor::None;
            true
        } else {
            false
        }
    }
}
```

### 3. Field Grid Definition

```rust
/// Static definition of the field grid layout.
pub struct FieldGrid {
    /// All fields in the grid, indexed by FieldId.
    fields: Vec<EditableField>,
    /// Row definitions: number of columns per row.
    row_widths: Vec<usize>,
}

impl FieldGrid {
    /// Create the standard issue field grid.
    pub fn new() -> Self {
        let fields = vec![
            EditableField::new(FieldId::Summary, "Summary", 0, 0, EditorType::InlineText, 1),
            EditableField::new(FieldId::Status, "Status", 1, 0, EditorType::ModalPicker, 1),
            EditableField::new(FieldId::Priority, "Priority", 1, 1, EditorType::ModalPicker, 1),
            EditableField::new(FieldId::Assignee, "Assignee", 1, 2, EditorType::ModalPicker, 1),
            EditableField::new(FieldId::Labels, "Labels", 2, 0, EditorType::TagEditor, 1),
            EditableField::new(FieldId::Components, "Components", 2, 1, EditorType::TagEditor, 1),
            EditableField::new(FieldId::Description, "Description", 3, 0, EditorType::TextEditor, 1),
            EditableField::new(FieldId::Comments, "Comments", 4, 0, EditorType::Panel, 1),
            EditableField::new(FieldId::LinkedIssues, "Linked Issues", 4, 1, EditorType::Panel, 1),
        ];

        Self {
            fields,
            row_widths: vec![1, 3, 2, 1, 2], // Columns per row
        }
    }

    /// Get field at a given position.
    pub fn field_at(&self, pos: GridPosition) -> Option<&EditableField> {
        self.fields.iter().find(|f| f.position == pos)
    }

    /// Get field by ID.
    pub fn field_by_id(&self, id: FieldId) -> Option<&EditableField> {
        self.fields.iter().find(|f| f.id == id)
    }

    /// Get the number of rows in the grid.
    pub fn row_count(&self) -> usize {
        self.row_widths.len()
    }

    /// Get the number of columns in a specific row.
    pub fn col_count(&self, row: usize) -> usize {
        self.row_widths.get(row).copied().unwrap_or(0)
    }

    /// Get the first editable field position.
    pub fn first_editable(&self) -> Option<GridPosition> {
        self.fields
            .iter()
            .filter(|f| f.editable)
            .map(|f| f.position)
            .min_by_key(|p| (p.row, p.col))
    }
}
```

### 4. Navigation Algorithm

The navigation algorithm handles hjkl movement with these behaviors:
- **Skip read-only fields**: Automatically skip to the next editable field
- **Column memory**: Remember preferred column when moving vertically
- **Edge handling**: Stop at grid boundaries (no wrapping)

```rust
impl FieldGrid {
    /// Navigate from current position in the given direction.
    /// Returns the new position, or None if movement is not possible.
    pub fn navigate(
        &self,
        from: GridPosition,
        direction: Direction,
        preferred_col: &mut usize,
    ) -> Option<GridPosition> {
        match direction {
            Direction::Left => self.move_left(from),
            Direction::Right => self.move_right(from),
            Direction::Up => self.move_up(from, preferred_col),
            Direction::Down => self.move_down(from, preferred_col),
        }
    }

    /// Move left within the current row.
    fn move_left(&self, from: GridPosition) -> Option<GridPosition> {
        let mut col = from.col;
        while col > 0 {
            col -= 1;
            let pos = GridPosition::new(from.row, col);
            if let Some(field) = self.field_at(pos) {
                if field.editable {
                    return Some(pos);
                }
            }
        }
        None // Cannot move further left
    }

    /// Move right within the current row.
    fn move_right(&self, from: GridPosition) -> Option<GridPosition> {
        let row_width = self.col_count(from.row);
        let mut col = from.col;
        while col + 1 < row_width {
            col += 1;
            let pos = GridPosition::new(from.row, col);
            if let Some(field) = self.field_at(pos) {
                if field.editable {
                    return Some(pos);
                }
            }
        }
        None // Cannot move further right
    }

    /// Move up to the previous row, using column memory.
    fn move_up(&self, from: GridPosition, preferred_col: &mut usize) -> Option<GridPosition> {
        // Update preferred column based on current position
        *preferred_col = from.col;

        let mut row = from.row;
        while row > 0 {
            row -= 1;
            if let Some(pos) = self.find_best_col_in_row(row, *preferred_col) {
                return Some(pos);
            }
        }
        None // Cannot move further up
    }

    /// Move down to the next row, using column memory.
    fn move_down(&self, from: GridPosition, preferred_col: &mut usize) -> Option<GridPosition> {
        // Update preferred column based on current position
        *preferred_col = from.col;

        let mut row = from.row;
        while row + 1 < self.row_count() {
            row += 1;
            if let Some(pos) = self.find_best_col_in_row(row, *preferred_col) {
                return Some(pos);
            }
        }
        None // Cannot move further down
    }

    /// Find the best column in a row given a preferred column.
    /// Prefers exact match, then nearest editable field.
    fn find_best_col_in_row(&self, row: usize, preferred_col: usize) -> Option<GridPosition> {
        let row_width = self.col_count(row);
        if row_width == 0 {
            return None;
        }

        // Clamp preferred column to row width
        let target_col = preferred_col.min(row_width - 1);

        // Try exact position first
        let pos = GridPosition::new(row, target_col);
        if let Some(field) = self.field_at(pos) {
            if field.editable {
                return Some(pos);
            }
        }

        // Search outward from target for nearest editable field
        let mut offset = 1;
        while offset < row_width {
            // Try right
            if target_col + offset < row_width {
                let pos = GridPosition::new(row, target_col + offset);
                if let Some(field) = self.field_at(pos) {
                    if field.editable {
                        return Some(pos);
                    }
                }
            }
            // Try left
            if target_col >= offset {
                let pos = GridPosition::new(row, target_col - offset);
                if let Some(field) = self.field_at(pos) {
                    if field.editable {
                        return Some(pos);
                    }
                }
            }
            offset += 1;
        }

        None // No editable field in this row
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
```

### 5. State Transition Diagram

```
                              ┌──────────────────┐
                              │   IssueDetail    │
                              │  (Normal Mode)   │
                              └────────┬─────────┘
                                       │
                                   [f] │ Press 'f' to enter field edit mode
                                       ▼
                              ┌──────────────────┐
                              │  FieldEditMode   │
          ┌──────────────────▶│  (Navigating)    │◀──────────────────┐
          │                   └────────┬─────────┘                   │
          │                            │                             │
          │ [Esc] Exit mode            │ [h/j/k/l] Navigate          │
          │ (no editor open)           │ [Enter] Open editor         │
          │                            ▼                             │
          │                   ┌──────────────────┐                   │
          │                   │  Choose Editor   │                   │
          │                   │  Based on Field  │                   │
          │                   └────────┬─────────┘                   │
          │                            │                             │
          │            ┌───────────────┼───────────────┐             │
          │            ▼               ▼               ▼             │
          │   ┌────────────────┐ ┌──────────────┐ ┌──────────────┐   │
          │   │ Inline Editor  │ │ Modal Picker │ │ Panel View   │   │
          │   │ (Summary)      │ │ (Status,     │ │ (Comments,   │   │
          │   │                │ │  Priority,   │ │  Linked)     │   │
          │   │                │ │  Assignee,   │ │              │   │
          │   │                │ │  Labels,     │ │              │   │
          │   │                │ │  Components) │ │              │   │
          │   └───────┬────────┘ └──────┬───────┘ └──────┬───────┘   │
          │           │                 │                │           │
          │           │ [Esc/Enter]     │ [Esc/Select]   │ [Esc]     │
          │           └─────────────────┴────────────────┴───────────┘
          │                             │
          │                      Close editor
          │                      Return to navigating
          └─────────────────────────────┘
```

**State Transitions:**

1. **Normal → FieldEditMode**: Press `f` key
2. **FieldEditMode → Normal**: Press `Esc` when no editor is open
3. **FieldEditMode → Editor**: Press `Enter` on a field
4. **Editor → FieldEditMode**: Press `Esc` or complete action (save/cancel)

### 6. Integration with Existing Components

**Existing Components to Reuse:**

| Component | Location | Usage |
|-----------|----------|-------|
| `TextInput` | `src/ui/components/input.rs` | Summary inline editing |
| `TextEditor` | `src/ui/components/text_editor.rs` | Description editing |
| `TransitionPicker` | `src/ui/components/transition_picker.rs` | Status changes |
| `PriorityPicker` | `src/ui/components/priority_picker.rs` | Priority changes |
| `TagEditor` | `src/ui/components/tag_editor.rs` | Labels/Components |
| `CommentsPanel` | `src/ui/components/comments.rs` | Comments |
| `LinkedIssuesSection` | `src/ui/components/linked_issues.rs` | Linked issues |

**Current Component Patterns Observed:**

1. **Action Enums**: Each component defines an `Action` enum (e.g., `PriorityAction`, `TagAction`) for returning results to parent
2. **Visibility Control**: `is_visible()`, `show()`, `hide()` pattern
3. **Loading States**: `is_loading()`, `set_loading()`, `show_loading()` pattern
4. **Input Handling**: `handle_input(KeyEvent) -> Option<Action>` pattern
5. **Rendering**: `render(&self, frame: &mut Frame, area: Rect)` method
6. **Input Mode**: Normal vs Insert mode for vim-style editing (seen in `TagEditor`)

**New Component Needed:**

- `AssigneePicker`: Similar to `PriorityPicker`, loads users from project and allows selection

**Integration Points in DetailView:**

1. Add `FieldEditState` to `IssueDetailView` struct
2. Add `FieldGrid` instance (can be static/const)
3. Modify `handle_input` to check for field edit mode
4. Add field highlight rendering when in edit mode
5. Handle editor actions and propagate to API client

### 7. Visual Design for Field Highlighting

When in field edit mode, the focused field should be visually highlighted:

```
┌─ Summary ───────────────────────────────────────────────────┐
│ Fix login bug on mobile                                     │
└─────────────────────────────────────────────────────────────┘

┌─ Status ─────────┐ ┌─ Priority ───────┐ ╔═ Assignee ════════╗
│ In Progress      │ │ High             │ ║ John Smith        ║ ← Focused
└──────────────────┘ └──────────────────┘ ╚════════════════════╝

┌─ Labels ─────────────────┐ ┌─ Components ─────────────────┐
│ bug, mobile              │ │ Authentication               │
└──────────────────────────┘ └──────────────────────────────┘
```

**Highlight Style:**
- Focused: Double-line border, bright color (Cyan or Yellow)
- Unfocused editable: Single-line border, dim color
- Read-only: No border or very dim border

### 8. Keyboard Shortcuts

| Key | Action | Context |
|-----|--------|---------|
| `f` | Enter field edit mode | Detail view (normal) |
| `h` / `←` | Move left | Field edit mode |
| `j` / `↓` | Move down | Field edit mode |
| `k` / `↑` | Move up | Field edit mode |
| `l` / `→` | Move right | Field edit mode |
| `Enter` | Open editor for field | Field edit mode |
| `Esc` | Close editor / exit mode | Field edit mode / Editor |

These do not conflict with existing bindings in detail view since:
- `e` (edit) → current full-screen edit mode (will be deprecated)
- `s` (status) → direct shortcut (can coexist)
- `c` (comment) → direct shortcut (can coexist)

### 9. Implementation Order

Based on dependencies and complexity:

1. **Task 2.1**: Create `FieldGrid` and navigation types
2. **Task 2.2**: Create `FieldEditState` and state management
3. **Task 3.1**: Integrate with DetailView, add visual highlighting
4. **Task 3.2**: Connect existing pickers (Status, Priority, Labels, etc.)
5. **Task 4**: Create AssigneePicker component
6. **Task 5**: Add inline Summary editing with TextInput
7. **Task 6**: Integrate Description editing with TextEditor overlay
8. **Task 7**: Connect Comments and LinkedIssues panels
9. **Task 8**: Update keybindings, add help text
10. **Task 9**: Testing and polish

---

## Completion Summary

**Completed:** 2025-12-01

### Files Reviewed

- `src/ui/views/detail.rs` - Current detail view with scroll-based layout, existing editor integrations
- `src/ui/components/input.rs` - TextInput for single-line input (vim Normal/Insert modes)
- `src/ui/components/text_editor.rs` - Multi-line editor with line-based editing
- `src/ui/components/transition_picker.rs` - Modal picker for status transitions
- `src/ui/components/priority_picker.rs` - Modal picker for priority selection
- `src/ui/components/tag_editor.rs` - Chip-based editor for labels/components with search
- `src/ui/components/comments.rs` - Side panel for viewing/adding comments
- `src/ui/components/linked_issues.rs` - Expandable section for linked issues/subtasks
- `src/events/keys.rs` - Keybinding definitions by context

### Key Decisions Made

1. **Grid Layout**: 5 rows with varying column counts (1, 3, 2, 1, 2)
2. **Navigation**: hjkl with column memory and read-only field skipping
3. **Editor Types**: Inline (Summary), Modal (Status/Priority/Assignee/Labels/Components), Overlay (Description), Panel (Comments/Linked)
4. **State Machine**: Three states (Normal → FieldEditMode → Editor) with Esc to go back
5. **Component Reuse**: All existing picker/editor components can be reused as-is
6. **Missing Component**: AssigneePicker needs to be created (similar pattern to PriorityPicker)

### Ready for Implementation

All acceptance criteria documented. Architecture is ready for Task 2.1 (FieldGrid types) to begin.
