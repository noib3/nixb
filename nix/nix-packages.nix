{
  fetchFromGitHub,
  fetchpatch,
  generateSplicesForMkScope,
  lib,
  nixDependencies,
  # --,
  nixpkgs,
}:

let
  teams = [
    lib.teams.nix
    lib.teams.security-review
  ];

  lowdown30Patch = fetchpatch {
    name = "nix-lowdown-3.0-support.patch";
    url = "https://github.com/NixOS/nix/commit/472c35c561bd9e8db1465e0677f1efe2cb88c568.patch";
    hash = "sha256-ZCQgI/euBN8t9rgdCsGRgrcEWG3T5MUc+bQc4tIcHuI=";
  };

  mkNixPackage =
    nixSourceKey: nixSource:
    let
      nixComponents =
        (nixDependencies.callPackage
          "${nixpkgs}/pkgs/tools/package-management/nix/modular/packages.nix"
          {
            inherit teams;
            otherSplices = generateSplicesForMkScope [
              "nixVersions"
              "nixComponents_${nixSourceKey}"
            ];
            src = nixSource;
            version = nixSource.rev;
          }
        ).appendPatches
          (
            lib.optionals
              (builtins.elem nixSourceKey [
                "2_32"
                "2_33"
              ])
              [
                lowdown30Patch
              ]
          );
    in
    nixComponents.nix-everything.overrideAttrs {
      doCheck = false;
    };
in
lib.mapAttrs mkNixPackage (
  import ./nix-sources.nix { inherit fetchFromGitHub; }
)
