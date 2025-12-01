# Task 6: Update Help Panel and Keybindings

**Documentation:** [Feature: Edit Issue UI Redesign](../[Feature]%20Edit%20Issue%20UI%20Redesign.md)
**Task Number:** 6
**Area:** Documentation
**Estimated Effort:** S (2-4 hours)

## Description

Update the help panel and keybinding documentation to reflect the new field edit mode. This includes adding the new keybindings to the help screen, updating any existing documentation, and ensuring keyboard hints are consistent throughout the UI.

## Acceptance Criteria

- [ ] Help panel includes field edit mode section
- [ ] New keybindings documented: `f` (enter mode), `hjkl` (navigate), `Enter` (edit), `Esc` (exit)
- [ ] Help panel accessible via existing help key (typically `?`)
- [ ] Keybindings in `src/events/keys.rs` updated if needed
- [ ] Any existing keybinding docs updated to reflect changes
- [ ] Footer hints consistent with help panel content
- [ ] No conflicts between new and existing keybindings

## Implementation Details

### Approach

1. **Update help panel content**:
   ```rust
   // In help view or help generation
   fn field_edit_mode_help() -> Vec<(&'static str, &'static str)> {
       vec![
           ("Field Edit Mode", ""),
           ("f", "Enter field edit mode"),
           ("h", "Move to left field"),
           ("j", "Move to field below"),
           ("k", "Move to field above"),
           ("l", "Move to right field"),
           ("Enter", "Edit focused field"),
           ("Esc", "Exit field edit mode / Close editor"),
       ]
   }
   ```

2. **Update keybinding definitions** in `src/events/keys.rs`:
   ```rust
   // Add or verify these key definitions
   pub const KEY_FIELD_EDIT_MODE: char = 'f';
   pub const KEY_NAV_LEFT: char = 'h';
   pub const KEY_NAV_DOWN: char = 'j';
   pub const KEY_NAV_UP: char = 'k';
   pub const KEY_NAV_RIGHT: char = 'l';
   ```

3. **Verify no keybinding conflicts**:
   - `f` in detail view: Check not used for other function
   - `hjkl` in field edit mode: These override any other meaning
   - `Enter` and `Esc` are standard and should be fine

4. **Update help view rendering**:
   ```rust
   fn render_help(&self, frame: &mut Frame, area: Rect) {
       let sections = vec![
           ("Global", global_keybindings()),
           ("Issue List", list_keybindings()),
           ("Issue Detail", detail_keybindings()),
           ("Field Edit Mode", field_edit_mode_help()), // New section
           // ... other sections
       ];

       // Render help sections
   }
   ```

5. **Update existing detail view help section**:
   ```rust
   fn detail_keybindings() -> Vec<(&'static str, &'static str)> {
       vec![
           ("Esc/q", "Return to list"),
           ("f", "Enter field edit mode"), // Add this
           ("e", "Edit summary/description (legacy)"), // Mark as legacy
           ("s", "Change status"),
           ("a", "Change assignee"),
           ("P", "Change priority"),
           // ...
       ]
   }
   ```

6. **Ensure footer hints match**:
   - Normal detail view: Include "f: edit fields" hint
   - Field edit mode: "hjkl: navigate | Enter: edit | Esc: exit"
   - Active editor: "Esc: close" or "Enter: save | Esc: cancel"

### Files to Modify/Create

- `src/ui/views/help.rs`: Add field edit mode section, update detail section
- `src/events/keys.rs`: Add/verify key constants
- `src/ui/views/detail.rs`: Update footer hints for different modes

### Keybinding Reference

| Key | Context | Action |
|-----|---------|--------|
| `f` | Detail View | Enter field edit mode |
| `h` | Field Edit Mode | Move cursor left |
| `j` | Field Edit Mode | Move cursor down |
| `k` | Field Edit Mode | Move cursor up |
| `l` | Field Edit Mode | Move cursor right |
| `Enter` | Field Edit Mode | Edit focused field |
| `Esc` | Field Edit Mode | Exit mode / Close editor |
| `?` | Any | Open help panel |

### Migration Notes

Document the relationship between new and existing keybindings:

```
# Edit Mode Migration

## Current (Legacy)
- `e` in detail view: Enter summary/description edit mode

## New (Field Edit Mode)
- `f` in detail view: Enter field edit mode
- Navigate to Summary/Description, press Enter to edit

## Compatibility
Both modes coexist during transition. The `e` key continues
to work for quick summary/description edits. Consider
deprecating `e` mode in a future release.
```

## Testing Requirements

- [ ] Verify `?` opens help panel
- [ ] Verify new keybindings appear in help
- [ ] Verify footer hints update correctly per mode
- [ ] Verify no keybinding conflicts
- [ ] Verify help sections render without overflow

## Dependencies

- **Prerequisite Tasks:** Task 3.1 (to know final keybindings)
- **Blocks Tasks:** None
- **External:** None

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Help panel includes field edit mode section
- [ ] Keybindings documented and consistent
- [ ] Footer hints match help panel
- [ ] No conflicts with existing bindings
- [ ] Code follows project standards
- [ ] Code reviewed and merged
