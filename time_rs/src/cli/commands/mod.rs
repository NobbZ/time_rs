// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

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
pub enum Error {
    #[error("failed repo operation")]
    Repo(#[from] RepoError),
}

type Result<T> = std::result::Result<T, Error>;

/// Common interface to run subcommands from the CLI.
pub trait Command {
    fn run(&self, progress_root: Arc<Root>, args: &Cli, config: Config) -> Result<()>;
}
