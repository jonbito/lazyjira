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

pub use create_issue::{CreateIssueAction, CreateIssueRenderData, CreateIssueView};
pub use detail::{DetailAction, DetailView};
pub use filter::{FilterPanelAction, FilterPanelView};
pub use help::{HelpAction, HelpView};
pub use list::{ListAction, ListView};
pub use profile::{
    DeleteProfileDialog, FormField, ProfileFormAction, ProfileFormData, ProfileFormView,
    ProfileListAction, ProfileListView, ProfileSummary,
};
