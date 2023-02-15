{
  rust-bin,
  makeRustPlatform,
}: let
  rust = rust-bin.stable."1.67.0".default;

  rustc = rust;
  cargo = rust;
in {
  inherit rust rustc cargo;
  rustPlatform = makeRustPlatform {
    inherit rustc cargo;
  };
}
