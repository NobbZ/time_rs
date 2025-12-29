// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

#![allow(missing_docs)]

use std::env;

use chrono::{DateTime, TimeZone, Utc};

const SOURCE_DATE_EPOCH: &str = "SOURCE_DATE_EPOCH";

#[allow(missing_docs)]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unwrap_used)]
fn parse_timestamp(s: String) -> DateTime<Utc> {
    let i = s.parse().unwrap_or_else(|e| {
        println!("cargo::warning=Invalid {SOURCE_DATE_EPOCH} '{s}', defaulting to 0");
        println!("cargo::warning=Error has been {e:?}");
        0
    });

    Utc.timestamp_opt(i, 0).earliest().unwrap()
}

fn main() {
    let now = env::var(SOURCE_DATE_EPOCH).map_or_else(|_| Utc::now(), parse_timestamp);

    let today = now.date_naive();

    println!("cargo:rerun-if-env-changed={SOURCE_DATE_EPOCH}");
    println!("cargo:rustc-env=BUILD_DATE={today}");

    let rustc = rustc_version::version().map_or_else(
        |e| {
            println!("cargo::warning=rustc version problem: {e:?}");
            "unknown".to_string()
        },
        |v| v.to_string(),
    );

    println!("cargo:rustc-env=RUST_VERSION={rustc}");
}
