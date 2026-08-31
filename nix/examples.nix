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

  # Plugin examples are compiled to shared libraries that a Nix process
  # dlopens; the other examples are standalone executables that embed Nix.
  isPluginExample = example: lib.elem "cdylib" (example.crate-type or [ ]);

  mkExamplePackage =
    nixSourceKey: nixPackage: example:
    let
      nixFeature = "nix-${builtins.replaceStrings [ "_" ] [ "-" ] nixSourceKey}";
      isPlugin = isPluginExample example;
      features = [ nixFeature ] ++ (example.required-features or [ ]);
    in
    rustPlatform.buildRustPackage {
      pname = "example-${example.name}-${nixFeature}";
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
          --features ${lib.concatStringsSep "," features} \
          --example ${example.name}

        runHook postBuild
      '';

      installPhase =
        if isPlugin then
          ''
            runHook preInstall

            mkdir -p "$out/lib"
            cp "$CARGO_TARGET_DIR/debug/examples/lib${example.name}${sharedLibraryExt}" "$out/lib/"

            runHook postInstall
          ''
        else
          ''
            runHook preInstall

            mkdir -p "$out/bin"
            cp "$CARGO_TARGET_DIR/debug/examples/${example.name}" "$out/bin/"

            runHook postInstall
          '';
    };

  examplesManifest = builtins.fromTOML (builtins.readFile ../examples/Cargo.toml);

  mkExamplesBundle =
    nixSourceKey: nixPackage:
    let
      nixFeature = "nix-${builtins.replaceStrings [ "_" ] [ "-" ] nixSourceKey}";
      examplePackages = builtins.listToAttrs (
        map (example: {
          name = example.name;
          value = mkExamplePackage nixSourceKey nixPackage example;
        }) examplesManifest.example
      );
    in
    (linkFarm "examples-${nixFeature}" (
      map (
        example:
        let
          relPath =
            if isPluginExample example then
              "lib/lib${example.name}${sharedLibraryExt}"
            else
              "bin/${example.name}";
        in
        {
          name = relPath;
          path = "${examplePackages.${example.name}}/${relPath}";
        }
      ) examplesManifest.example
    )).overrideAttrs
      {
        passthru = examplePackages;
      };
in
lib.mapAttrs' (nixSourceKey: nixPackage: {
  name = "nix-${builtins.replaceStrings [ "_" ] [ "-" ] nixSourceKey}";
  value = mkExamplesBundle nixSourceKey nixPackage;
}) nixPackages
