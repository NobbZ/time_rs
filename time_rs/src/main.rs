// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

#![cfg(not(tarpaulin_include))]

use std::{env, path::PathBuf};

use clap::Parser;
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

fn get_config_dirs() -> Vec<PathBuf> {
    // env_var_or_default_with_suffix(XDG_CONFIG_HOME, XDG_CONFIG_DEFAULT, SUFFIX)

    let mut dirs = Vec::new();

    ProjectDirs::from("dev", "nobbz", SUFFIX)
        .map(|d| d.config_dir().to_owned())
        .iter()
        .for_each(|p| dirs.push(p.to_owned()));

    env::var("XDG_DATA_DIRS")
        .map_or(vec![], |dirs| dirs.split(':').map(PathBuf::from).collect())
        .iter()
        .for_each(|p| dirs.push(p.to_owned()));

    dirs
}

fn main() {
    let cli = Cli::parse();

    let data_dir = cli.data_dir.unwrap_or_else(get_data_dir);
    let config_dir = cli.config_dir.map_or_else(get_config_dirs, |d| vec![d]);

    use Commands::*;

    match &cli.command {
        Some(Start(start)) => start.run(data_dir, config_dir),
        Some(Stop(stop)) => stop.run(data_dir, config_dir),
        Some(Status(status)) => status.run(data_dir, config_dir),
        Some(Summary(summary)) => summary.run(data_dir, config_dir),
        None => todo!("We want to have a dashboard here, later…"),
    };
}
