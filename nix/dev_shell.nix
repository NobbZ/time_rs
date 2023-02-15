{
  mkShell,
  helix,
  rust,
  rust-analyzer,
  cargo-nextest,
  nixpkgs-fmt,
  pre-commit,
}: let
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
      rustWithExtensions
      rust-analyzer
      cargo-nextest
      nixpkgs-fmt
      helix
      pre-commit
    ];
  }
