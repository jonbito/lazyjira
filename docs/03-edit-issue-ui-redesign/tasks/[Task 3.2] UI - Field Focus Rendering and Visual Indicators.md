# Task 3.2: Field Focus Rendering and Visual Indicators

**Documentation:** [Feature: Edit Issue UI Redesign](../[Feature]%20Edit%20Issue%20UI%20Redesign.md)
**Task Number:** 3.2
**Area:** Frontend/UI
**Estimated Effort:** M (4-6 hours)

## Description

Implement the visual rendering for field edit mode, including focus indicators, field boundaries, editable vs read-only styling, mode indicator in status line, and keyboard hint footer.

## Acceptance Criteria

- [ ] Focused field has yellow/gold highlighted border
- [ ] Editable fields have colored (cyan/blue) border labels
- [ ] Read-only fields are displayed with dimmed gray text
- [ ] Status line shows "FIELD EDIT" indicator when in mode
- [ ] Footer shows "hjkl: navigate | Enter: edit | Esc: exit" when in mode
- [ ] Current field label shows "(editing)" when inline editor is active
- [ ] Visual distinction is clear and consistent
- [ ] Rendering performs well (no flicker or lag)

## Implementation Details

### Approach

1. **Define field rendering styles**:
   ```rust
   // In src/ui/field_grid.rs or a new styles module
   pub struct FieldStyles {
       pub focused_border: Style,
       pub editable_label: Style,
       pub readonly_label: Style,
       pub readonly_value: Style,
       pub editing_label: Style,
   }

   impl Default for FieldStyles {
       fn default() -> Self {
           Self {
               focused_border: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
               editable_label: Style::default().fg(Color::Cyan),
               readonly_label: Style::default().fg(Color::DarkGray),
               readonly_value: Style::default().fg(Color::DarkGray),
               editing_label: Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
           }
       }
   }
   ```

2. **Update DetailView render() method**:
   ```rust
   fn render(&self, frame: &mut Frame, area: Rect) {
       // ... existing render logic

       // Determine if in field edit mode
       let field_edit_active = self.field_edit_state.is_some();
       let focused_field = self.current_field();

       // Render each field with appropriate styling
       for (field, field_area) in self.field_layout() {
           let style = if Some(field) == focused_field {
               self.styles.focused_border
           } else if field.is_editable() {
               self.styles.editable_label
           } else {
               self.styles.readonly_label
           };

           // Apply style to field rendering
           self.render_field(frame, field, field_area, style);
       }

       // Render mode indicator and footer
       if field_edit_active {
           self.render_field_edit_footer(frame, footer_area);
       }
   }
   ```

3. **Create field layout helper**:
   ```rust
   fn field_layout(&self) -> Vec<(EditableField, Rect)> {
       // Map each editable field to its screen position
       // This may need to integrate with existing detail view layout
   }
   ```

4. **Render mode indicator in status line**:
   ```rust
   fn render_status_line(&self, frame: &mut Frame, area: Rect) {
       let mode_text = if self.is_in_field_edit_mode() {
           if self.field_edit_state.as_ref().map(|s| s.is_editing()).unwrap_or(false) {
               " FIELD EDIT (editing) "
           } else {
               " FIELD EDIT "
           }
       } else {
           " DETAIL "
       };

       let mode_span = Span::styled(mode_text, Style::default().bg(Color::Blue).fg(Color::White));
       // ... render status line
   }
   ```

5. **Render keyboard hints footer**:
   ```rust
   fn render_field_edit_footer(&self, frame: &mut Frame, area: Rect) {
       let hints = if self.field_edit_state.as_ref().map(|s| s.is_editing()).unwrap_or(false) {
           "Esc: close editor"
       } else {
           "h/j/k/l: navigate | Enter: edit | Esc: exit"
       };

       let footer = Paragraph::new(hints)
           .style(Style::default().fg(Color::DarkGray));
       frame.render_widget(footer, area);
   }
   ```

6. **Highlight focused field with border**:
   ```rust
   fn render_field_with_focus(&self, frame: &mut Frame, field: EditableField, area: Rect, focused: bool) {
       let block = Block::default()
           .borders(Borders::ALL)
           .border_style(if focused {
               Style::default().fg(Color::Yellow)
           } else {
               Style::default().fg(Color::DarkGray)
           });

       let inner = block.inner(area);
       frame.render_widget(block, area);
       // Render field content in inner area
   }
   ```

### Files to Modify/Create

- `src/ui/views/detail.rs`: Update render() method with field edit mode rendering
- `src/ui/field_grid.rs`: Add FieldStyles struct
- Potentially `src/ui/theme.rs` or similar: If theme system exists, integrate styles

### Technical Specifications

- Use ratatui `Style` for all styling
- Colors: Yellow for focus, Cyan for editable, DarkGray for read-only, Green for editing
- Border style uses `Borders::ALL` with colored border
- Mode indicator should be prominent but not obtrusive
- Footer keyboard hints should be subtle (DarkGray text)

## Testing Requirements

- [ ] Visual test: Focused field is clearly distinguishable
- [ ] Visual test: Editable vs read-only fields are visually distinct
- [ ] Visual test: Mode indicator appears in status line
- [ ] Visual test: Footer hints update based on editor state
- [ ] Performance test: No noticeable rendering lag or flicker
- [ ] Test: Styles are consistent across all field types

## Dependencies

- **Prerequisite Tasks:** Task 3.1 (DetailView integration with field edit state)
- **Blocks Tasks:** None directly (can be done in parallel with Task 4.x)
- **External:** ratatui styling primitives

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Visual styling matches feature spec
- [ ] No rendering performance issues
- [ ] Styles integrate with existing theme (if applicable)
- [ ] Code follows project standards
- [ ] Code reviewed and merged
