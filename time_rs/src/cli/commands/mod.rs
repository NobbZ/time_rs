// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use eyre::Result;

use crate::config::Config;

mod repo;
mod start;
mod status;
mod stop;
mod summary;

pub use repo::Repo;
pub use start::Start;
pub use status::Status;
pub use stop::Stop;
pub use summary::Summary;

/// Common interface to run subcommands from the CLI.
pub trait Command {
    fn run(&self, config: Config) -> Result<()>;
}
