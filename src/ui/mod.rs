//! User interface components and views.
//!
//! This module contains all TUI rendering logic, including views for different
//! screens and reusable UI components.

mod components;
mod theme;
mod views;

pub use components::{
    ConfirmDialog, ErrorDialog, InlineLoader, LoadingIndicator, Modal, MultiSelect, Notification,
    NotificationManager, NotificationType, ProfilePicker, ProfilePickerAction, SelectItem,
    SpinnerStyle, Table, TextInput,
};
pub use theme::{issue_type_prefix, priority_style, status_style, truncate, Theme};
pub use views::{
    DeleteProfileDialog, DetailAction, DetailView, FilterPanelAction, FilterPanelView, FormField,
    FormMode, ListAction, ListView, ProfileFormAction, ProfileFormData, ProfileFormView,
    ProfileListAction, ProfileListView, ProfileSummary,
};
