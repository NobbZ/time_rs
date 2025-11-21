// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use clap::Args;
use eyre::Result;

use crate::config::Config;

use super::Command;

#[derive(Debug, PartialEq, Eq, Args)]
pub struct Start {}

impl Command for Start {
    fn run(&self, _config: Config) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use figment::Figment;

    use super::*;

    #[test]
    fn basic_operation_succeeds() -> Result<()> {
        let start = Start {};
        let figment = Figment::new().merge(("data_dir", "/"));
        let config = figment.try_into()?;

        let result = start.run(config);

        assert!(result.is_ok());

        Ok(())
    }
}
