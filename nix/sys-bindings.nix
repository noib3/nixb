{
  lib,
  llvmPackages,
  makeRustPlatform,
  runCommand,
  rust-script,
  rustfmt,
  # --
  rust,
  nixSources,
}:

let
  rustPlatform = makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };

  sysBindingsGeneratorSrc = runCommand "nixb-sys-generator-src" { } ''
    ${lib.getExe rust-script} --package --pkg-path "$out" ${./sys_bindings_generator.rs}
    mkdir -p "$out/src"
    cp ${./sys_bindings_generator.rs} "$out/src/main.rs"
    cp ${./sys_bindings_generator_Cargo.lock} "$out/Cargo.lock"
  '';

  sysBindingsGenerator = rustPlatform.buildRustPackage rec {
    pname = "nixb-sys-generator";
    version = "0.1.0";
    src = sysBindingsGeneratorSrc;
    cargoLock.lockFile = ./sys_bindings_generator_Cargo.lock;
    doCheck = false;
    meta.mainProgram = pname;
  };

  mkSysBindingsFor =
    moduleName: nixSource:
    runCommand "nix-sys-bindings-${nixSource.rev}"
      {
        # The generator needs `rustfmt`, so add the Rust toolchain.
        nativeBuildInputs = [
          (rustfmt.override { asNightly = true; })
        ];
        env = {
          LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
          NIX_SOURCE = nixSource;
          NIX_VERSION = nixSource.rev;
        };
      }
      ''
        mkdir -p "$out"
        OUTPUT_FILE="$out/${moduleName}.rs" ${lib.getExe sysBindingsGenerator}
      '';

  generatedBindings = lib.mapAttrsToList mkSysBindingsFor nixSources;
in
runCommand "sys-bindings" { } ''
  mkdir -p "$out"
  ${lib.concatMapStringsSep "\n" (
    bindings: "cp -R ${bindings}/. \"$out/\""
  ) generatedBindings}
''
