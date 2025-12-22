// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! This module provides the errors that can happen during parsing and loading
//! a configuration.

use std::path::PathBuf;

use figment::Error as FigmentError;
use glob::{GlobError, PatternError};
use thiserror::Error as ThisError;
use tokio::task::JoinError;

#[derive(Debug, ThisError)]
/// Describes errors that might happen during configuration parsing.
pub enum Error {
    #[error("config can not get extracted")]
    /// Signifies, that there was an error extracting the configuration from figment.
    ExtractionError(#[from] Box<FigmentError>),
    #[error("glob pattern matching failed")]
    /// Signifies, that there were no matching files for the glob.
    GlobError(#[from] GlobError),
    #[error("invalid glob pattern: {}", .0)]
    /// Signifies, that the parsing of the glob failed, should never be returned.
    PatternError(String, PatternError),
    #[error("{:?} can't be converted to string", .0)]
    /// A given [`PathBuf`] couldn't be converted to a string.
    PathStringConversion(PathBuf),
    #[error("{:?} has unsupported extension {}", .0, .1)]
    /// A candidate for a config file did not have a supported extension.
    UnknownExtension(PathBuf, String),
    #[error("{:?} has no extension", .0)]
    /// A candidate for a config file did not have an extension at all.
    NoExtension(PathBuf),
    #[error("Unable to load the configuration")]
    /// There was a generic error while loading a configuration.
    LoadingConfig(#[source] Box<Error>),
    #[error("couldn't join work units")]
    /// Joining of different workunits has not been possible.
    JoinError(#[source] JoinError),
}
