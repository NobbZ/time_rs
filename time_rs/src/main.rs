// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

#![cfg(not(tarpaulin_include))]
#![warn(clippy::unwrap_used, clippy::expect_used)]

use std::{env, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::{OptionExt, Result, WrapErr};
use directories::ProjectDirs;

use time_rs::{
    cli::{commands::Command, Cli, Commands},
    config::Config,
};

const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

const XDG_DATA_DEFAULT: &str = "~/.local/share";

const SUFFIX: &str = "timers";

fn env_var_or_default_with_suffix(var: &str, default: &str, suffix: &str) -> PathBuf {
    let base: PathBuf = env::var_os(var).unwrap_or_else(|| default.into()).into();

    base.join(suffix)
}

fn get_data_dir() -> PathBuf {
    env_var_or_default_with_suffix(XDG_DATA_HOME, XDG_DATA_DEFAULT, SUFFIX)
}

fn get_config_dirs() -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();

    let project_dirs =
        ProjectDirs::from("dev", "nobbz", SUFFIX).ok_or_eyre("resolving project dirs")?;

    let project_path = project_dirs.project_path();

    dirs.push(project_dirs.config_dir().to_owned());

    env::var("XDG_CONFIG_DIRS")
        .into_iter()
        .for_each(|xdg_dirs| {
            xdg_dirs
                .split(':')
                .map(PathBuf::from)
                .map(|d| d.join(project_path))
                .for_each(|p| dirs.push(p.to_owned()))
        });

    // TODO: Remove duplicates before returning

    Ok(dirs)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let data_dir = cli.data_dir.unwrap_or_else(get_data_dir);
    let config_dir = cli
        .config_dir
        .map_or_else(get_config_dirs, |d| Ok(vec![d]))?;

    let mut config = Config::load(config_dir)?;
    config.add_data_dir(data_dir)?;

    use Commands::*;

    match &cli.command {
        Some(Start(start)) => start.run(config).wrap_err("start command"),
        Some(Stop(stop)) => stop.run(config).wrap_err("stop command"),
        Some(Status(status)) => status.run(config).wrap_err("status command"),
        Some(Summary(summary)) => summary.run(config).wrap_err("summary command"),
        None => todo!("We want to have a dashboard here, later…"),
    }
}
