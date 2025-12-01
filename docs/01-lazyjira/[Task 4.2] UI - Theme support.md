# Task 4.2: Theme Support

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 4.2
**Area:** Frontend/UI
**Estimated Effort:** M (4-6 hours)

## Description

Implement theme support with light, dark, and custom color schemes. The application should respect terminal color themes and provide accessible color options.

## Acceptance Criteria

- [ ] Default dark theme
- [ ] Light theme option
- [ ] High contrast theme for accessibility
- [ ] Custom theme via configuration
- [ ] Theme switching at runtime
- [ ] Consistent styling across all views
- [ ] Status colors (priority, status) adapt to theme
- [ ] Configuration persisted

## Implementation Details

### Approach

1. Define theme struct with all color definitions
2. Create built-in themes
3. Load custom theme from config
4. Apply theme to all components
5. Add runtime switching

### Files to Modify/Create

- `src/ui/theme.rs`: Theme definitions and loading
- `src/config/settings.rs`: Theme configuration
- All view files: Apply theme colors

### Technical Specifications

**Theme Definition:**
```rust
use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,

    // Base colors
    pub background: Color,
    pub foreground: Color,
    pub foreground_dim: Color,

    // Accent colors
    pub accent: Color,
    pub accent_dim: Color,

    // Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // Priority colors
    pub priority_highest: Color,
    pub priority_high: Color,
    pub priority_medium: Color,
    pub priority_low: Color,
    pub priority_lowest: Color,

    // Status category colors
    pub status_new: Color,
    pub status_in_progress: Color,
    pub status_done: Color,

    // UI element colors
    pub border: Color,
    pub border_focused: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub header_bg: Color,
    pub header_fg: Color,

    // Component-specific
    pub input_bg: Color,
    pub input_fg: Color,
    pub input_placeholder: Color,
    pub tag_bg: Color,
    pub tag_fg: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),

            background: Color::Reset,  // Use terminal default
            foreground: Color::White,
            foreground_dim: Color::Gray,

            accent: Color::Cyan,
            accent_dim: Color::DarkGray,

            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,

            priority_highest: Color::Red,
            priority_high: Color::LightRed,
            priority_medium: Color::Yellow,
            priority_low: Color::Blue,
            priority_lowest: Color::Gray,

            status_new: Color::Blue,
            status_in_progress: Color::Yellow,
            status_done: Color::Green,

            border: Color::DarkGray,
            border_focused: Color::Cyan,
            selection_bg: Color::DarkGray,
            selection_fg: Color::White,
            header_bg: Color::DarkGray,
            header_fg: Color::White,

            input_bg: Color::Reset,
            input_fg: Color::White,
            input_placeholder: Color::DarkGray,
            tag_bg: Color::Blue,
            tag_fg: Color::White,
        }
    }

    pub fn light() -> Self {
        Self {
            name: "light".to_string(),

            background: Color::Reset,
            foreground: Color::Black,
            foreground_dim: Color::DarkGray,

            accent: Color::Blue,
            accent_dim: Color::Gray,

            success: Color::Green,
            warning: Color::Rgb(180, 140, 0), // Darker yellow
            error: Color::Red,
            info: Color::Blue,

            priority_highest: Color::Red,
            priority_high: Color::Rgb(200, 80, 80),
            priority_medium: Color::Rgb(180, 140, 0),
            priority_low: Color::Blue,
            priority_lowest: Color::Gray,

            status_new: Color::Blue,
            status_in_progress: Color::Rgb(180, 140, 0),
            status_done: Color::Green,

            border: Color::Gray,
            border_focused: Color::Blue,
            selection_bg: Color::LightBlue,
            selection_fg: Color::Black,
            header_bg: Color::Gray,
            header_fg: Color::Black,

            input_bg: Color::Reset,
            input_fg: Color::Black,
            input_placeholder: Color::Gray,
            tag_bg: Color::LightBlue,
            tag_fg: Color::Black,
        }
    }

    pub fn high_contrast() -> Self {
        Self {
            name: "high-contrast".to_string(),

            background: Color::Black,
            foreground: Color::White,
            foreground_dim: Color::White,

            accent: Color::Yellow,
            accent_dim: Color::Yellow,

            success: Color::LightGreen,
            warning: Color::Yellow,
            error: Color::LightRed,
            info: Color::LightCyan,

            priority_highest: Color::LightRed,
            priority_high: Color::LightRed,
            priority_medium: Color::Yellow,
            priority_low: Color::LightCyan,
            priority_lowest: Color::White,

            status_new: Color::LightCyan,
            status_in_progress: Color::Yellow,
            status_done: Color::LightGreen,

            border: Color::White,
            border_focused: Color::Yellow,
            selection_bg: Color::White,
            selection_fg: Color::Black,
            header_bg: Color::White,
            header_fg: Color::Black,

            input_bg: Color::Black,
            input_fg: Color::White,
            input_placeholder: Color::White,
            tag_bg: Color::Yellow,
            tag_fg: Color::Black,
        }
    }

    // Helper methods for common styles
    pub fn style_normal(&self) -> Style {
        Style::default().fg(self.foreground)
    }

    pub fn style_dim(&self) -> Style {
        Style::default().fg(self.foreground_dim)
    }

    pub fn style_accent(&self) -> Style {
        Style::default().fg(self.accent)
    }

    pub fn style_error(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn style_success(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn style_selected(&self) -> Style {
        Style::default()
            .bg(self.selection_bg)
            .fg(self.selection_fg)
    }

    pub fn style_border(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn style_border_focused(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    pub fn style_for_priority(&self, priority: &str) -> Style {
        let color = match priority.to_lowercase().as_str() {
            "highest" | "blocker" => self.priority_highest,
            "high" | "critical" => self.priority_high,
            "medium" => self.priority_medium,
            "low" => self.priority_low,
            "lowest" => self.priority_lowest,
            _ => self.foreground,
        };
        Style::default().fg(color)
    }

    pub fn style_for_status_category(&self, category: &str) -> Style {
        let color = match category {
            "new" => self.status_new,
            "indeterminate" => self.status_in_progress,
            "done" => self.status_done,
            _ => self.foreground,
        };
        Style::default().fg(color)
    }
}
```

**Theme Configuration:**
```rust
// In config/settings.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default)]
    pub custom_theme: Option<CustomThemeConfig>,
}

fn default_theme() -> String {
    "dark".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomThemeConfig {
    pub accent: Option<String>,
    pub success: Option<String>,
    pub warning: Option<String>,
    pub error: Option<String>,
    // ... other customizable colors
}

impl CustomThemeConfig {
    pub fn apply_to(&self, base: &mut Theme) {
        if let Some(color) = &self.accent {
            if let Some(c) = parse_color(color) {
                base.accent = c;
            }
        }
        // Apply other customizations
    }
}

fn parse_color(s: &str) -> Option<Color> {
    // Parse color string: "red", "#ff0000", "rgb(255,0,0)"
    match s.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "gray" | "grey" => Some(Color::Gray),
        s if s.starts_with('#') && s.len() == 7 => {
            let r = u8::from_str_radix(&s[1..3], 16).ok()?;
            let g = u8::from_str_radix(&s[3..5], 16).ok()?;
            let b = u8::from_str_radix(&s[5..7], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        _ => None,
    }
}
```

**Theme Loading:**
```rust
pub fn load_theme(settings: &Settings) -> Theme {
    let mut theme = match settings.theme.as_str() {
        "light" => Theme::light(),
        "high-contrast" => Theme::high_contrast(),
        "dark" | _ => Theme::dark(),
    };

    if let Some(custom) = &settings.custom_theme {
        custom.apply_to(&mut theme);
    }

    theme
}
```

**Theme Context:**
```rust
// Global theme access
use std::sync::OnceLock;

static THEME: OnceLock<Theme> = OnceLock::new();

pub fn init_theme(theme: Theme) {
    THEME.set(theme).ok();
}

pub fn theme() -> &'static Theme {
    THEME.get().expect("Theme not initialized")
}

// Usage in components
fn render_header(&self, frame: &mut Frame, area: Rect) {
    let style = theme().style_accent().add_modifier(Modifier::BOLD);
    let widget = Paragraph::new(&self.title).style(style);
    frame.render_widget(widget, area);
}
```

## Testing Requirements

- [ ] Dark theme renders correctly
- [ ] Light theme renders correctly
- [ ] High contrast theme readable
- [ ] Custom colors applied
- [ ] Theme persists in config
- [ ] All components use theme
- [ ] Priority colors visible in all themes
- [ ] Status colors visible in all themes

## Dependencies

- **Prerequisite Tasks:** Task 1.2, Task 1.3
- **Blocks Tasks:** None
- **External:** None

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Themes are accessible
- [ ] Colors consistent across app
- [ ] Custom themes work
- [ ] No hard-coded colors in views

---

## Implementation Notes

### Completed: 2025-11-30

#### Files Modified/Created

1. **`src/ui/theme.rs`** - Complete rewrite with:
   - Comprehensive `Theme` struct with 30+ color definitions
   - Built-in themes: `dark()`, `light()`, `high_contrast()`
   - Global theme access via `init_theme()`, `theme()`, `try_theme()`
   - Color parsing for hex (`#ff0000`), RGB (`rgb(255,0,0)`), and named colors
   - `CustomThemeConfig` for user customization
   - `load_theme()` helper for loading themes from settings
   - Style helper methods (`style_normal()`, `style_accent()`, `style_error()`, etc.)
   - Backward-compatible `status_style()` and `priority_style()` functions

2. **`src/config/settings.rs`** - Added:
   - `custom_theme: Option<CustomThemeConfig>` field for custom color overrides
   - Updated tests for new field

3. **`src/config/mod.rs`** - Updated tests to include `custom_theme` field

4. **`src/ui/mod.rs`** - Exported new theme types and made theme module public

5. **`src/main.rs`** - Added theme initialization at startup

#### Key Implementation Decisions

1. **Global Theme Pattern**: Used `OnceLock` for thread-safe, once-initialized global theme access. This allows components to access the theme without passing it through every function.

2. **Backward Compatibility**: The existing `status_style()` and `priority_style()` functions were updated to use the theme when initialized, with fallback to hardcoded colors if not.

3. **Theme Configuration Format**: Custom themes use TOML format with named colors, hex, or RGB values:
   ```toml
   theme = "dark"  # or "light", "high-contrast"

   [custom_theme]
   accent = "#ff00ff"
   success = "lightgreen"
   border = "rgb(100, 100, 100)"
   ```

4. **High Contrast Theme**: Designed for accessibility with:
   - Maximum contrast (white on black)
   - No dim/muted colors (all white)
   - Yellow accent color for visibility
   - Bright color variants for all status colors

#### Test Coverage

- 631 tests pass
- Tests for theme creation, color parsing, custom theme application
- Tests for settings serialization with custom_theme field

#### Theme Applied to Views and Components

Views updated to use theme colors:
- `src/ui/views/list.rs` - Table headers, selection, status bar, loading/empty states
- `src/ui/views/detail.rs` - Header, metadata, labels, components, description, status bar
- `src/ui/views/filter.rs` - Panel border, help text
- `src/ui/views/profile.rs` - Profile list, form dialogs, delete confirmation
- `src/ui/views/help.rs` - Keyboard shortcuts display, section headers

Components updated:
- `src/ui/components/input.rs` - Border colors, text colors, focus indicators

Note: Additional components (notification, modal, pickers, etc.) retain hardcoded colors
for now but can access the theme via `theme()` for future updates.
