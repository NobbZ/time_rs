// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use figment::Error as FigmentError;
use glob::{GlobError, PatternError};
use thiserror::Error as ThisError;
use tokio::task::JoinError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("config can not get extracted")]
    ExtractionError(#[from] Box<FigmentError>),
    #[error("glob pattern matching failed")]
    GlobError(#[from] GlobError),
    #[error("invalid glob pattern: {}", .0)]
    PatternError(String, PatternError),
    #[error("{:?} can't be converted to string", .0)]
    PathStringConversion(PathBuf),
    #[error("{:?} has unsupported extension {}", .0, .1)]
    UnknownExtension(PathBuf, String),
    #[error("{:?} has no extension", .0)]
    NoExtension(PathBuf),
    #[error("Unable to load the configuration")]
    LoadingConfig(#[source] Box<Error>),
    #[error("couldn't join work units")]
    JoinError(#[source] JoinError),
}
