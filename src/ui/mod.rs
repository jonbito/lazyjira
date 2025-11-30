//! User interface components and views.
//!
//! This module contains all TUI rendering logic, including views for different
//! screens and reusable UI components.

mod components;
mod theme;
mod views;

pub use components::{
    ConfirmDialog, ErrorDialog, InlineLoader, Input, LoadingIndicator, Modal, Notification,
    NotificationManager, NotificationType, ProfilePicker, ProfilePickerAction, SpinnerStyle, Table,
};
pub use theme::{issue_type_prefix, priority_style, status_style, truncate, Theme};
pub use views::{DetailAction, DetailView, FilterView, ListAction, ListView, ProfileView};
