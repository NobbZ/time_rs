{ mkShell, helix, rust, rust-analyzer, cargo2nix, nixpkgs-fmt }:

let
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
    cargo2nix
    rustWithExtensions
    rust-analyzer
    nixpkgs-fmt
    helix
  ];
}
