// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    thread::sleep,
    time::{Duration, Instant},
};

use clap::{Args, Subcommand};
use eyre::{ensure, OptionExt, Result};
use gix::{
    clone::PrepareFetch,
    create::{Kind, Options},
    NestedProgress, Progress, ThreadSafeRepository,
};
use prodash::{tree::Root, unit::display::Mode};

use super::Command;
use crate::{cli::Cli, config::Config};

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
    /// Delete the repo locally
    Destroy {},
}

impl Command for Repo {
    fn run(&self, progress: Arc<Root>, args: &Cli, config: Config) -> Result<()> {
        match self.command {
            RepoCommand::Init {} => self.init(config),
            RepoCommand::Sync {} => self.sync(),
            RepoCommand::Clone { .. } => self.clone(progress, config),
            RepoCommand::Destroy {} => self.destroy(progress, args, config),
        }
    }
}

impl Repo {
    fn init(&self, config: Config) -> Result<()> {
        let target_folder = config
            .data_dir
            .ok_or_eyre("no datadir specified")?
            .join("repo");

        fs::create_dir_all(&target_folder)?;

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

        fs::create_dir_all(&target_folder)?;
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

    fn destroy(&self, progress: Arc<Root>, args: &Cli, config: Config) -> Result<()> {
        let mode = Mode::with_throughput().and_percentage();
        let files = prodash::unit::label_and_mode("files", mode);

        let mut destroy_progress = progress.add_child("destroying repo");
        destroy_progress.init(Some(0), Some(files));

        ensure!(
            args.force,
            "This operation is destructive and requires `--force` to be present"
        );

        let target_folder = config
            .data_dir
            .ok_or_eyre("no datadir specified")?
            .join("repo");

        remove_folder(target_folder, &mut destroy_progress)?;

        destroy_progress.done("destroyed".into());
        sleep(Duration::from_millis(500));

        Ok(())
    }
}

fn remove_folder<P>(folder: PathBuf, progress: &mut P) -> Result<()>
where
    P: NestedProgress,
    P::SubProgress: 'static,
{
    let files: Vec<_> = fs::read_dir(&folder)?.collect();
    let new_max = progress.max().unwrap_or(0) + files.len();
    progress.set_max(Some(new_max));

    for p in files {
        let p = p?;

        if p.file_type()?.is_dir() {
            remove_folder(p.path(), progress)?;
        } else {
            fs::remove_file(p.path())?;
            progress.inc();
        }
    }

    fs::remove_dir(folder)?;
    progress.inc();

    Ok(())
}
