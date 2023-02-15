use std::path::PathBuf;

use clap::Args;

use super::Command;

#[derive(Debug, PartialEq, Eq, Args)]
pub struct Status {}

impl Command for Status {
    fn run(&self, data_dir: PathBuf, config_dir: Vec<PathBuf>) {
        dbg!((data_dir, config_dir));
    }
}
