// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

#![cfg(not(tarpaulin_include))]

use std::{env, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::{OptionExt, Result, WrapErr};
use directories::ProjectDirs;

use time_rs::cli::{commands::Command, Cli, Commands};

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

    // TODO: make this lazy_static
    let project_dirs =
        ProjectDirs::from("dev", "nobbz", SUFFIX).ok_or_eyre("resolving project dirs")?;

    let project_path = project_dirs.project_path();

    dirs.push(project_dirs.config_dir().to_owned());

    env::var("XDG_CONFIG_DIRS")
        .map_or(vec![], |dirs| dirs.split(':').map(PathBuf::from).collect())
        .iter()
        .map(|d| d.join(project_path))
        .for_each(|p| dirs.push(p.to_owned()));

    // TODO: Remove duplicates before returning

    Ok(dirs)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let data_dir = cli.data_dir.unwrap_or_else(get_data_dir);
    let config_dir = cli
        .config_dir
        .map_or_else(get_config_dirs, |d| Ok(vec![d]))?;

    use Commands::*;

    match &cli.command {
        Some(Start(start)) => start.run(data_dir, config_dir).wrap_err("start command")?,
        Some(Stop(stop)) => stop.run(data_dir, config_dir).wrap_err("stop command")?,
        Some(Status(status)) => status
            .run(data_dir, config_dir)
            .wrap_err("status command")?,
        Some(Summary(summary)) => summary
            .run(data_dir, config_dir)
            .wrap_err("summary command")?,
        None => todo!("We want to have a dashboard here, later…"),
    };
}
