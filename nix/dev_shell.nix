{ mkShell, rust, nixpkgs-fmt }:

let
  rustWithExtensions = rust.override {
    extensions = [
      "rustfmt-preview"
      "rust-src"
      "rls-preview"
      "rust-analysis"
    ];
  };
in
mkShell {
  name = "timeRS-dev-shell";
  version = "0.0.0";

  packages = [
    rustWithExtensions
    nixpkgs-fmt
  ];
}
