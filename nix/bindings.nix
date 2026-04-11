{ ... }:

{
  imports = [
    ./rust.nix
  ];

  perSystem =
    {
      pkgs,
      lib,
      self',
      rust,
      ...
    }:
    let
      rustPlatform = pkgs.makeRustPlatform {
        cargo = rust.mkToolchain pkgs;
        rustc = rust.mkToolchain pkgs;
      };

      sysBindingsGeneratorSrc = pkgs.runCommand "nixb-sys-generator-src" { } ''
        ${lib.getExe pkgs.rust-script} --package --pkg-path "$out" ${./bindings_generator.rs}
        mkdir -p "$out/src"
        cp ${./bindings_generator.rs} "$out/src/main.rs"
        cp ${./bindings_generator_Cargo.lock} "$out/Cargo.lock"
      '';

      sysBindingsGenerator = rustPlatform.buildRustPackage rec {
        pname = "nixb-sys-generator";
        version = "0.1.0";
        src = sysBindingsGeneratorSrc;
        cargoLock.lockFile = ./bindings_generator_Cargo.lock;
        doCheck = false;
        meta.mainProgram = pname;
      };

      nixSources = {
        "2_32" = pkgs.fetchFromGitHub {
          owner = "NixOS";
          repo = "nix";
          rev = "2.32.6";
          sha256 = "1rp59n297mh6h5sbqbz5kg43j3arph5dph7yx0iwy1jzkb3gg8g5";
        };
        "2_33" = pkgs.fetchFromGitHub {
          owner = "NixOS";
          repo = "nix";
          rev = "2.33.3";
          sha256 = "0jm8wdg6iprhpja35v80cwi17nwdbklf71caq75y7d2rxzhimj6q";
        };
        "2_34" = pkgs.fetchFromGitHub {
          owner = "NixOS";
          repo = "nix";
          rev = "2.34.2";
          sha256 = "0i235h58b6sncd9p7sd1f4npmccwn4jznw2rx2kflz1fq62ahqbz";
        };
      };

      mkSysBindingsFor =
        moduleName: nixSource:
        pkgs.runCommand "nix-sys-bindings-${nixSource.rev}"
          {
            # The generator needs `rustfmt`, so add the Rust toolchain.
            nativeBuildInputs = [ (rust.mkToolchain pkgs) ];
            env = {
              LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
              NIX_SOURCE = nixSource;
              NIX_VERSION = nixSource.rev;
            };
          }
          ''
            mkdir -p "$out"
            OUTPUT_FILE="$out/${moduleName}.rs" ${lib.getExe sysBindingsGenerator}
          '';
    in
    {
      packages = {
        sys-bindings =
          let
            generatedBindings = lib.mapAttrsToList mkSysBindingsFor nixSources;
          in
          pkgs.runCommand "sys-bindings" { } ''
            mkdir -p "$out"
            ${lib.concatMapStringsSep "\n" (bindings: "cp -R ${bindings}/. \"$out/\"") generatedBindings}
          '';
      };

      apps.update-sys-bindings = {
        type = "app";
        program = lib.getExe (
          pkgs.writeShellApplication {
            name = "update-sys-bindings";
            runtimeInputs = [
              pkgs.coreutils
              pkgs.git
            ];
            text = ''
              repo_root="$(git rev-parse --show-toplevel)"
              bindings_dir="$repo_root/crates/nixb-sys/src/bindings"

              for path in "$bindings_dir"/*; do
                if [ "$(basename "$path")" != "mod.rs" ]; then
                  rm -rf "$path"
                fi
              done

              cp ${self'.packages.sys-bindings}/*.rs "$bindings_dir/"
            '';
          }
        );
      };

      apps.update-bindings-generator-lockfile = {
        type = "app";
        program = lib.getExe (
          pkgs.writeShellApplication {
            name = "update-bindings-generator-lockfile";
            runtimeInputs = [
              pkgs.coreutils
              pkgs.git
              pkgs.rust-script
              (rust.mkToolchain pkgs)
            ];
            text = ''
              repo_root="$(git rev-parse --show-toplevel)"
              tmpdir="$(mktemp -d)"
              trap 'rm -rf "$tmpdir"' EXIT

              rust-script --package --pkg-path "$tmpdir/pkg" "$repo_root/nix/bindings_generator.rs" > /dev/null
              cargo generate-lockfile --manifest-path "$tmpdir/pkg/Cargo.toml"
              cp "$tmpdir/pkg/Cargo.lock" "$repo_root/nix/bindings_generator_Cargo.lock"
            '';
          }
        );
      };
    };
}
