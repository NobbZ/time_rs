# SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
#
# SPDX-License-Identifier: MIT
{
  rust-bin,
  makeRustPlatform,
}: let
  rust = rust-bin.stable."1.90.0".default;
  rust-analyzer = rust-bin.stable."1.90.0".rust-analyzer;

  rustc = rust;
  cargo = rust;
in {
  inherit rust rustc cargo rust-analyzer;
  rustPlatform = makeRustPlatform {
    inherit rustc cargo;
  };
}
