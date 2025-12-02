//! User interface components and views.
//!
//! This module contains all TUI rendering logic, including views for different
//! screens and reusable UI components.

mod components;
pub mod theme;
mod views;

pub use components::{
    render_context_help, CommandPalette, CommandPaletteAction, ConfirmDialog, ErrorDialog,
    ExternalEditor, InlineLoader, JqlAction, JqlInput, LoadingIndicator, Modal, MultiSelect,
    Notification, NotificationManager, NotificationType, ProfilePicker, ProfilePickerAction,
    SavedFiltersAction, SavedFiltersDialog, SelectItem, SpinnerStyle, Table, TextEditor, TextInput,
    TransitionAction, TransitionPicker,
};
pub use theme::{
    init_theme, issue_type_prefix, load_theme, parse_color, priority_style, status_style, theme,
    truncate, try_theme, CustomThemeConfig, Theme,
};
pub use views::{
    DeleteProfileDialog, DetailAction, DetailView, EditField, EditState, FilterPanelAction,
    FilterPanelView, FormField, FormMode, HelpAction, HelpView, ListAction, ListView,
    ProfileFormAction, ProfileFormData, ProfileFormView, ProfileListAction, ProfileListView,
    ProfileSummary,
};
