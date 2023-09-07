{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # Rust
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";

    # Dev tools
    treefmt-nix.url = "github:numtide/treefmt-nix";
    mission-control.url = "github:Platonic-Systems/mission-control";
    flake-root.url = "github:srid/flake-root";

    # App dependenciues
    devour-flake.url = "github:srid/devour-flake/master";
    devour-flake.flake = false;
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
        inputs.treefmt-nix.flakeModule
        inputs.mission-control.flakeModule
        inputs.flake-root.flakeModule
      ];

      perSystem = { config, self', pkgs, lib, system, ... }:
        let
          src = lib.cleanSourceWith {
            src = ./.; # The original, unfiltered source
            filter = path: type:
              # Default filter from crane (allow .rs files)
              (craneLib.filterCargoSources path type)
            ;
          };
          rustToolchain = (pkgs.rust-bin.fromRustupToolchainFile (./rust-toolchain.toml)).override {
            extensions = [
              "rust-src"
              "rust-analyzer"
              "clippy"
            ];
          };
          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;
          nativeBuildInputs = with pkgs; with pkgs.darwin.apple_sdk.frameworks; lib.optionals stdenv.isDarwin [
            Security
          ] ++ [
            libiconv
            openssl
            pkgconfig
          ];
          args = {
            inherit src nativeBuildInputs;
            DEVOUR_FLAKE = inputs.devour-flake;
          };
          rustDevShell = pkgs.mkShell {
            shellHook = ''
              # For rust-analyzer 'hover' tooltips to work.
              export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library";
            '';
            nativeBuildInputs = nativeBuildInputs ++ [
              rustToolchain
            ];
          };
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              inputs.rust-overlay.overlays.default
            ];
          };

          # Flake outputs
          packages.default = craneLib.buildPackage args;
          overlayAttrs.nixci = self'.packages.default;

          devShells.default = pkgs.mkShell {
            name = "nixci";
            inputsFrom = [
              rustDevShell
              config.treefmt.build.devShell
              config.mission-control.devShell
              config.flake-root.devShell
            ];
            shellHook = ''
              export DEVOUR_FLAKE=${inputs.devour-flake}
            '';
            nativeBuildInputs = [
              pkgs.cargo-watch
            ];
          };

          # Add your auto-formatters here.
          # cf. https://numtide.github.io/treefmt/
          treefmt.config = {
            projectRootFile = "flake.nix";
            programs = {
              nixpkgs-fmt.enable = true;
              rustfmt.enable = true;
            };
          };

          # Makefile'esque but in Nix. Add your dev scripts here.
          # cf. https://github.com/Platonic-Systems/mission-control
          mission-control.scripts = {
            fmt = {
              exec = config.treefmt.build.wrapper;
              description = "Auto-format project tree";
            };

            run = {
              exec = ''
                cargo run "$@"
              '';
              description = "Run the project executable";
            };

            watch = {
              exec = ''
                set -x
                cargo watch -x "run -- $*"
              '';
              description = "Watch for changes and run the project executable";
            };
          };
        };
    };
}
