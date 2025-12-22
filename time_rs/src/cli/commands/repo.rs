// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::{path::PathBuf, sync::Arc, time::Duration};

use clap::{Args, Subcommand};
use gix::{
    clone::PrepareFetch,
    create::{Kind, Options as CreateOptions},
    open::Options as OpenOptions,
    NestedProgress, Progress, ThreadSafeRepository,
};
use prodash::{tree::Root, unit::display::Mode};
use tokio::{fs, task, time::sleep};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

pub use self::error::Error;
use super::{Command, Result};
use crate::{cli::Cli, config::Config};

pub mod error;

#[derive(Debug, PartialEq, Eq, Args, Clone)]
#[allow(missing_docs)]
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
    async fn run(&self, progress: Arc<Root>, args: &Cli, config: Config) -> Result<()> {
        match self.command {
            RepoCommand::Init {} => self.init(config).await,
            RepoCommand::Sync {} => self.sync().await,
            RepoCommand::Clone { .. } => self.clone(progress, config).await,
            RepoCommand::Destroy {} => self.destroy(progress, args, config).await,
        }
    }
}

impl Repo {
    async fn init(&self, config: Config) -> Result<()> {
        let target_folder = config
            .data_dir
            .ok_or_else(|| Error::NoDataDir)?
            .join("repo");

        fs::create_dir_all(&target_folder)
            .await
            .map_err(Error::Io)?;

        task::spawn_blocking(|| {
            ThreadSafeRepository::init(
                target_folder,
                Kind::WithWorktree,
                CreateOptions {
                    destination_must_be_empty: true,
                    ..CreateOptions::default()
                },
            )
        })
        .await
        .map_err(Error::JoinError)?
        .map_err(|e| Error::GixInit(Box::new(e)))?;

        Ok(())
    }

    #[allow(clippy::unused_async)]
    async fn sync(&self) -> Result<()> {
        Ok(())
    }

    async fn clone(&self, progress: Arc<Root>, config: Config) -> Result<()> {
        let mode = Mode::with_throughput();
        let steps = prodash::unit::label_and_mode("steps", mode);
        let mut clone_progress = progress.add_child("clone");
        clone_progress.init(Some(4), Some(steps));
        let fetch_progress = clone_progress.add_child("fetch");
        let checkout_progress = clone_progress.add_child("checkout");

        let data_dir = config.data_dir.ok_or_else(|| Error::NoDataDir)?;
        let target_folder = data_dir.join("repo");

        let RepoCommand::Clone { ref url } = self.command else {
            unreachable!("RepoCommand in clone");
        };
        let gix_url =
            gix::Url::try_from(url.as_str()).map_err(|e| Error::GixUrlParse(e, url.to_string()))?;
        clone_progress.inc();

        fs::create_dir_all(&data_dir).await.map_err(Error::Io)?;
        clone_progress.inc();

        let mut fetch = PrepareFetch::new(
            gix_url,
            target_folder,
            Kind::WithWorktree,
            CreateOptions {
                destination_must_be_empty: true,
                ..CreateOptions::default()
            },
            OpenOptions::default(),
        )
        .map_err(|e| Error::GixClone(Box::new(e)))?;

        let (mut checkout, _fetch_outcome) = fetch
            .fetch_then_checkout(fetch_progress, &gix::interrupt::IS_INTERRUPTED)
            .map_err(|e| Error::GixFetch(Box::new(e)))?;
        clone_progress.inc();

        let (_repo, _checkout_outcome) = checkout
            .main_worktree(checkout_progress, &gix::interrupt::IS_INTERRUPTED)
            .map_err(|e| Error::GixCheckout(Box::new(e)))?;
        clone_progress.inc();

        Ok(())
    }

    async fn destroy(&self, progress: Arc<Root>, args: &Cli, config: Config) -> Result<()> {
        let mode = Mode::with_throughput().and_percentage();
        let files = prodash::unit::label_and_mode("files", mode);

        let mut destroy_progress = progress.add_child("destroying repo");
        destroy_progress.init(Some(0), Some(files));

        if !args.force {
            return Err(Error::DestructiveOperation("repo destroy".to_string()).into());
        }

        let target_folder = config
            .data_dir
            .ok_or_else(|| Error::NoDataDir)?
            .join("repo");

        remove_folder(target_folder, &mut destroy_progress).await?;

        destroy_progress.done("destroyed".into());
        sleep(Duration::from_millis(500)).await;

        Ok(())
    }
}

async fn remove_folder<P>(folder: PathBuf, progress: &mut P) -> Result<()>
where
    P: NestedProgress,
    P::SubProgress: 'static,
{
    let read_dir = fs::read_dir(&folder).await.map_err(Error::Io)?;
    let mut stream = ReadDirStream::new(read_dir);
    let files = {
        let mut vec = vec![];
        while let Some(file) = stream.next().await {
            vec.push(file);
        }
        vec
    };
    let new_max = progress.max().unwrap_or(0) + files.len();
    progress.set_max(Some(new_max));

    for p in files {
        let p = p.map_err(Error::Io)?;

        if p.file_type().await.map_err(Error::Io)?.is_dir() {
            Box::pin(remove_folder(p.path(), progress)).await?;
        } else {
            fs::remove_file(p.path()).await.map_err(Error::Io)?;
            progress.inc();
        }
    }

    fs::remove_dir(folder).await.map_err(Error::Io)?;
    progress.inc();

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_fs::prelude::*;
    use figment::Figment;
    use prodash::{tree::root::Options, unit::display::Mode};
    use rstest::*;

    use super::*;
    use crate::cli::commands::Error as CommandError;
    use crate::cli::Commands;

    #[fixture]
    fn progress() -> Arc<Root> {
        Options::default().create().into()
    }

    #[fixture]
    fn cli_args(#[default(RepoCommand::Init {})] command: RepoCommand) -> Cli {
        Cli {
            command: Some(Commands::Repo(Repo { command })),
            ..Default::default()
        }
    }

    #[fixture]
    fn destroy_repo() -> Repo {
        Repo {
            command: RepoCommand::Destroy {},
        }
    }

    #[rstest]
    #[tokio::test]
    async fn clone_fails_without_data_dir(progress: Arc<Root>) {
        let repo = Repo {
            command: RepoCommand::Clone {
                url: "https://github.com/NobbZ/time_rs".to_string(),
            },
        };

        let figment = Figment::new(); // No data_dir specified
        let config: Config = figment.try_into().unwrap();
        let cli_args = cli_args(repo.command.clone());

        let result = repo.run(progress, &cli_args, config);

        assert!(matches!(
            result.await,
            Err(CommandError::Repo(Error::NoDataDir))
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn clone_fails_with_invalid_url(progress: Arc<Root>) {
        let tmp = assert_fs::TempDir::new().unwrap();

        let repo = Repo {
            command: RepoCommand::Clone {
                url: "not-a-valid-url".to_string(),
            },
        };

        let figment = Figment::new().merge(("data_dir", tmp.path().to_str().unwrap()));
        let config: Config = figment.try_into().unwrap();
        let cli_args = cli_args(repo.command.clone());

        let result = repo.run(progress, &cli_args, config);

        assert!(result.await.is_err());
    }

    #[rstest]
    #[tokio::test]
    async fn clone_fails_when_destination_not_empty(progress: Arc<Root>) {
        let tmp = assert_fs::TempDir::new().unwrap();

        // Create the repo directory with some content to make it non-empty
        tmp.child("repo").create_dir_all().unwrap();
        tmp.child("repo/existing_file.txt")
            .write_str("existing content")
            .unwrap();

        let repo = Repo {
            command: RepoCommand::Clone {
                url: "https://github.com/NobbZ/time_rs".to_string(),
            },
        };

        let figment = Figment::new().merge(("data_dir", tmp.path().to_str().unwrap()));
        let config: Config = figment.try_into().unwrap();
        let cli_args = cli_args(repo.command.clone());

        let result = repo.run(progress, &cli_args, config);

        // Should fail because destination is not empty
        assert!(result.await.is_err());
    }

    #[rstest]
    #[tokio::test]
    async fn init_succeeds_with_empty_directory(progress: Arc<Root>) {
        let tmp = assert_fs::TempDir::new().unwrap();

        let repo = Repo {
            command: RepoCommand::Init {},
        };

        let figment = Figment::new().merge(("data_dir", tmp.path().to_str().unwrap()));
        let config: Config = figment.try_into().unwrap();
        let cli_args = cli_args(repo.command.clone());

        let result = repo.run(progress, &cli_args, config);

        assert!(result.await.is_ok());
        assert!(tmp.child("repo").exists());
    }

    #[rstest]
    #[tokio::test]
    async fn destroy_fails_without_force_flag(
        destroy_repo: Repo,
        progress: Arc<prodash::tree::Root>,
    ) {
        let temp = assert_fs::TempDir::new().unwrap();
        let figment = Figment::new().merge(("data_dir", temp.path().to_str().unwrap()));
        let cli_args = Cli {
            command: Some(Commands::Repo(Repo {
                command: RepoCommand::Destroy {},
            })),
            force: false,
            ..Default::default()
        };
        let config = figment.try_into().unwrap();

        let result = destroy_repo.run(progress, &cli_args, config);

        let err = result.await.unwrap_err();
        assert!(matches!(
            err,
            CommandError::Repo(Error::DestructiveOperation(_operation))
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn destroy_succeeds_with_force_flag(
        destroy_repo: Repo,
        progress: Arc<prodash::tree::Root>,
    ) {
        let temp = assert_fs::TempDir::new().unwrap();
        // Create a repo folder with some content
        let repo_dir = temp.child("repo");
        repo_dir.create_dir_all().unwrap();
        repo_dir.child("test_file.txt").touch().unwrap();
        repo_dir.child("nested").create_dir_all().unwrap();
        repo_dir.child("nested/inner.txt").touch().unwrap();

        let figment = Figment::new().merge(("data_dir", temp.path().to_str().unwrap()));
        let cli_args = Cli {
            command: Some(Commands::Repo(Repo {
                command: RepoCommand::Destroy {},
            })),
            force: true,
            ..Default::default()
        };
        let config = figment.try_into().unwrap();

        let result = destroy_repo.run(progress, &cli_args, config);

        assert!(result.await.is_ok());
        assert!(!repo_dir.exists());
    }

    #[rstest]
    #[tokio::test]
    async fn destroy_fails_when_repo_folder_does_not_exist(
        destroy_repo: Repo,
        progress: Arc<prodash::tree::Root>,
    ) {
        let temp = assert_fs::TempDir::new().unwrap();
        // Don't create repo folder - it should fail when trying to read the directory

        let figment = Figment::new().merge(("data_dir", temp.path().to_str().unwrap()));
        let cli_args = Cli {
            command: Some(Commands::Repo(Repo {
                command: RepoCommand::Destroy {},
            })),
            force: true,
            ..Default::default()
        };
        let config = figment.try_into().unwrap();

        let result = destroy_repo.run(progress, &cli_args, config);

        // The operation should fail because the repo folder doesn't exist
        assert!(result.await.is_err());
    }

    #[tokio::test]
    async fn remove_folder_empty_directory() -> eyre::Result<()> {
        let tmp = assert_fs::TempDir::new()?;
        let test_dir = tmp.child("empty_dir");
        test_dir.create_dir_all()?;

        let progress: Arc<_> = Options::default().create().into();
        let mode = Mode::with_throughput().and_percentage();
        let files = prodash::unit::label_and_mode("files", mode);
        let mut destroy_progress = progress.add_child("test");
        destroy_progress.init(Some(0), Some(files));

        let result = remove_folder(test_dir.path().to_path_buf(), &mut destroy_progress).await;

        assert!(result.is_ok());
        assert!(!test_dir.path().exists());

        Ok(())
    }

    #[tokio::test]
    async fn remove_folder_with_files() -> eyre::Result<()> {
        let tmp = assert_fs::TempDir::new()?;
        let test_dir = tmp.child("dir_with_files");
        test_dir.create_dir_all()?;
        test_dir.child("file1.txt").write_str("content1")?;
        test_dir.child("file2.txt").write_str("content2")?;

        let progress: Arc<_> = Options::default().create().into();
        let mode = Mode::with_throughput().and_percentage();
        let files = prodash::unit::label_and_mode("files", mode);
        let mut destroy_progress = progress.add_child("test");
        destroy_progress.init(Some(0), Some(files));

        let result = remove_folder(test_dir.path().to_path_buf(), &mut destroy_progress).await;

        assert!(result.is_ok());
        assert!(!test_dir.path().exists());

        Ok(())
    }

    #[tokio::test]
    async fn remove_folder_nested_directories() -> eyre::Result<()> {
        let tmp = assert_fs::TempDir::new()?;
        let test_dir = tmp.child("nested");
        test_dir.create_dir_all()?;

        // Create nested structure: nested/level1/level2/file.txt
        let level1 = test_dir.child("level1");
        level1.create_dir_all()?;
        level1.child("file_at_level1.txt").write_str("content")?;

        let level2 = level1.child("level2");
        level2.create_dir_all()?;
        level2.child("file_at_level2.txt").write_str("content")?;

        // Add another sibling directory
        let sibling = test_dir.child("sibling");
        sibling.create_dir_all()?;
        sibling.child("sibling_file.txt").write_str("content")?;

        let progress: Arc<_> = Options::default().create().into();
        let mode = Mode::with_throughput().and_percentage();
        let files = prodash::unit::label_and_mode("files", mode);
        let mut destroy_progress = progress.add_child("test");
        destroy_progress.init(Some(0), Some(files));

        let result = remove_folder(test_dir.path().to_path_buf(), &mut destroy_progress).await;

        assert!(result.is_ok());
        assert!(!test_dir.path().exists());
        assert!(!level1.path().exists());
        assert!(!level2.path().exists());
        assert!(!sibling.path().exists());

        Ok(())
    }

    #[tokio::test]
    async fn remove_folder_updates_progress_correctly() -> eyre::Result<()> {
        let tmp = assert_fs::TempDir::new()?;
        let test_dir = tmp.child("progress_test");
        test_dir.create_dir_all()?;

        // Create a known structure:
        // progress_test/
        //   file1.txt
        //   sub/
        //     file2.txt
        test_dir.child("file1.txt").write_str("content")?;
        let sub = test_dir.child("sub");
        sub.create_dir_all()?;
        sub.child("file2.txt").write_str("content")?;

        let progress: Arc<_> = Options::default().create().into();
        let mode = Mode::with_throughput().and_percentage();
        let files = prodash::unit::label_and_mode("files", mode);
        let mut destroy_progress = progress.add_child("test");
        destroy_progress.init(Some(0), Some(files));

        let result = remove_folder(test_dir.path().to_path_buf(), &mut destroy_progress).await;

        assert!(result.is_ok());
        assert!(!test_dir.path().exists());
        // Progress max counts each entry when read_dir() is called on a directory:
        // - Root folder (progress_test/) has 2 entries: file1.txt and sub/
        // - Subdirectory (sub/) has 1 entry: file2.txt
        // Total max = 2 + 1 = 3
        assert_eq!(destroy_progress.max(), Some(3));

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn sync_succeeds(progress: Arc<Root>) {
        let repo = Repo {
            command: RepoCommand::Sync {},
        };

        let figment = Figment::new();
        let config: Config = figment.try_into().unwrap();
        let cli_args = cli_args(repo.command.clone());

        let result = repo.run(progress, &cli_args, config);

        assert!(result.await.is_ok());
    }

    #[rstest]
    #[tokio::test]
    async fn clone_succeeds(progress: Arc<Root>) {
        let tmp = assert_fs::TempDir::new().unwrap();

        let repo = Repo {
            command: RepoCommand::Clone {
                url: "https://github.com/git-fixtures/basic".to_string(),
            },
        };

        let figment = Figment::new().merge(("data_dir", tmp.path().to_str().unwrap()));
        let config: Config = figment.try_into().unwrap();
        let cli_args = cli_args(repo.command.clone());

        let result = repo.run(progress, &cli_args, config).await;

        if let Err(e) = &result {
            eprintln!("{e:?}");
        }

        assert!(result.is_ok());
        assert!(tmp.child("repo").exists());
        assert!(tmp.child("repo/.git").exists());
    }
}
