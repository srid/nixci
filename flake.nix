{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # Rust
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    cargo-doc-live.url = "github:srid/cargo-doc-live";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";

    # Dev tools
    treefmt-nix.url = "github:numtide/treefmt-nix";

    # App dependenciues
    devour-flake.url = "github:srid/devour-flake";
    devour-flake.flake = false;
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
        inputs.cargo-doc-live.flakeModule
        inputs.process-compose-flake.flakeModule
        inputs.treefmt-nix.flakeModule
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
            SystemConfiguration
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
            ];
            shellHook = ''

              export DEVOUR_FLAKE=${inputs.devour-flake}
            '';
            nativeBuildInputs = [
              pkgs.just
              pkgs.cargo-watch
              config.process-compose.cargo-doc-live.outputs.package
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
        };
    };
}
