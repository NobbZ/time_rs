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
pub struct Start {}

impl Command for Start {
    fn run(&self, _progress: Arc<Root>, _config: Config) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use figment::Figment;
    use prodash::tree::root::Options;

    use super::*;

    #[test]
    fn basic_operation_succeeds() -> Result<()> {
        let start = Start {};
        let figment = Figment::new().merge(("data_dir", "/"));
        let config = figment.try_into()?;
        let progress: Arc<_> = Options::default().create().into();

        let result = start.run(progress, config);

        assert!(result.is_ok());

        Ok(())
    }
}
