// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use clap::{Args, Subcommand};
use eyre::{OptionExt, Result};
use gix::{
    create::{Kind, Options},
    ThreadSafeRepository,
};

use super::Command;
use crate::config::Config;

#[derive(Debug, PartialEq, Eq, Args)]
pub struct Repo {
    #[command(subcommand)]
    command: RepoCommand,
}

#[derive(Debug, PartialEq, Eq, Subcommand, Clone)]
pub enum RepoCommand {
    /// Initialise the repository
    Init {},
    /// synchronizes the repo to the configured remotes
    Sync {},
}

impl Command for Repo {
    fn run(&self, config: Config) -> Result<()> {
        match self.command {
            RepoCommand::Init {} => self.init(config),
            RepoCommand::Sync {} => self.sync(),
        }
    }
}

impl Repo {
    fn init(&self, config: Config) -> Result<()> {
        let target_folder = config
            .data_dir
            .ok_or_eyre("no datadir specified")?
            .join("repo");

        std::fs::create_dir_all(&target_folder)?;

        ThreadSafeRepository::init(
            target_folder,
            Kind::WithWorktree,
            Options {
                destination_must_be_empty: true,
                ..Options::default()
            },
        )?;

        Ok(())
    }

    fn sync(&self) -> Result<()> {
        Ok(())
    }
}
