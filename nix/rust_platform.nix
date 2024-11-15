# SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
#
# SPDX-License-Identifier: MIT
{
  rust-bin,
  makeRustPlatform,
}: let
  rust = rust-bin.stable."1.80.0".default;

  rustc = rust;
  cargo = rust;
in {
  inherit rust rustc cargo;
  rustPlatform = makeRustPlatform {
    inherit rustc cargo;
  };
}
