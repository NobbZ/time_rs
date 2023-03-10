// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

mod start;
mod status;
mod stop;
mod summary;

pub use start::Start;
pub use status::Status;
pub use stop::Stop;
pub use summary::Summary;

pub trait Command {
    fn run(&self, data_dir: PathBuf, config_dir: Vec<PathBuf>);
}
