// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub mod commands;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Basefolder to use for data, can be influenced by `XDG_DATA_HOME`.
    #[arg(long, short, global = true)]
    pub data_dir: Option<PathBuf>,

    /// Basefolder to use for config, can be influenced by `XDG_CONFIG_HOME`.
    #[arg(long, short, global = true)]
    pub config_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum Commands {
    /// Start a timer
    Start(commands::Start),
    /// Stop a currently running timer
    Stop(commands::Stop),
    /// Show the status of a currently running timer
    Status(commands::Status),
    /// Prints the summary of a given time frame
    Summary(commands::Summary),
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use rstest::*;

    use super::Cli;

    use super::commands::{Start, Status, Stop, Summary};
    use super::Commands;

    #[rstest]
    #[case(Commands::Start(Start {}), "start")]
    #[case(Commands::Stop(Stop {}), "stop")]
    #[case(Commands::Status(Status {}), "status")]
    #[case(Commands::Summary(Summary {}), "summary")]
    fn command_detection(#[case] cmd: Commands, #[case] arg: &str) {
        let cli = Cli::parse_from(["timers", arg]);

        assert_eq!(cmd, cli.command.unwrap());
    }
}
