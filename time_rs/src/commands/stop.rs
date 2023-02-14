use std::path::PathBuf;

use clap::Args;

use super::Command;

#[derive(Debug, Args)]
pub struct Stop {}

impl Command for Stop {
    fn run(&self, _data_dir: PathBuf, _config_dir: PathBuf) {
        todo!()
    }
}
