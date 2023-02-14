use std::{env, path::PathBuf};

use clap::Parser;

use time_rs::cli::{commands::Command, Cli, Commands};

const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";

const XDG_DATA_DEFAULT: &str = "~/.local/share";
const XDG_CONFIG_DEFAULT: &str = "~/.config";

const SUFFIX: &str = "timers";

fn env_var_or_default_with_suffix(var: &str, default: &str, suffix: &str) -> PathBuf {
    let base: PathBuf = env::var_os(var).unwrap_or_else(|| default.into()).into();

    base.join(suffix)
}

fn get_data_dir() -> PathBuf {
    env_var_or_default_with_suffix(XDG_DATA_HOME, XDG_DATA_DEFAULT, SUFFIX)
}

fn get_config_dir() -> PathBuf {
    env_var_or_default_with_suffix(XDG_CONFIG_HOME, XDG_CONFIG_DEFAULT, SUFFIX)
}

fn main() {
    let cli = Cli::parse();

    let data_dir = cli.data_dir.unwrap_or_else(get_data_dir);
    let config_dir = cli.config_dir.unwrap_or_else(get_config_dir);

    use Commands::*;

    match &cli.command {
        Some(Start(start)) => start.run(data_dir, config_dir),
        Some(Stop(stop)) => stop.run(data_dir, config_dir),
        Some(Status(status)) => status.run(data_dir, config_dir),
        Some(Summary(summary)) => summary.run(data_dir, config_dir),
        None => todo!("We want to have a dashboard here, later…"),
    };
}
