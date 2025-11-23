# SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
#
# SPDX-License-Identifier: MIT
{
  mkShell,
  rust,
  rust-analyzer,
  cargo-nextest,
  cargo-audit,
  cargo-deny,
  cargo-tarpaulin,
  pre-commit,
  nil,
  pkg-config,
  openssl,
  bacon,
}: let
  rustWithExtensions = rust.override {
    extensions = [
      "rust-src"
    ];
  };
in
  mkShell {
    name = "timeRS-dev-shell";
    version = "0.0.0";

    packages = [
      rustWithExtensions
      bacon
      rust-analyzer
      cargo-nextest
      cargo-audit
      cargo-deny
      cargo-tarpaulin
      pre-commit
      nil
      pkg-config
      openssl
    ];
  }
