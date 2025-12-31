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
