# SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
#
# SPDX-License-Identifier: MIT
{
  rust-bin,
  makeRustPlatform,
}: let
  rust = rust-bin.stable."1.90.0".default;
  rust-analyzer = rust-bin.stable."1.90.0".rust-analyzer;

  # rust = rust-bin.nightly."2025-12-01".default;
  # rust-analyzer = rust-bin.nightly."2025-12-01".rust-analyzer;
  rustfmt = rust-bin.nightly.latest.rustfmt;

  rustc = rust;
  cargo = rust;
in {
  inherit rust rustc cargo rust-analyzer rustfmt;
  rustPlatform = makeRustPlatform {
    inherit rustc cargo;
  };
}
