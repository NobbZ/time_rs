{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    parts.url = "github:hercules-ci/flake-parts";

    oxalica.url = "github:oxalica/rust-overlay";

    nobbz.url = "github:nobbz/nixos-config";

    dream2nix.url = "github:nix-community/dream2nix";
    dream2nix.inputs.nixpkgs.follows = "nixpkgs";
    dream2nix.inputs.all-cabal-json.follows = "nixpkgs";
  };

  outputs = {
    self,
    parts,
    ...
  } @ inputs:
    parts.lib.mkFlake {inherit inputs;}
    {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];
      imports = [inputs.dream2nix.flakeModuleBeta];

      perSystem = {
        inputs',
        system,
        pkgs,
        ...
      }: let
        rustTooling = pkgs.callPackage ./nix/rust_platform.nix {};
      in {
        _module.args.pkgs = inputs'.nixpkgs.legacyPackages.extend inputs.oxalica.overlays.default;

        formatter = inputs'.nobbz.formatter;

        dream2nix.inputs.timers = {
          source = self;
          projects.timers = {
            subsystem = "rust";
            translator = "cargo-lock";
            # builder = "crane";
          };
          packageOverrides."^.*" = {
            set-toolchain.overrideRustToolchain = _: {
              inherit (rustTooling) rustc cargo;
            };
          };
        };

        devShells.default = pkgs.callPackage ./nix/dev_shell.nix {
          inherit (rustTooling) rust;
        };
      };
    };
}
