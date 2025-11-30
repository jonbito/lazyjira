//! JIRA API client and types.
//!
//! This module provides the interface for communicating with the JIRA REST API v3.
//!
//! # Overview
//!
//! The API module is structured as follows:
//!
//! - [`JiraClient`]: The main client for making API requests
//! - [`auth`]: Authentication handling and keyring integration
//! - [`types`]: Request and response types for the JIRA API
//! - [`error`]: Error types for API operations
//!
//! # Example
//!
//! ```ignore
//! use lazyjira::api::{JiraClient, auth};
//! use lazyjira::config::Profile;
//!
//! // Store a token first
//! auth::store_token("my-profile", "my-api-token")?;
//!
//! // Create a profile
//! let profile = Profile::new(
//!     "my-profile".to_string(),
//!     "https://company.atlassian.net".to_string(),
//!     "user@company.com".to_string(),
//! );
//!
//! // Create a client (validates connection)
//! let client = JiraClient::new(&profile).await?;
//!
//! // Search for issues
//! let results = client.search_issues("project = PROJ", 0, 50).await?;
//! for issue in results.issues {
//!     println!("{}: {}", issue.key, issue.summary());
//! }
//! ```

pub mod auth;
mod client;
pub mod error;
pub mod types;

// Re-export main types for convenience
pub use auth::Auth;
pub use client::JiraClient;
pub use error::{ApiError, Result};
pub use types::{CurrentUser, Issue, IssueFields, SearchResult};
