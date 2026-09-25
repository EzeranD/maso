{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };
  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    fenix,
    advisory-db,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib;

        rustToolchain = fenix.packages.${system}.latest.withComponents [
          "cargo"
          "clippy"
          "rust-analyzer"
          "rustc"
          "rustfmt"
          "rust-src"
        ];

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = lib.fileset.toSource {
          root = ./.;
          fileset = craneLib.fileset.commonCargoSources ./.;
        };

        commonArgs = {
          inherit src;
          pname = "maso";
          version = (lib.importTOML ./Cargo.toml).workspace.package.version;
          strictDeps = true;

          nativeBuildInputs = [
            pkgs.clang
            pkgs.libclang.lib
            pkgs.pkg-config
          ];

          buildInputs = [
            pkgs.ffmpeg-full
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.ffmpeg-full
          ];
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        maso = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
      in {
        checks = {
          inherit maso;

          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets --all-features -- --deny warnings";
            }
          );

          fmt = craneLib.cargoFmt commonArgs;

          toml-fmt = craneLib.taploFmt (
            commonArgs
            // {
              src = pkgs.lib.sources.sourceFilesBySuffices src [".toml"];
            }
          );

          audit = craneLib.cargoAudit (
            commonArgs
            // {
              inherit advisory-db;
            }
          );

          deny = craneLib.cargoDeny commonArgs;

          nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );
        };

        packages.default = maso;

        formatter = pkgs.alejandra;

        devShells.default = craneLib.devShell (
          commonArgs
          // {
            checks = self.checks.${system};

            version = null;
          }
        );
      }
    );
}
