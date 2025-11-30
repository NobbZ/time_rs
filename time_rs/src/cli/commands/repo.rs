// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use clap::{Args, Subcommand};
use eyre::{OptionExt, Result};
use gix::{
    clone::PrepareFetch,
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
    /// Clone a repo from a remote
    Clone {
        /// URL
        url: String,
    },
}

impl Command for Repo {
    fn run(&self, config: Config) -> Result<()> {
        match self.command {
            RepoCommand::Init {} => self.init(config),
            RepoCommand::Sync {} => self.sync(),
            RepoCommand::Clone { .. } => self.clone(config),
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

    fn clone(&self, config: Config) -> Result<()> {
        let target_folder = config
            .data_dir
            .ok_or_eyre("no datadir specified")?
            .join("repo");

        let RepoCommand::Clone { ref url } = self.command else {
            unreachable!("RepoCommand in clone");
        };
        let gix_url = gix::Url::try_from(url.as_str())?;

        std::fs::create_dir_all(&target_folder)?;

        let mut fetch = PrepareFetch::new(
            gix_url,
            target_folder,
            Kind::WithWorktree,
            Options {
                destination_must_be_empty: true,
                ..Options::default()
            },
            Default::default(),
        )?;

        let (mut checkout, _fetch_outcome) =
            fetch.fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

        let (_repo, _checkout_outcome) =
            checkout.main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

        Ok(())
    }
}
