{ ... }:

{
  perSystem =
    { pkgs, ... }:
    {
      _module.args.common =
        let
          mkNixVersion = targetPkgs: targetPkgs.nixVersions.nix_2_34;
          mkBuildInputs = targetPkgs: [ (mkNixVersion targetPkgs).dev ];
          mkEnv = targetPkgs: {
            # This silences a warning emitted by the build script of the
            # nixb-cpp crate. See https://github.com/NixOS/nixpkgs/issues/395191
            # and https://github.com/NixOS/nixpkgs/pull/396373 for details.
            NIX_CC_WRAPPER_SUPPRESS_TARGET_WARNING = "1";
          };
        in
        {
          inherit mkBuildInputs mkEnv;
          buildInputs = mkBuildInputs pkgs;
          env = mkEnv pkgs;
          nativeBuildInputs = [ pkgs.pkg-config ];
          nixVersion = mkNixVersion pkgs;
        };
    };
}
