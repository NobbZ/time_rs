# SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
#
# SPDX-License-Identifier: MIT
{
  mkShell,
  lib,
  rust,
  rust-analyzer,
  cargo-nextest,
  cargo-audit,
  cargo-deny,
  cargo-tarpaulin,
  cargo-mutants,
  pre-commit,
  nil,
  pkg-config,
  openssl,
  bacon,
  cue,
  rustfmt,
}: let
  rustWithExtensions = rust.override {
    extensions = [
      "rust-src"
    ];
  };
in
  mkShell {
    name = lib.pipe ../time_rs/Cargo.toml [
      builtins.readFile
      builtins.fromTOML
      (p: p.package.name)
      (n: "${n}-dev-shell")
    ];

    version = lib.pipe ../Cargo.toml [
      builtins.readFile
      builtins.fromTOML
      (ws: ws.workspace.package.version)
      (v: "${v}-dev")
    ];

    packages = [
      rustfmt
      rustWithExtensions
      bacon
      rust-analyzer
      cargo-nextest
      cargo-audit
      cargo-deny
      cargo-tarpaulin
      cargo-mutants
      pre-commit
      nil
      pkg-config
      openssl
      cue
    ];

    shellHook = ''
      export SOURCE_DATE_EPOCH=365515200
    '';
  }
