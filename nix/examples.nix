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

  examplesSrc =
    let
      root = toString ../.;
      keepRootFile = name: name == "Cargo.lock" || (lib.hasSuffix ".toml" name);
    in
    lib.cleanSourceWith {
      src = ../.;
      filter =
        path: _type:
        let
          pathStr = toString path;
          relPath = lib.removePrefix (root + "/") pathStr;
          parts = lib.splitString "/" relPath;
          topLevel = builtins.head parts;
        in
        pathStr == root
        || builtins.elem topLevel [
          "crates"
          "examples"
        ]
        || (builtins.length parts == 1 && keepRootFile relPath);
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
      src = examplesSrc;
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
