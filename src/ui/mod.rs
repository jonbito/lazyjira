//! User interface components and views.
//!
//! This module contains all TUI rendering logic, including views for different
//! screens and reusable UI components.

mod components;
mod theme;
mod views;

pub use components::{Input, Modal, Table};
pub use theme::Theme;
pub use views::{DetailView, FilterView, ListView, ProfileView};
