// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use clap::Args;

use super::Command;

#[derive(Debug, PartialEq, Eq, Args)]
pub struct Start {}

impl Command for Start {
    fn run(&self, data_dir: PathBuf, config_dir: Vec<PathBuf>) {
        dbg!((data_dir, config_dir));
    }
}
