// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use clap::Args;
use eyre::Result;
use prodash::tree::Root;

use crate::{cli::Cli, config::Config};

use super::Command;

#[derive(Debug, PartialEq, Eq, Args)]
pub struct Status {}

impl Command for Status {
    fn run(&self, _progress: Arc<Root>, _args: &Cli, _config: Config) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::Command;
    use crate::cli::Cli;
    use crate::config::Config;
    use prodash::tree::Root;
    use std::sync::Arc;

    #[test]
    fn test_status_run() {
        let temp = assert_fs::TempDir::new().unwrap();
        let home_dir = temp.path().to_path_buf();
        let cli = Cli::default();
        let config = Config::load(vec![home_dir]).unwrap();
        let progress = Arc::new(Root::new());
        let status = Status {};
        let result = status.run(Arc::clone(&progress), &cli, config);
        assert!(result.is_ok());
    }
}
