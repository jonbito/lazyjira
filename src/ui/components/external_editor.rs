//! External editor component for editing content in the user's preferred text editor.
//!
//! This module provides functionality to:
//! - Detect the user's preferred editor from environment variables
//! - Create temporary files with content for editing
//! - Launch the external editor and wait for it to complete
//! - Read back the modified content and clean up temporary files

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use thiserror::Error;

/// Errors that can occur during external editor operations.
#[derive(Debug, Error)]
pub enum ExternalEditorError {
    /// Failed to create the temporary file.
    #[error("Failed to create temporary file: {0}")]
    TempFileCreation(#[source] io::Error),

    /// Failed to spawn the editor process.
    #[error("Failed to launch editor '{editor}': {source}")]
    EditorSpawn {
        editor: String,
        #[source]
        source: io::Error,
    },

    /// Editor exited with a non-zero status code.
    #[error("Editor exited with status code {0}")]
    EditorExecution(i32),

    /// Editor was terminated by a signal.
    #[error("Editor was terminated by a signal")]
    EditorTerminated,

    /// Failed to read content back from the temporary file.
    #[error("Failed to read content from temporary file: {0}")]
    ContentRead(#[source] io::Error),
}

/// Result of an external editor session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalEditResult {
    /// The content after editing.
    pub content: String,
    /// Whether the content was modified from the original.
    pub was_modified: bool,
}

/// External editor utility for launching text editors with temporary files.
#[derive(Debug, Clone)]
pub struct ExternalEditor {
    /// The editor command to use.
    editor: String,
}

impl ExternalEditor {
    /// Create a new external editor instance.
    ///
    /// Detects the editor from environment variables in this order:
    /// 1. `$EDITOR`
    /// 2. `$VISUAL`
    /// 3. Falls back to `vi`
    pub fn new() -> Self {
        Self {
            editor: get_editor(),
        }
    }

    /// Create an external editor instance with a specific editor command.
    pub fn with_editor(editor: impl Into<String>) -> Self {
        Self {
            editor: editor.into(),
        }
    }

    /// Get the editor command that will be used.
    pub fn editor(&self) -> &str {
        &self.editor
    }

    /// Open content in the external editor for editing.
    ///
    /// This method:
    /// 1. Creates a temporary file with the given content
    /// 2. Launches the editor with that file
    /// 3. Waits for the editor to exit
    /// 4. Reads back the modified content
    /// 5. Cleans up the temporary file
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The JIRA issue key (used in temp file naming)
    /// * `content` - The initial content to edit
    ///
    /// # Returns
    ///
    /// Returns `Ok(ExternalEditResult)` with the new content and whether it was modified,
    /// or an error if any step fails.
    pub fn open(
        &self,
        issue_key: &str,
        content: &str,
    ) -> Result<ExternalEditResult, ExternalEditorError> {
        // Create the temporary file
        let temp_path = create_temp_file(issue_key, content)?;

        // Launch the editor and wait for it to exit
        let result = self.launch_editor(&temp_path);

        // Always try to read and clean up, even if editor failed
        let read_result = read_and_cleanup(&temp_path, content);

        // Return editor error if it occurred
        result?;

        // Return content or read error
        read_result
    }

    /// Launch the editor process with the given file path.
    fn launch_editor(&self, path: &PathBuf) -> Result<(), ExternalEditorError> {
        let status = Command::new(&self.editor).arg(path).status().map_err(|e| {
            ExternalEditorError::EditorSpawn {
                editor: self.editor.clone(),
                source: e,
            }
        })?;

        if status.success() {
            Ok(())
        } else {
            match status.code() {
                Some(code) => Err(ExternalEditorError::EditorExecution(code)),
                None => Err(ExternalEditorError::EditorTerminated),
            }
        }
    }
}

impl Default for ExternalEditor {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect the user's preferred editor from environment variables.
///
/// Checks in this order:
/// 1. `$EDITOR`
/// 2. `$VISUAL`
/// 3. Falls back to `vi`
pub fn get_editor() -> String {
    env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string())
}

/// Create a temporary file with the given content.
///
/// The file is created in the system's temp directory with the pattern:
/// `lazyjira-{issue_key}-{pid}.md`
fn create_temp_file(issue_key: &str, content: &str) -> Result<PathBuf, ExternalEditorError> {
    let temp_dir = env::temp_dir();
    let filename = format!("lazyjira-{}-{}.md", issue_key, std::process::id());
    let path = temp_dir.join(filename);

    fs::write(&path, content).map_err(ExternalEditorError::TempFileCreation)?;

    Ok(path)
}

/// Read content from the temporary file and clean it up.
///
/// This function always attempts to delete the temporary file,
/// logging a warning if deletion fails but not propagating that error.
fn read_and_cleanup(
    path: &PathBuf,
    original_content: &str,
) -> Result<ExternalEditResult, ExternalEditorError> {
    // Read the content
    let content = fs::read_to_string(path).map_err(ExternalEditorError::ContentRead)?;

    // Clean up the temp file (non-fatal if it fails)
    if let Err(e) = fs::remove_file(path) {
        tracing::warn!("Failed to clean up temporary file {:?}: {}", path, e);
    }

    Ok(ExternalEditResult {
        was_modified: content != original_content,
        content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_editor_with_editor_env() {
        // Save original values
        let original_editor = env::var("EDITOR").ok();
        let original_visual = env::var("VISUAL").ok();

        // Set EDITOR
        env::set_var("EDITOR", "nvim");
        env::remove_var("VISUAL");

        let editor = get_editor();
        assert_eq!(editor, "nvim");

        // Restore original values
        match original_editor {
            Some(val) => env::set_var("EDITOR", val),
            None => env::remove_var("EDITOR"),
        }
        match original_visual {
            Some(val) => env::set_var("VISUAL", val),
            None => env::remove_var("VISUAL"),
        }
    }

    #[test]
    fn test_get_editor_with_visual_env() {
        // Save original values
        let original_editor = env::var("EDITOR").ok();
        let original_visual = env::var("VISUAL").ok();

        // Set only VISUAL
        env::remove_var("EDITOR");
        env::set_var("VISUAL", "emacs");

        let editor = get_editor();
        assert_eq!(editor, "emacs");

        // Restore original values
        match original_editor {
            Some(val) => env::set_var("EDITOR", val),
            None => env::remove_var("EDITOR"),
        }
        match original_visual {
            Some(val) => env::set_var("VISUAL", val),
            None => env::remove_var("VISUAL"),
        }
    }

    #[test]
    fn test_get_editor_fallback_to_vi() {
        // Save original values
        let original_editor = env::var("EDITOR").ok();
        let original_visual = env::var("VISUAL").ok();

        // Remove both
        env::remove_var("EDITOR");
        env::remove_var("VISUAL");

        let editor = get_editor();
        assert_eq!(editor, "vi");

        // Restore original values
        match original_editor {
            Some(val) => env::set_var("EDITOR", val),
            None => env::remove_var("EDITOR"),
        }
        match original_visual {
            Some(val) => env::set_var("VISUAL", val),
            None => env::remove_var("VISUAL"),
        }
    }

    #[test]
    fn test_editor_priority_editor_over_visual() {
        // Save original values
        let original_editor = env::var("EDITOR").ok();
        let original_visual = env::var("VISUAL").ok();

        // Set both
        env::set_var("EDITOR", "vim");
        env::set_var("VISUAL", "code");

        let editor = get_editor();
        assert_eq!(editor, "vim"); // EDITOR takes priority

        // Restore original values
        match original_editor {
            Some(val) => env::set_var("EDITOR", val),
            None => env::remove_var("EDITOR"),
        }
        match original_visual {
            Some(val) => env::set_var("VISUAL", val),
            None => env::remove_var("VISUAL"),
        }
    }

    #[test]
    fn test_external_editor_new() {
        let editor = ExternalEditor::new();
        // Should return some editor (either from env or fallback)
        assert!(!editor.editor().is_empty());
    }

    #[test]
    fn test_external_editor_with_editor() {
        let editor = ExternalEditor::with_editor("nano");
        assert_eq!(editor.editor(), "nano");
    }

    #[test]
    fn test_create_temp_file() {
        let content = "Test content for JIRA issue";
        let issue_key = "TEST-123";

        let path = create_temp_file(issue_key, content).expect("Should create temp file");

        // Verify the file exists and contains the content
        assert!(path.exists());
        assert!(path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("lazyjira-TEST-123-"));
        assert!(path.file_name().unwrap().to_str().unwrap().ends_with(".md"));

        let read_content = fs::read_to_string(&path).expect("Should read file");
        assert_eq!(read_content, content);

        // Clean up
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_create_temp_file_naming_pattern() {
        let path = create_temp_file("PROJ-456", "content").expect("Should create temp file");
        let filename = path.file_name().unwrap().to_str().unwrap();

        // Should match pattern: lazyjira-{issue_key}-{pid}.md
        assert!(filename.starts_with("lazyjira-PROJ-456-"));
        assert!(filename.ends_with(".md"));

        // Should contain the process ID
        let pid = std::process::id().to_string();
        assert!(filename.contains(&pid));

        // Clean up
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_read_and_cleanup() {
        // Create a temp file manually
        let temp_dir = env::temp_dir();
        let path = temp_dir.join(format!("test-cleanup-{}.md", std::process::id()));
        let original = "original content";
        let modified = "modified content";

        fs::write(&path, modified).expect("Should write file");

        let result = read_and_cleanup(&path, original).expect("Should read and cleanup");

        assert_eq!(result.content, modified);
        assert!(result.was_modified);
        assert!(!path.exists(), "File should be cleaned up");
    }

    #[test]
    fn test_read_and_cleanup_unchanged() {
        let temp_dir = env::temp_dir();
        let path = temp_dir.join(format!("test-unchanged-{}.md", std::process::id()));
        let content = "same content";

        fs::write(&path, content).expect("Should write file");

        let result = read_and_cleanup(&path, content).expect("Should read and cleanup");

        assert_eq!(result.content, content);
        assert!(!result.was_modified);
        assert!(!path.exists(), "File should be cleaned up");
    }

    #[test]
    fn test_external_edit_result_eq() {
        let result1 = ExternalEditResult {
            content: "test".to_string(),
            was_modified: true,
        };
        let result2 = ExternalEditResult {
            content: "test".to_string(),
            was_modified: true,
        };
        let result3 = ExternalEditResult {
            content: "other".to_string(),
            was_modified: true,
        };

        assert_eq!(result1, result2);
        assert_ne!(result1, result3);
    }

    #[test]
    fn test_editor_spawn_error_display() {
        let error = ExternalEditorError::EditorSpawn {
            editor: "nonexistent-editor".to_string(),
            source: io::Error::new(io::ErrorKind::NotFound, "command not found"),
        };
        let msg = format!("{}", error);
        assert!(msg.contains("nonexistent-editor"));
        assert!(msg.contains("Failed to launch editor"));
    }

    #[test]
    fn test_editor_execution_error_display() {
        let error = ExternalEditorError::EditorExecution(1);
        let msg = format!("{}", error);
        assert!(msg.contains("status code 1"));
    }

    #[test]
    fn test_temp_file_creation_error_display() {
        let error = ExternalEditorError::TempFileCreation(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "no access",
        ));
        let msg = format!("{}", error);
        assert!(msg.contains("temporary file"));
    }

    #[test]
    fn test_content_read_error_display() {
        let error =
            ExternalEditorError::ContentRead(io::Error::new(io::ErrorKind::NotFound, "file gone"));
        let msg = format!("{}", error);
        assert!(msg.contains("read content"));
    }

    #[test]
    fn test_editor_terminated_error_display() {
        let error = ExternalEditorError::EditorTerminated;
        let msg = format!("{}", error);
        assert!(msg.contains("terminated by a signal"));
    }

    #[test]
    fn test_default_external_editor() {
        let editor = ExternalEditor::default();
        assert!(!editor.editor().is_empty());
    }

    #[test]
    fn test_open_with_nonexistent_editor() {
        let editor = ExternalEditor::with_editor("nonexistent-editor-that-does-not-exist-12345");
        let result = editor.open("TEST-1", "content");

        assert!(result.is_err());
        match result.unwrap_err() {
            ExternalEditorError::EditorSpawn { editor: name, .. } => {
                assert_eq!(name, "nonexistent-editor-that-does-not-exist-12345");
            }
            other => panic!("Expected EditorSpawn error, got {:?}", other),
        }
    }
}
