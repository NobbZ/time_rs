// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! This module defines several commands that can be used from the CLI via flags.

use std::future::Future;
use std::sync::Arc;

use prodash::tree::Root;
use thiserror::Error as ThisError;

use crate::cli::Cli;
use crate::config::Config;

mod repo;
mod start;
mod status;
mod stop;
mod summary;

use repo::Error as RepoError;
pub use repo::Repo;
pub use start::Start;
pub use status::Status;
pub use stop::Stop;
pub use summary::Summary;

#[derive(Debug, ThisError)]
/// An error that signifies a failed command operation.
pub enum Error {
    #[error("failed repo operation")]
    /// This signifies an error during a repository operation
    Repo(#[from] RepoError),
}

type Result<T> = std::result::Result<T, Error>;

/// Common interface to run subcommands from the CLI.
pub trait Command {
    /// Runs the operation associated with the command.
    fn run(
        &self,
        progress_root: Arc<Root>,
        args: &Cli,
        config: Config,
    ) -> impl Future<Output = Result<()>> + Send;
}
