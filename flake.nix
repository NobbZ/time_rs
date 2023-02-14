{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    oxalica.url = "github:oxalica/rust-overlay";

    cargo2nix.url = "github:cargo2nix/cargo2nix";
    cargo2nix.inputs.rust-overlay.follows = "oxalica";
  };

  outputs = { self, nixpkgs, oxalica, cargo2nix }:
    let
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [ oxalica.overlays.default cargo2nix.overlays.default ];
      };

      rustTooling = pkgs.callPackage ./nix/rust_platform.nix {};
      rustPkgs = pkgs.rustBuilder.makePackageSet {
        rustVersion = "2023-02-01";
        rustChannel = "nightly";
        packageFun = import ./Cargo.nix;
      };
    in
      {
        devShells.x86_64-linux.default = pkgs.callPackage ./nix/dev_shell.nix {
          inherit (rustTooling) rust;
          inherit (cargo2nix.packages.x86_64-linux) cargo2nix;
        };

        packages.x86_64-linux.default = (rustPkgs.workspace.time_rs {}).bin;
      };
}
