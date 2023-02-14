use std::{env, path::PathBuf};

use clap::{Parser, Subcommand};

mod commands;

use commands::Command;

const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";

const XDG_DATA_DEFAULT: &str = "~/.local/share";
const XDG_CONFIG_DEFAULT: &str = "~/.config";

const SUFFIX: &str = "timers";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Basefolder to use for data, can be influenced by `XDG_DATA_HOME`.
    #[arg(long, short, global = true)]
    data_dir: Option<PathBuf>,

    /// Basefolder to use for config, can be influenced by `XDG_CONFIG_HOME`.
    #[arg(long, short, global = true)]
    config_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Start a timer
    Start(commands::Start),
    /// Stop a currently running timer
    Stop(commands::Stop),
    /// Show the status of a currently running timer
    Status(commands::Status),
    /// Prints the summary of a given time frame
    Summary(commands::Summary),
}

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
        Some(Start(s)) => s.run(data_dir, config_dir),
        Some(_) => todo!(),
        None => todo!(),
    };
}
