//! JIRA API client and types.
//!
//! This module provides the interface for communicating with the JIRA REST API.

mod auth;
mod client;
mod types;

pub use auth::Auth;
pub use client::JiraClient;
pub use types::Issue;
