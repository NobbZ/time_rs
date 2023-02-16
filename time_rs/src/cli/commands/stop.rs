// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use clap::Args;

use super::Command;

#[derive(Debug, PartialEq, Eq, Args)]
pub struct Stop {}

impl Command for Stop {
    #[cfg(not(tarpaulin_include))]
    fn run(&self, _data_dir: PathBuf, _config_dir: Vec<PathBuf>) {
        todo!()
    }
}
