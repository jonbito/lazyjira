//! User interface components and views.
//!
//! This module contains all TUI rendering logic, including views for different
//! screens and reusable UI components.

mod components;
pub mod theme;
mod views;

pub use components::{
    render_context_help, CommandPalette, CommandPaletteAction, ConfirmDialog, ErrorDialog,
    ExternalEditor, JqlAction, JqlInput, LoadingIndicator, Notification, NotificationManager,
    ProfilePicker, ProfilePickerAction, SavedFiltersAction, SavedFiltersDialog,
};
pub use theme::{init_theme, load_theme};
pub use views::{
    DeleteProfileDialog, DetailAction, DetailView, FilterPanelAction, FilterPanelView, FormField,
    HelpAction, HelpView, ListAction, ListView, ProfileFormAction, ProfileFormData,
    ProfileFormView, ProfileListAction, ProfileListView, ProfileSummary,
};
