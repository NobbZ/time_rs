// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use clap::Args;
use eyre::Result;
use prodash::tree::Root;

use crate::config::Config;

use super::Command;

#[derive(Debug, PartialEq, Eq, Args)]
pub struct Status {}

impl Command for Status {
    fn run(&self, _progress: Arc<Root>, _config: Config) -> Result<()> {
        Ok(())
    }
}
