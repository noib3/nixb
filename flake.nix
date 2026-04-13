{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      treefmt-nix,
      ...
    }:
    let
      forEachSystem =
        f:
        nixpkgs.lib.genAttrs [
          "aarch64-darwin"
          "aarch64-linux"
          "x86_64-linux"
        ] (system: f system nixpkgs.legacyPackages.${system});

      mkRustToolchain =
        pkgs:
        (rust-overlay.lib.mkRustBin { } pkgs).fromRustupToolchainFile
          ./rust-toolchain.toml;

      mkTreefmt =
        pkgs:
        treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";

          programs.nixfmt = {
            enable = true;
            width = 80;
          };

          programs.rustfmt = {
            enable = true;
            package = (pkgs.rustfmt.override { asNightly = true; });
          };

          settings.formatter.rustfmt.options = [
            "--config-path"
            "${./rustfmt.toml}"
          ];
        };
    in
    {
      packages = forEachSystem (
        system: pkgs: {
          sys-bindings = pkgs.callPackage ./nix/sys-bindings.nix {
            rust = mkRustToolchain pkgs;
            nixSources = import ./nix/nix-sources.nix {
              inherit (pkgs) fetchFromGitHub;
            };
          };
        }
      );

      apps = forEachSystem (
        system: pkgs:
        {
          update-bindings-generator-lockfile = {
            type = "app";
            program = nixpkgs.lib.getExe (
              pkgs.callPackage ./nix/update-sys-bindings-generator-lockfile.nix {
                rust = mkRustToolchain pkgs;
              }
            );
          };
          update-sys-bindings = {
            type = "app";
            program = nixpkgs.lib.getExe (
              pkgs.callPackage ./nix/update-sys-bindings.nix {
                sys-bindings = self.packages.${system}.sys-bindings;
              }
            );
          };
        }
        # Workaround for https://github.com/NixOS/nix/issues/8881 so that we
        # can run individual checks with `nix run .#check-<foo>`.
        // (nixpkgs.lib.mapAttrs' (name: check: {
          name = "check-${name}";
          value = {
            type = "app";
            program =
              (pkgs.writeShellScript "check-${name}" ''
                # Force evaluation of ${check}.
                echo -e "\033[1;32m✓\033[0m Check '${name}' passed"
              '').outPath;
          };
        }) self.checks.${system})
      );

      devShells = forEachSystem (
        system: pkgs: {
          default = pkgs.mkShell {
            buildInputs = [
              pkgs.nixVersions.nix_2_34.dev
            ];
            nativeBuildInputs = [
              pkgs.pkg-config
              ((mkRustToolchain pkgs).override {
                extensions = [
                  "clippy"
                  "rust-analyzer"
                  "rust-src"
                  "rustfmt"
                ];
              })
            ];
            env = {
              # This silences a warning emitted by the build script of the
              # nixb-cpp crate. See
              # https://github.com/NixOS/nixpkgs/issues/395191 and
              # https://github.com/NixOS/nixpkgs/pull/396373 for details.
              NIX_CC_WRAPPER_SUPPRESS_TARGET_WARNING = "1";
            };
          };
        }
      );

      formatter = forEachSystem (
        _system: pkgs: (mkTreefmt pkgs).config.build.wrapper
      );

      checks = forEachSystem (
        _system: pkgs: {
          formatting = (mkTreefmt pkgs).config.build.check self;
        }
      );
    };
}
