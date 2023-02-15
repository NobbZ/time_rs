{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    parts.url = "github:hercules-ci/flake-parts";

    oxalica.url = "github:oxalica/rust-overlay";

    nobbz.url = "github:nobbz/nixos-config";

    cargo2nix.url = "github:cargo2nix/cargo2nix";
    cargo2nix.inputs.rust-overlay.follows = "oxalica";
  };

  outputs = {
    self,
    parts,
    ...
  } @ inputs:
    parts.lib.mkFlake {inherit inputs;}
    {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];

      perSystem = {
        inputs',
        system,
        pkgs,
        ...
      }: let
        rustTooling = pkgs.callPackage ./nix/rust_platform.nix {};
        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.67.0";
          rustChannel = "stable";
          packageFun = import ./Cargo.nix;
        };
      in {
        _module.args.pkgs = (inputs'.nixpkgs.legacyPackages.extend inputs.oxalica.overlays.default).extend inputs.cargo2nix.overlays.default;

        formatter = inputs'.nobbz.formatter;

        devShells.default = pkgs.callPackage ./nix/dev_shell.nix {
          inherit (rustTooling) rust;
          inherit (inputs'.cargo2nix.packages) cargo2nix;
        };
      };
    };
}
