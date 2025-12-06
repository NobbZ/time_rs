// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::io::Error as IoError;

use gix::clone::checkout::main_worktree::Error as GixCheckoutError;
use gix::clone::fetch::Error as GixFetchError;
use gix::clone::Error as GixCloneError;
use gix::init::Error as GixInitError;
use gix::url::parse::Error as UrlParseError;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("no datadir specified")]
    NoDataDir,
    #[error("input/output error")]
    Io(#[from] IoError),
    #[error("gix initialisation failed")]
    GixInit(#[from] Box<GixInitError>),
    #[error("gix clone failed")]
    GixClone(#[from] Box<GixCloneError>),
    #[error("gix fetch error")]
    GixFetch(#[from] Box<GixFetchError>),
    #[error("gix checkout error")]
    GixCheckout(#[from] Box<GixCheckoutError>),
    #[error("parsing '{}' into a gix-URL failed", .1)]
    GixUrlParse(#[source] UrlParseError, String),
    #[error("the {} operation is destructive, '--force' required", .0)]
    DestructiveOperation(String),
}
