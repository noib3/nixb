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

      mkNixPackages =
        pkgs:
        pkgs.callPackage ./nix/nix-packages.nix {
          inherit nixpkgs;
        };

      mkNixSources =
        pkgs: import ./nix/nix-sources.nix { inherit (pkgs) fetchFromGitHub; };

      mkShell =
        pkgs: nixPackage:
        pkgs.mkShell {
          packages = [
            (pkgs.lib.getBin nixPackage)
          ];
          buildInputs = [
            (pkgs.lib.getDev nixPackage)
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
    in
    {
      packages = forEachSystem (
        system: pkgs:
        let
          nixPackages = mkNixPackages pkgs;
          examples = pkgs.callPackage ./nix/examples.nix {
            inherit nixPackages;
            rust = mkRustToolchain pkgs;
          };
        in
        {
          inherit examples;
          check-examples = pkgs.callPackage ./nix/check-examples.nix {
            inherit nixPackages;
            examplesPackages = examples;
          };
          sys-bindings = pkgs.callPackage ./nix/sys-bindings.nix {
            rust = mkRustToolchain pkgs;
          };
        }
      );

      apps = forEachSystem (
        system: pkgs: {
          check-formatting = {
            type = "app";
            program = nixpkgs.lib.getExe (
              pkgs.writeShellApplication {
                name = "check-formatting";
                text = ''
                  ${nixpkgs.lib.getExe self.formatter.${system}} --fail-on-change
                '';
              }
            );
          };
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
      );

      devShells = forEachSystem (
        _system: pkgs:
        let
          nixPackages = mkNixPackages pkgs;
          nixSourceShells = pkgs.lib.mapAttrs' (nixSourceKey: _nixSource: {
            name = "nix-${builtins.replaceStrings [ "_" ] [ "-" ] nixSourceKey}";
            value = mkShell pkgs nixPackages.${nixSourceKey};
          }) (mkNixSources pkgs);
        in
        nixSourceShells
        // {
          default = nixSourceShells."nix-2-34";
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
