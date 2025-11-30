// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

#![cfg(not(tarpaulin_include))]
#![warn(clippy::unwrap_used, clippy::expect_used)]

use std::{collections::HashSet, env, path::PathBuf, str::FromStr, sync::Arc};

use clap::Parser;
use color_eyre::eyre::{OptionExt, Result, WrapErr};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use prodash::{tree::root::Options, tree::Root};
use time_rs::{
    cli::{commands::Command, Cli, Commands},
    config::Config,
};

const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

const XDG_DATA_DEFAULT: &str = "~/.local/share";

const SUFFIX: &str = "timers";

lazy_static! {
    static ref PROJECT_DIRS: Option<ProjectDirs> = ProjectDirs::from("dev", "nobbz", SUFFIX);
}

fn env_var_or_default_with_suffix(var: &str, default: &str, suffix: &str) -> PathBuf {
    let base: PathBuf = env::var_os(var).unwrap_or_else(|| default.into()).into();

    base.join(suffix)
}

fn get_data_dir() -> Result<PathBuf> {
    let raw_data_dir = env_var_or_default_with_suffix(XDG_DATA_HOME, XDG_DATA_DEFAULT, SUFFIX)
        .to_string_lossy()
        .to_string();
    let expanded_str = shellexpand::tilde(raw_data_dir.as_str()).to_string();
    Ok(PathBuf::from_str(expanded_str.as_str())?)
}

fn get_config_dirs() -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();

    let project_dirs = PROJECT_DIRS.clone().ok_or_eyre("resolving project dirs")?;

    let project_path = project_dirs.project_path();

    let mut seen = HashSet::new();

    seen.insert(project_dirs.config_dir().to_owned());
    dirs.push(project_dirs.config_dir().to_owned());

    env::var("XDG_CONFIG_DIRS")
        .into_iter()
        .for_each(|xdg_dirs| {
            xdg_dirs
                .split(':')
                .map(PathBuf::from)
                .map(|d| d.join(project_path))
                .for_each(|p| {
                    seen.insert(p.to_owned()).then(|| dirs.push(p.to_owned()));
                })
        });

    Ok(dirs)
}

fn setup_progress() -> Arc<Root> {
    // Create the progress root
    let progress: Arc<_> = Options {
        message_buffer_capacity: 1000,
        ..Default::default()
    }
    .create()
    .into();

    progress
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let progress = setup_progress();
    let mut handle = prodash::render::line::render(
        std::io::stderr(),
        Arc::downgrade(&progress),
        prodash::render::line::Options::default()
            .auto_configure(prodash::render::line::StreamKind::Stderr),
    );
    handle.disconnect();

    let data_dir = match cli.data_dir {
        None => get_data_dir()?,
        Some(ref path) => path.clone(),
    };
    let config_dir = cli
        .config_dir
        .clone()
        .map_or_else(get_config_dirs, |d| Ok(vec![d]))?;

    let mut config = Config::load(config_dir)?;
    config.add_data_dir(data_dir)?;

    use Commands::*;

    match &cli.command {
        Some(Repo(repo)) => repo.run(progress, &cli, config).wrap_err("repo command"),
        Some(Start(start)) => start.run(progress, &cli, config).wrap_err("start command"),
        Some(Status(status)) => status
            .run(progress, &cli, config)
            .wrap_err("status command"),
        Some(Stop(stop)) => stop.run(progress, &cli, config).wrap_err("stop command"),
        Some(Summary(summary)) => summary
            .run(progress, &cli, config)
            .wrap_err("summary command"),
        None => todo!("We want to have a dashboard here, later…"),
    }
}
