// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use clap::Args;

use super::Command;

#[derive(Debug, PartialEq, Eq, Args)]
pub struct Status {}

impl Command for Status {
    #[cfg(not(tarpaulin_include))]
    fn run(&self, data_dir: PathBuf, config_dir: Vec<PathBuf>) {
        dbg!((data_dir, config_dir));
    }
}
