use std::path::PathBuf;

use clap::Args;

use super::Command;

#[derive(Debug, Args)]
pub struct Summary {}

impl Command for Summary {
    fn run(&self, data_dir: PathBuf, config_dir: PathBuf) {
        dbg!((data_dir, config_dir));
    }
}
