//! Reusable UI components.

// UI component methods are part of the public API
#![allow(dead_code)]

mod assignee_picker;
mod command_palette;
mod comments;
mod dropdown;
mod external_editor;
mod help_bar;
mod input;
mod issue_search_picker;
mod jql_input;
mod link_type_picker;
mod linked_issues;
mod loading;
mod modal;
mod multiselect;
mod notification;
mod priority_picker;
mod profile_picker;
mod saved_filters;
mod search_bar;
mod table;
mod tag_editor;
mod text_editor;
mod transition_picker;

pub use assignee_picker::{AssigneeAction, AssigneePicker};
pub use command_palette::{CommandPalette, CommandPaletteAction};
pub use dropdown::{Dropdown, DropdownAction, DropdownItem};
pub use comments::{CommentAction, CommentsPanel};
pub use external_editor::ExternalEditor;
pub use help_bar::render_context_help;
pub use input::{InputMode, TextInput};
pub use issue_search_picker::{IssueSearchPicker, IssueSearchPickerAction};
pub use jql_input::{JqlAction, JqlInput};
pub use link_type_picker::{LinkManager, LinkManagerAction};
pub use linked_issues::LinkedIssuesSection;
pub use loading::LoadingIndicator;
pub use modal::{ConfirmDialog, ErrorDialog};
pub use multiselect::{MultiSelect, SelectItem};
pub use notification::{Notification, NotificationManager};
pub use priority_picker::{PriorityAction, PriorityPicker};
pub use profile_picker::{ProfilePicker, ProfilePickerAction};
pub use saved_filters::{SavedFiltersAction, SavedFiltersDialog};
pub use search_bar::{highlight_text, render_search_bar, QuickSearch};
pub use tag_editor::{TagAction, TagEditor};
pub use text_editor::TextEditor;
pub use transition_picker::{TransitionAction, TransitionPicker};
