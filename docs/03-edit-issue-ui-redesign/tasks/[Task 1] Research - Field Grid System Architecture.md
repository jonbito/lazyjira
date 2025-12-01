# Task 1: Field Grid System Architecture

**Documentation:** [Feature: Edit Issue UI Redesign](../[Feature]%20Edit%20Issue%20UI%20Redesign.md)
**Task Number:** 1
**Area:** Research/Architecture
**Estimated Effort:** S (2-4 hours)

## Description

Research and design the field grid system architecture that will enable spatial navigation between editable fields in the issue detail view. This task establishes the foundational design patterns and data structures before implementation begins.

## Acceptance Criteria

- [ ] Document the field grid layout structure (rows, columns, field positions)
- [ ] Define the navigation algorithm for hjkl movement (including read-only field skipping)
- [ ] Identify integration points with existing DetailView and editor components
- [ ] Create type definitions for FieldGrid, EditableField, and FieldEditState
- [ ] Document state transition diagram for field edit mode
- [ ] Review existing picker/editor components for compatibility

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

- [ ] Architecture document completed
- [ ] Navigation algorithm defined and edge cases documented
- [ ] Type definitions reviewed and approved
- [ ] Integration points identified for all 8 editable fields
- [ ] State transition diagram completed
- [ ] Ready for implementation to begin

## Notes

Key design decisions to document:

1. **Grid representation**: Should use a sparse representation since rows have varying column counts
2. **Field skipping**: Navigation should skip read-only fields automatically
3. **Column memory**: When moving vertically between rows of different widths, remember the original column preference
4. **Mode entry**: Use `f` key to enter field edit mode (not conflicting with existing bindings)
5. **Esc behavior**: First Esc closes active editor, second Esc exits field edit mode
