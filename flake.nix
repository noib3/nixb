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

          settings.global.excludes = [
            "crates/nixb-sys/src/bindings/*.rs"
          ];

          programs.nixfmt = {
            enable = true;
            width = 80;
          };

          programs.rustfmt = {
            enable = true;
            package = pkgs.rustfmt.override { asNightly = true; };
          };

          settings.formatter.rustfmt.options = [
            "--config-path"
            "${./rustfmt.toml}"
          ];
        };
    in
    {
      packages = forEachSystem (
        system: pkgs:
        let
          nixPackages = pkgs.callPackage ./nix/nix-packages.nix {
            inherit nixpkgs;
          };
        in
        {
          sys-bindings = pkgs.callPackage ./nix/sys-bindings.nix {
            rust = mkRustToolchain pkgs;
          };
          examples = pkgs.callPackage ./nix/examples.nix {
            inherit nixPackages;
            rust = mkRustToolchain pkgs;
          };
          check-examples = pkgs.callPackage ./nix/check-examples.nix {
            inherit nixPackages;
            examplesPackages = self.packages.${system}.examples;
          };
        }
      );

      apps = forEachSystem (
        system: pkgs: {
          update-bindings-generator-lockfile = {
            type = "app";
            program = nixpkgs.lib.getExe (
              pkgs.callPackage ./nix/update-sys-bindings-generator-lockfile.nix {
                rust = mkRustToolchain pkgs;
              }
            );
          };
          check-examples = nixpkgs.lib.mapAttrs (_name: check: {
            type = "app";
            program = nixpkgs.lib.getExe check;
          }) self.packages.${system}.check-examples;
          update-sys-bindings = {
            type = "app";
            program = nixpkgs.lib.getExe (
              pkgs.callPackage ./nix/update-sys-bindings.nix {
                sys-bindings = self.packages.${system}.sys-bindings;
              }
            );
          };
        }
      );

      devShells = forEachSystem (
        system: pkgs: {
          default = pkgs.mkShell {
            buildInputs = [
              pkgs.nixVersions.nix_2_34.dev
            ];
            nativeBuildInputs = [
              pkgs.pkg-config
              (pkgs.rustfmt.override { asNightly = true; })
              ((mkRustToolchain pkgs).override {
                extensions = [
                  "clippy"
                  "rust-analyzer"
                  "rust-src"
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
        system: pkgs: {
          formatting = (mkTreefmt pkgs).config.build.check self;
          examples = self.packages.${system}.check-examples;
        }
      );
    };
}
