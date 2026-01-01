// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Error types for event sourcing operations

use thiserror::Error;

/// Errors that can occur during event sourcing operations
#[derive(Debug, Error)]
pub enum EventSourcingError {
    /// Error occurred while performing git operations
    #[error("Git operation failed: {0}")]
    GitError(String),

    /// Error occurred while serializing or deserializing events
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Event not found in the store
    #[error("Event not found: {0}")]
    EventNotFound(String),

    /// Repository not found or invalid
    #[error("Repository error: {0}")]
    RepositoryError(String),

    /// IO error occurred
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Multiple files in a single commit (violates constraint)
    #[error("Multiple files in commit {commit}: expected 1 file, found {count}")]
    MultipleFilesInCommit {
        /// The commit id
        commit: String,
        /// Number of files found
        count: usize,
    },
}

/// Result type for event sourcing operations
pub type Result<T> = std::result::Result<T, EventSourcingError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_error_display() {
        let error = EventSourcingError::GitError("test error".to_string());
        assert_eq!(error.to_string(), "Git operation failed: test error");
    }

    #[test]
    fn test_serialization_error_conversion() {
        let json_error = serde_json::from_str::<i32>("invalid json").unwrap_err();
        let error: EventSourcingError = json_error.into();
        assert!(matches!(error, EventSourcingError::SerializationError(_)));
    }

    #[test]
    fn test_event_not_found_display() {
        let error = EventSourcingError::EventNotFound("event-123".to_string());
        assert_eq!(error.to_string(), "Event not found: event-123");
    }

    #[test]
    fn test_repository_error_display() {
        let error = EventSourcingError::RepositoryError("invalid repo".to_string());
        assert_eq!(error.to_string(), "Repository error: invalid repo");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: EventSourcingError = io_error.into();
        assert!(matches!(error, EventSourcingError::IoError(_)));
    }

    #[test]
    fn test_multiple_files_in_commit_display() {
        let error = EventSourcingError::MultipleFilesInCommit {
            commit: "abc123".to_string(),
            count: 3,
        };
        assert_eq!(
            error.to_string(),
            "Multiple files in commit abc123: expected 1 file, found 3"
        );
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<EventSourcingError>();
    }
}
