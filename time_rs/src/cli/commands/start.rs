// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use clap::Args;
use eyre::Result;

use super::Command;

#[derive(Debug, PartialEq, Eq, Args)]
pub struct Start {}

impl Command for Start {
    fn run(&self, data_dir: PathBuf, config_dir: Vec<PathBuf>) -> Result<()> {
        dbg!((data_dir, config_dir));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_operation_succeeds() {
        let start = Start {};

        let result = start.run("/".into(), vec!["/".into()]);

        assert!(result.is_ok());
    }
}
