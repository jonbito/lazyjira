# Feature: Edit Issue UI Redesign

## Overview

Redesign the Edit Issue UI to provide a unified, grid-based field navigation system using hjkl keys. Users can navigate between fields spatially, press Enter to edit a field (opening inline editors or modals as appropriate), and interact with a multi-column layout that mirrors the current issue detail view's visual structure.

## User Story

As a power user of LazyJira, I want to navigate between editable issue fields using hjkl keys and edit them with Enter so that I can efficiently modify issues without leaving the keyboard or memorizing many different keybindings.

## Problem Statement

The current Edit Issue UI has fragmented interaction patterns:
- Summary and description are edited via `e` key entering a full edit mode
- Status, assignee, priority use dedicated single-key shortcuts (`s`, `a`, `P`) that open pickers
- Labels and components use different shortcuts (`l`, `C`)
- There's no spatial navigation between fields
- Users must memorize many keybindings for different field types
- The edit mode only covers summary/description, not all editable fields

The proposed redesign unifies all field editing under a consistent navigation paradigm: move with hjkl, edit with Enter.

## Acceptance Criteria

- [ ] Given the user is in issue detail view, when they press a key to enter "field edit mode", then navigable fields are highlighted and hjkl navigation is enabled
- [ ] Given the user is navigating fields with hjkl, when they press `h`, then the cursor moves left to the adjacent field (if one exists)
- [ ] Given the user is navigating fields with hjkl, when they press `j`, then the cursor moves down to the field below (if one exists)
- [ ] Given the user is navigating fields with hjkl, when they press `k`, then the cursor moves up to the field above (if one exists)
- [ ] Given the user is navigating fields with hjkl, when they press `l`, then the cursor moves right to the adjacent field (if one exists)
- [ ] Given the user has a field focused, when they press `Enter`, then the appropriate editor/picker opens for that field type
- [ ] Given the user is editing summary, when they press `Enter`, then an inline single-line text input appears for editing
- [ ] Given the user is editing description, when they press `Enter`, then an inline multi-line text editor appears (or external editor option)
- [ ] Given the user is editing status, when they press `Enter`, then the transition picker modal opens
- [ ] Given the user is editing priority, when they press `Enter`, then the priority picker modal opens
- [ ] Given the user is editing assignee, when they press `Enter`, then the assignee picker modal opens
- [ ] Given the user is editing labels, when they press `Enter`, then the label tag editor opens
- [ ] Given the user is editing links, when they press `Enter`, then the link management interface opens
- [ ] Given the user is editing comments, when they press `Enter`, then the comments panel opens for viewing/adding comments
- [ ] Given the user presses `Esc` while navigating fields, then field edit mode is exited and normal view mode resumes
- [ ] Given editable fields in a multi-column layout, when rendered, then fields are arranged logically (e.g., Summary full-width, Assignee/Status/Type in a row)

## Technical Requirements

### System Areas Affected

- [x] Frontend (UI components and views)
- [ ] Backend
- [x] CLI (keyboard handling)
- [ ] Database
- [ ] Infrastructure

### Implementation Approach

1. **Create a FieldGrid abstraction** that defines the spatial layout of editable fields
2. **Introduce FieldEditMode** as a new sub-state within the detail view
3. **Define field types** with associated edit behaviors (inline text, modal picker, panel)
4. **Implement navigation logic** that respects the multi-column grid structure
5. **Unify field editing** by routing all edits through the field grid system
6. **Update rendering** to show focus indicators and field boundaries clearly

### Key Components

#### New Components

- **`FieldGrid`**: Manages the 2D grid layout of fields and cursor position
  - Tracks current row/column position
  - Provides navigation methods (move_left, move_right, move_up, move_down)
  - Returns the currently focused `EditableField`

- **`EditableField`**: Enum representing each editable field with its edit behavior
  ```rust
  enum EditableField {
      Summary,       // Inline TextInput
      Description,   // Inline TextEditor or external editor
      Status,        // Modal: TransitionPicker
      Priority,      // Modal: PriorityPicker
      Assignee,      // Modal: AssigneePicker
      Labels,        // Panel: TagEditor
      Links,         // Panel: LinkedIssues with add/remove
      Comments,      // Panel: CommentsPanel
  }
  ```

- **`FieldEditState`**: Tracks the current field navigation state
  ```rust
  struct FieldEditState {
      grid: FieldGrid,
      active_editor: Option<ActiveEditor>,
  }

  enum ActiveEditor {
      Summary(TextInput),
      Description(TextEditor),
      // Modal/panel editors are handled separately
  }
  ```

#### Modified Components

- **`DetailView`** (`src/ui/views/detail.rs`):
  - Add `field_edit_state: Option<FieldEditState>`
  - Modify `handle_input()` to process hjkl navigation when in field edit mode
  - Update `render()` to show field focus indicators and grid layout

- **`DetailAction`** (`src/ui/views/detail.rs`):
  - Add `EnterFieldEditMode`
  - Add `ExitFieldEditMode`
  - Add `NavigateField(Direction)` where Direction is Left/Right/Up/Down
  - Add `ActivateField` (triggered by Enter)

### Grid Layout Design

Based on the screenshot, the proposed grid layout:

```
Row 0: [Summary                                              ]  (1 column, full width)
Row 1: [Assignee      ] [Status        ] [Type (read-only)   ]  (3 columns)
Row 2: [Key (r/o)     ] [Parent (r/o)  ] [Sprint (r/o)       ]  (3 columns, read-only)
Row 3: [Project (r/o)                                        ]  (1 column, read-only)
Row 4: [Priority      ] [Reporter (r/o)]                        (2 columns)
Row 5: [Created (r/o) ] [Last Update   ] [Due Date           ]  (3 columns, 2 read-only)
Row 6: [Resolved (r/o)] [Resolution(r/o)]                       (2 columns, read-only)
Row 7: [Labels                                               ]  (1 column, full width)
Row 8: [Links                                                ]  (1 column, full width)
Row 9: [Description                                          ]  (1 column, full width)
Row 10:[Comments                                             ]  (1 column, full width)
```

**Editable fields (8 total):**
- Row 0: Summary
- Row 1: Assignee, Status
- Row 4: Priority
- Row 5: Due Date (if editing dates is supported)
- Row 7: Labels
- Row 8: Links
- Row 9: Description
- Row 10: Comments

**Navigation rules:**
- Skip read-only fields during navigation
- Wrap within rows or stop at edges (configurable)
- Remember column position when moving between rows of different widths

### Data Requirements

No new data models needed. Uses existing:
- `Issue` and `IssueFields` from `src/api/types.rs`
- `IssueUpdateRequest` for saving changes
- Existing picker/editor components for field editing

### Rendering Updates

1. **Field Focus Indicator**: Highlighted border (yellow/gold) around the currently focused field
2. **Editable vs Read-only**: Visual distinction (editable fields have colored borders, read-only are dimmed)
3. **Mode Indicator**: Status line shows "FIELD EDIT" or similar when in field edit mode
4. **Keyboard Hints**: Footer shows "hjkl: navigate | Enter: edit | Esc: exit"

## Dependencies

### Internal Dependencies

- `TextInput` component (for summary editing)
- `TextEditor` component (for description editing)
- `TransitionPicker` component (for status changes)
- `PriorityPicker` component (for priority changes)
- `AssigneePicker` component (for assignee changes)
- `TagEditor` component (for labels)
- `LinkedIssues` and link management components (for links)
- `CommentsPanel` component (for comments)

### External Dependencies

- `ratatui` for layout and rendering
- `crossterm` for keyboard input handling

## Success Criteria

### Definition of Done

- [ ] All acceptance criteria met
- [ ] Field grid navigation works correctly with hjkl
- [ ] All 8 editable fields are accessible and editable
- [ ] Visual focus indicators are clear and consistent
- [ ] Existing picker/editor components integrate seamlessly
- [ ] Tests written and passing
- [ ] Help panel updated with new keybindings
- [ ] Code reviewed and merged

### Success Metrics

- Navigation between any two editable fields requires at most O(n) key presses where n is the grid distance
- No regression in existing edit functionality
- Consistent editing experience across all field types

## Risk Assessment

### Technical Risks

- **Complex navigation edge cases**: Fields of varying widths and read-only gaps make grid navigation non-trivial
  - *Mitigation*: Define clear navigation rules (skip read-only, remember column position)

- **State management complexity**: Adding another sub-state (field edit mode) increases complexity
  - *Mitigation*: Keep field edit state orthogonal to existing edit state, clear state transitions

- **Integration with existing pickers**: Pickers are designed to be triggered by single keys, not Enter
  - *Mitigation*: Pickers already have show/hide methods; just trigger show() on Enter

### Timeline Risks

- **Scope creep**: Adding more editable fields or fancy features
  - *Mitigation*: Focus on the 8 specified fields only; additional fields can be added later

- **Rendering performance**: Complex grid layout calculations on each frame
  - *Mitigation*: Cache layout calculations, only recalculate on resize

## Technical Notes

### Key Implementation Details

1. **Mode Entry/Exit**:
   - Enter field edit mode with `f` key (or similar) from detail view
   - Exit with `Esc` (returns to normal detail view)
   - If a field editor is active, `Esc` first closes the editor, second `Esc` exits field edit mode

2. **Grid Implementation**:
   ```rust
   struct FieldGrid {
       rows: Vec<Vec<EditableField>>,  // 2D grid of fields
       row: usize,
       col: usize,
   }

   impl FieldGrid {
       fn current_field(&self) -> &EditableField;
       fn move_left(&mut self) -> bool;   // Returns false if at edge
       fn move_right(&mut self) -> bool;
       fn move_up(&mut self) -> bool;
       fn move_down(&mut self) -> bool;
   }
   ```

3. **Field Edit Behavior Mapping**:
   - **Inline editors** (Summary, Description): Replace field content with editor widget
   - **Modal pickers** (Status, Priority, Assignee): Open centered modal, return to grid on close
   - **Panel editors** (Labels, Links, Comments): Open side/bottom panel, return to grid on close

4. **Keybinding Summary**:
   | Key | Action |
   |-----|--------|
   | `f` | Enter field edit mode |
   | `h` | Move left |
   | `j` | Move down |
   | `k` | Move up |
   | `l` | Move right |
   | `Enter` | Edit focused field |
   | `Esc` | Close editor / Exit field edit mode |

5. **Visual Design**:
   - Focused field: Yellow/gold border with brighter text
   - Editable fields: Cyan/blue border labels
   - Read-only fields: Dim gray text, no special border
   - Current field label shows "(editing)" when editor is active

### Migration Path

The existing edit mode (via `e` key) can coexist during transition:
- `e` enters the legacy summary/description edit mode (deprecated)
- `f` enters the new field edit mode
- Eventually, `e` can be remapped or removed

### Future Enhancements (Out of Scope)

- Editing Sprint, Due Date, Time Tracking fields
- Bulk field editing across multiple issues
- Field editing from the issue list view
- Custom field support

---

## Task Breakdown

This feature has been broken down into the following implementation tasks:

### 1. Research & Architecture

- [ ] [Task 1] Research: Field Grid System Architecture - [`tasks/[Task 1] Research - Field Grid System Architecture.md`](tasks/[Task%201]%20Research%20-%20Field%20Grid%20System%20Architecture.md)

### 2. Core Implementation

- [ ] [Task 2.1] Core: FieldGrid and EditableField Types - [`tasks/[Task 2.1] Core - FieldGrid and EditableField Types.md`](tasks/[Task%202.1]%20Core%20-%20FieldGrid%20and%20EditableField%20Types.md)
- [ ] [Task 2.2] Core: FieldEditState and Navigation Logic - [`tasks/[Task 2.2] Core - FieldEditState and Navigation Logic.md`](tasks/[Task%202.2]%20Core%20-%20FieldEditState%20and%20Navigation%20Logic.md)

### 3. UI Implementation

- [ ] [Task 3.1] UI: DetailView Field Edit Mode Integration - [`tasks/[Task 3.1] UI - DetailView Field Edit Mode Integration.md`](tasks/[Task%203.1]%20UI%20-%20DetailView%20Field%20Edit%20Mode%20Integration.md)
- [ ] [Task 3.2] UI: Field Focus Rendering and Visual Indicators - [`tasks/[Task 3.2] UI - Field Focus Rendering and Visual Indicators.md`](tasks/[Task%203.2]%20UI%20-%20Field%20Focus%20Rendering%20and%20Visual%20Indicators.md)

### 4. Editor Integration

- [ ] [Task 4.1] Integration: Inline Editors (Summary, Description) - [`tasks/[Task 4.1] Integration - Inline Editors (Summary, Description).md`](tasks/[Task%204.1]%20Integration%20-%20Inline%20Editors%20(Summary,%20Description).md)
- [ ] [Task 4.2] Integration: Modal Pickers (Status, Priority, Assignee) - [`tasks/[Task 4.2] Integration - Modal Pickers (Status, Priority, Assignee).md`](tasks/[Task%204.2]%20Integration%20-%20Modal%20Pickers%20(Status,%20Priority,%20Assignee).md)
- [ ] [Task 4.3] Integration: Panel Editors (Labels, Links, Comments) - [`tasks/[Task 4.3] Integration - Panel Editors (Labels, Links, Comments).md`](tasks/[Task%204.3]%20Integration%20-%20Panel%20Editors%20(Labels,%20Links,%20Comments).md)

### 5. Testing & Quality

- [ ] [Task 5] Testing: Field Grid Navigation and Integration - [`tasks/[Task 5] Testing - Field Grid Navigation and Integration.md`](tasks/[Task%205]%20Testing%20-%20Field%20Grid%20Navigation%20and%20Integration.md)

### 6. Documentation

- [ ] [Task 6] Documentation: Help Panel and Keybindings - [`tasks/[Task 6] Documentation - Help Panel and Keybindings.md`](tasks/[Task%206]%20Documentation%20-%20Help%20Panel%20and%20Keybindings.md)

**Total Tasks:** 10
**Estimated Effort:** M-L (approximately 30-45 hours total)

**Critical Path:** Task 1 → Task 2.1 → Task 2.2 → Task 3.1 → Task 4.1/4.2/4.3 (parallel) → Task 5

**Implementation Order:**
1. Task 1 (Research) - Foundational architecture design
2. Task 2.1, 2.2 (Core) - Core types and navigation logic
3. Task 3.1 (UI Integration) - DetailView field edit mode
4. Task 3.2 (UI Rendering) - Can run parallel with 4.x
5. Task 4.1, 4.2, 4.3 (Editor Integration) - Can run in parallel
6. Task 5 (Testing) - After core implementation complete
7. Task 6 (Documentation) - After feature stabilizes
