{
  lib,
  stdenv,
  writeShellApplication,
  # --
  examplesPackages,
  nixPackages,
}:

let
  sharedLibraryExt = stdenv.hostPlatform.extensions.sharedLibrary;
  nixFeature =
    nixPackage:
    "nix-${
      builtins.replaceStrings [ "." ] [ "-" ] (
        lib.versions.majorMinor (lib.getVersion nixPackage)
      )
    }";
  versionedNixPackages = lib.filterAttrs (
    nixSourceKey: _: builtins.match "[0-9]+_[0-9]+" nixSourceKey != null
  ) nixPackages;

  mkCheck =
    nixPackage: exampleName: examplePackage:
    writeShellApplication {
      name = "check-example-${exampleName}-${lib.getVersion nixPackage}";
      runtimeInputs = [ nixPackage ];
      text = ''
        shopt -s nullglob

        # Plugin examples are loaded into a Nix process.
        for plugin_file in ${examplePackage}/lib/*${sharedLibraryExt}; do
          ${lib.getExe nixPackage} eval \
            --option plugin-files "$plugin_file" \
            --expr null \
            >/dev/null
        done

        # Embedded examples are standalone executables.
        for embedded_bin in ${examplePackage}/bin/*; do
          "$embedded_bin"
        done
      '';
    };

  mkChecks =
    nixPackage: examplesBundle:
    let
      exampleChecks = lib.mapAttrs' (
        exampleName: examplePackage:
        lib.nameValuePair exampleName (mkCheck nixPackage exampleName examplePackage)
      ) examplesBundle.passthru;
    in
    (writeShellApplication {
      name = "check-examples-${lib.getVersion nixPackage}";
      text = lib.concatStringsSep "\n" (
        map lib.getExe (builtins.attrValues exampleChecks)
      );
    }).overrideAttrs
      {
        passthru = exampleChecks;
      };
in
lib.mapAttrs' (_nixSourceKey: nixPackage: {
  name = nixFeature nixPackage;
  value = mkChecks nixPackage examplesPackages.${nixFeature nixPackage};
}) versionedNixPackages
