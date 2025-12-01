//! Application views (screens).

mod detail;
mod filter;
mod help;
mod history;
mod list;
mod profile;

pub use detail::{DetailAction, DetailView, EditField, EditState};
pub use filter::{FilterPanelAction, FilterPanelView};
pub use help::{HelpAction, HelpView};
pub use history::{HistoryAction, HistoryFilter, HistoryView};
pub use list::{ListAction, ListView};
pub use profile::{
    DeleteProfileDialog, FormField, FormMode, ProfileFormAction, ProfileFormData, ProfileFormView,
    ProfileListAction, ProfileListView, ProfileSummary,
};
