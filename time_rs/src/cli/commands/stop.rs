// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use clap::Args;
use eyre::Result;

use crate::config::Config;

use super::Command;

#[derive(Debug, PartialEq, Eq, Args)]
pub struct Stop {}

impl Command for Stop {
    fn run(&self, _config: Config) -> Result<()> {
        todo!()
    }
}
