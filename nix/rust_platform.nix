{
  rust-bin,
  makeRustPlatform,
}: let
  rust = rust-bin.selectLatestNightlyWith (tc: tc.default);

  rustc = rust;
  cargo = rust;
in {
  inherit rust rustc cargo;
  rustPlatform = makeRustPlatform {
    inherit rustc cargo;
  };
}
