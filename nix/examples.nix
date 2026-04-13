{
  lib,
  linkFarm,
  makeRustPlatform,
  pkg-config,
  stdenv,
  # --
  nixPackages,
  rust,
}:

let
  rustPlatform = makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };

  sharedLibraryExt = stdenv.hostPlatform.extensions.sharedLibrary;

  mkExamplePackage =
    nixSourceKey: nixPackage: exampleName:
    let
      nixFeature = "nix-${builtins.replaceStrings [ "_" ] [ "-" ] nixSourceKey}";
    in
    rustPlatform.buildRustPackage {
      pname = "example-${exampleName}-${nixFeature}";
      version = "0.1.0";
      src = lib.cleanSource ../.;
      cargoLock.lockFile = ../Cargo.lock;
      strictDeps = true;
      doCheck = false;

      nativeBuildInputs = [
        pkg-config
      ];
      buildInputs = [
        (lib.getDev nixPackage)
      ];

      env.NIX_CC_WRAPPER_SUPPRESS_TARGET_WARNING = "1";

      buildPhase = ''
        runHook preBuild

        export CARGO_TARGET_DIR="$PWD/target"

        cargo build \
          --locked \
          --manifest-path examples/Cargo.toml \
          --no-default-features \
          --features ${nixFeature} \
          --example ${exampleName}

        runHook postBuild
      '';

      installPhase = ''
        runHook preInstall

        mkdir -p "$out/lib"
        cp "$CARGO_TARGET_DIR/debug/examples/lib${exampleName}${sharedLibraryExt}" "$out/lib/"

        runHook postInstall
      '';
    };

  examplesManifest = builtins.fromTOML (builtins.readFile ../examples/Cargo.toml);
  exampleNames = map (example: example.name) examplesManifest.example;

  mkExamplesBundle =
    nixSourceKey: nixPackage:
    let
      nixFeature = "nix-${builtins.replaceStrings [ "_" ] [ "-" ] nixSourceKey}";
      examplePackages = lib.genAttrs exampleNames (
        exampleName: mkExamplePackage nixSourceKey nixPackage exampleName
      );
    in
    (linkFarm "examples-${nixFeature}" (
      map (exampleName: {
        name = "lib/lib${exampleName}${sharedLibraryExt}";
        path = "${examplePackages.${exampleName}}/lib/lib${exampleName}${sharedLibraryExt}";
      }) exampleNames
    )).overrideAttrs
      {
        passthru = examplePackages;
      };
in
lib.mapAttrs' (nixSourceKey: nixPackage: {
  name = "nix-${builtins.replaceStrings [ "_" ] [ "-" ] nixSourceKey}";
  value = mkExamplesBundle nixSourceKey nixPackage;
}) nixPackages
