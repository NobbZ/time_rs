{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-21.05";
    naersk = { url = "github:nmattia/naersk"; inputs.nixpkgs.follows = "nixpkgs"; };
    oxalica.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, naersk, oxalica }:
    let
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [ oxalica.overlay ];
      };

      naerskLib = pkgs.callPackage naersk {
        cargo = rustTooling.rust;
        rustc = rustTooling.rust;
      };

      rustTooling = pkgs.callPackage ./nix/rust_platform.nix {};
    in
      {
        devShell.x86_64-linux = pkgs.callPackage ./nix/dev_shell.nix {
          inherit (rustTooling) rust;
        };
      };
}
