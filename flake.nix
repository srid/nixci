{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # Rust
    rust-flake.url = "github:juspay/rust-flake";
    rust-flake.inputs.nixpkgs.follows = "nixpkgs";
    cargo-doc-live.url = "github:srid/cargo-doc-live";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    just-flake.url = "github:juspay/just-flake";
    pre-commit-hooks-nix = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixpkgs-stable.follows = "nixpkgs";
    };

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
        inputs.rust-flake.flakeModules.default
        inputs.rust-flake.flakeModules.nixpkgs
        inputs.cargo-doc-live.flakeModule
        inputs.process-compose-flake.flakeModule
        inputs.pre-commit-hooks-nix.flakeModule
        inputs.treefmt-nix.flakeModule
        inputs.just-flake.flakeModule
      ];

      perSystem = { config, self', pkgs, lib, system, ... }: {
        rust-project.crane.args = {
          nativeBuildInputs = with pkgs; with pkgs.darwin.apple_sdk.frameworks; lib.optionals stdenv.isDarwin [
            Security
            SystemConfiguration
          ] ++ [
            libiconv
            pkg-config
          ];
          buildInputs = lib.optionals pkgs.stdenv.isDarwin
            (
              with pkgs.darwin.apple_sdk.frameworks; [
                IOKit
                # apple_sdk refers to SDK version 10.12. To compile for `x86_64-darwin` we need 11.0
                # see: https://github.com/NixOS/nixpkgs/pull/261683#issuecomment-1772935802
                pkgs.darwin.apple_sdk_11_0.frameworks.CoreFoundation
              ]
            )
          ++ lib.optionals pkgs.stdenv.isLinux [
            pkgs.openssl
          ];
          DEVOUR_FLAKE = inputs.devour-flake;
        };

        pre-commit = {
          check.enable = true;
          settings = {
            hooks = {
              treefmt.enable = true;
              convco.enable = true;
            };
          };
        };

        just-flake.features = {
          treefmt.enable = true;
          convco.enable = true;
        };

        # Flake outputs
        packages.default = self'.packages.nixci;
        overlayAttrs.nixci = self'.packages.default;

        devShells.default = pkgs.mkShell {
          name = "nixci";
          inputsFrom = [
            self'.devShells.nixci
            config.treefmt.build.devShell
            config.just-flake.outputs.devShell
          ];
          shellHook = ''
            export DEVOUR_FLAKE=${inputs.devour-flake}
          '';
          packages = [
            pkgs.cargo-watch
            config.process-compose.cargo-doc-live.outputs.package
            config.pre-commit.settings.tools.convco
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
