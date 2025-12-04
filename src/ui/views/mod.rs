//! Application views (screens).

// View methods are part of the public API
#![allow(dead_code)]

mod create_issue;
mod detail;
mod filter;
mod help;
mod history;
mod list;
mod profile;

// CreateIssueView will be used by Task 4.2 (keybinding) and Task 5 (main loop integration)
#[allow(unused_imports)]
pub use create_issue::{CreateIssueAction, CreateIssueView};
pub use detail::{DetailAction, DetailView};
pub use filter::{FilterPanelAction, FilterPanelView};
pub use help::{HelpAction, HelpView};
pub use list::{ListAction, ListView};
pub use profile::{
    DeleteProfileDialog, FormField, ProfileFormAction, ProfileFormData, ProfileFormView,
    ProfileListAction, ProfileListView, ProfileSummary,
};
