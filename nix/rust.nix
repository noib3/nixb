{ inputs, ... }:

{
  perSystem =
    { pkgs, ... }:
    {
      _module.args.rust =
        let
          mkRustBin = targetPkgs: inputs.rust-overlay.lib.mkRustBin { } targetPkgs.buildPackages;
          mkToolchain = targetPkgs: (mkRustBin targetPkgs).fromRustupToolchainFile ../rust-toolchain.toml;
        in
        {
          inherit mkToolchain;
          toolchain = mkToolchain pkgs;
        };
    };
}
