// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use clap::{Args, Subcommand};
use eyre::{OptionExt, Result};
use gix::{
    clone::PrepareFetch,
    create::{Kind, Options},
    ThreadSafeRepository,
};
use prodash::{tree::Root, unit::display::Mode};

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
    fn run(&self, progress: Arc<Root>, config: Config) -> Result<()> {
        match self.command {
            RepoCommand::Init {} => self.init(config),
            RepoCommand::Sync {} => self.sync(),
            RepoCommand::Clone { .. } => self.clone(progress, config),
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

    fn clone(&self, progress: Arc<Root>, config: Config) -> Result<()> {
        let mode = Mode::with_throughput();
        let steps = prodash::unit::label_and_mode("steps", mode);
        let mut clone_progress = progress.add_child("clone");
        clone_progress.init(Some(4), Some(steps));
        let fetch_progress = clone_progress.add_child("fetch");
        let checout_progress = clone_progress.add_child("checkout");

        let target_folder = config
            .data_dir
            .ok_or_eyre("no datadir specified")?
            .join("repo");

        let RepoCommand::Clone { ref url } = self.command else {
            unreachable!("RepoCommand in clone");
        };
        let gix_url = gix::Url::try_from(url.as_str())?;
        clone_progress.inc();

        std::fs::create_dir_all(&target_folder)?;
        clone_progress.inc();

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
            fetch.fetch_then_checkout(fetch_progress, &gix::interrupt::IS_INTERRUPTED)?;
        clone_progress.inc();

        let (_repo, _checkout_outcome) =
            checkout.main_worktree(checout_progress, &gix::interrupt::IS_INTERRUPTED)?;
        clone_progress.inc();

        Ok(())
    }
}
