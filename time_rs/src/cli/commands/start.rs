// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use clap::Args;
use prodash::tree::Root;

use crate::{cli::Cli, config::Config};

use super::Command;
use super::Result;

#[derive(Debug, PartialEq, Eq, Args)]
pub struct Start {}

impl Command for Start {
    async fn run(&self, _progress: Arc<Root>, _args: &Cli, _config: Config) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use figment::Figment;
    use prodash::tree::root::Options;

    use crate::cli::Commands;

    use super::*;

    #[tokio::test]
    async fn basic_operation_succeeds() -> eyre::Result<()> {
        let start = Start {};
        let figment = Figment::new().merge(("data_dir", "/"));
        let cli_args = Cli {
            command: Some(Commands::Start(Start {})),
            ..Default::default()
        };
        let config = figment.try_into()?;
        let progress: Arc<_> = Options::default().create().into();

        let result = start.run(progress, &cli_args, config).await;

        assert!(result.is_ok());

        Ok(())
    }
}
