//! Application views (screens).

mod detail;
mod filter;
mod list;
mod profile;

pub use detail::DetailView;
pub use filter::FilterView;
pub use list::{ListAction, ListView};
pub use profile::ProfileView;
