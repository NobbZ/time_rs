{ mkShell, helix, rust, rust-analyzer, cargo-nextest, cargo2nix, nixpkgs-fmt }:

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
    cargo-nextest
    nixpkgs-fmt
    helix
  ];
}
