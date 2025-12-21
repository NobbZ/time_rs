// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::dbg_macro)]

#[cfg(test)]
#[allow(clippy::single_component_path_imports)]
#[allow(unused_imports)]
use rstest_reuse;

pub mod cli;
pub mod config;
