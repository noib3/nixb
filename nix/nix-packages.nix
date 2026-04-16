{
  fetchFromGitHub,
  generateSplicesForMkScope,
  lib,
  nixDependencies,
  perl,
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
      nixPackageVersion =
        if builtins.match "[0-9]+\\.[0-9]+\\.[0-9]+.*" nixSource.rev != null then
          nixSource.rev
        else
          "${builtins.replaceStrings [ "_" ] [ "." ] nixSourceKey}.0";

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
            version = nixPackageVersion;
          };

      nixComponents = nixComponentsBase.overrideScope (
        _finalScope: prevScope: {
          nix-util = prevScope.nix-util.overrideAttrs (prevAttrs: {
            buildInputs =
              (prevAttrs.buildInputs or [ ])
              ++ lib.optionals (nixSourceKey == "2_35") [ zstd ];
          });

          nix-store = prevScope.nix-store.overrideAttrs (prevAttrs: {
            nativeBuildInputs =
              (prevAttrs.nativeBuildInputs or [ ])
              ++ lib.optionals (nixSourceKey == "2_35") [ perl ];
            postPatch =
              (prevAttrs.postPatch or "")
              + lib.optionalString (nixSourceKey == "2_35") ''
                perl -0pi -e 's/#include <cstring>/#include <cstring>\n#include <regex>/' optimise-store.cc
              '';
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
