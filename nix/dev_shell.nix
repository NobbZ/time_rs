# SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
#
# SPDX-License-Identifier: MIT
{
  mkShell,
  helix,
  rust,
  rust-analyzer,
  cargo-nextest,
  cargo-audit,
  pre-commit,
  nil,
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
      rust-analyzer
      cargo-nextest
      cargo-audit
      helix
      pre-commit
      nil
    ];
  }
