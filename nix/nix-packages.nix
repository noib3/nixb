{
  fetchFromGitHub,
  generateSplicesForMkScope,
  lib,
  nixDependencies,
  zstd,
  # --,
  nixpkgs,
}:

let
  teams = [
    lib.teams.nix
    lib.teams.security-review
  ];

  mkNixPackage =
    nixSourceKey: nixSource:
    let
      nixComponentsBase =
        nixDependencies.callPackage
          "${nixpkgs}/pkgs/tools/package-management/nix/modular/packages.nix"
          {
            inherit teams;
            otherSplices = generateSplicesForMkScope [
              "nixVersions"
              "nixComponents_${nixSourceKey}"
            ];
            src = nixSource;
            version = nixSource.rev;
          };

      nixComponents = nixComponentsBase.overrideScope (
        _finalScope: prevScope: {
          nix-util = prevScope.nix-util.overrideAttrs (prevAttrs: {
            buildInputs =
              (prevAttrs.buildInputs or [ ])
              ++ lib.optionals (nixSourceKey == "2_35") [ zstd ];
          });
        }
      );
    in
    nixComponents.nix-everything.overrideAttrs {
      doCheck = false;
    };
in
lib.mapAttrs mkNixPackage (
  import ./nix-sources.nix { inherit fetchFromGitHub; }
)
