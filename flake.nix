# SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
#
# SPDX-License-Identifier: MIT
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    parts.url = "github:hercules-ci/flake-parts";

    oxalica.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    parts,
    ...
  } @ inputs:
    parts.lib.mkFlake {inherit inputs;}
    {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];
      imports = [];

      perSystem = {
        config,
        inputs',
        system,
        pkgs,
        ...
      }: let
        rustTooling = pkgs.callPackage ./nix/rust_platform.nix {};
      in {
        _module.args.pkgs = inputs'.nixpkgs.legacyPackages.extend inputs.oxalica.overlays.default;

        imports = [./nix/per_system/apps.nix];

        formatter = pkgs.alejandra;

        packages = {
        };

        devShells.default = pkgs.callPackage ./nix/dev_shell.nix {
          inherit (rustTooling) rust rust-analyzer;
          inherit (pkgs) nil;
        };
      };
    };
}
