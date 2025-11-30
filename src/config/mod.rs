//! Configuration management for LazyJira.
//!
//! This module handles loading, saving, and managing user configuration
//! including profiles and application settings.

mod profile;
mod settings;

pub use profile::Profile;
pub use settings::Settings;
