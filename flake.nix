{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # Rust
    dream2nix.url = "github:nix-community/dream2nix";

    # Dev tools
    treefmt-nix.url = "github:numtide/treefmt-nix";
    mission-control.url = "github:Platonic-Systems/mission-control";
    flake-root.url = "github:srid/flake-root";

    # App dependenciues
    devour-flake.url = "github:srid/devour-flake";
    devour-flake.flake = false;
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
        inputs.dream2nix.flakeModuleBeta
        inputs.treefmt-nix.flakeModule
        inputs.mission-control.flakeModule
        inputs.flake-root.flakeModule
      ];

      perSystem = { config, self', pkgs, lib, system, ... }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            (self: super: {
              devour-flake = self.callPackage inputs.devour-flake { };
            })
          ];
        };

        # Rust project definition
        # cf. https://github.com/nix-community/dream2nix
        dream2nix.inputs."nixci" = {
          source = lib.sourceFilesBySuffices ./. [
            ".rs"
            "Cargo.toml"
            "Cargo.lock"
          ];
          projects."nixci" = { name, ... }: {
            inherit name;
            subsystem = "rust";
            translator = "cargo-lock";
          };
          packageOverrides =
            let
              common = {
                add-deps = with pkgs; with pkgs.darwin.apple_sdk.frameworks; {
                  nativeBuildInputs = old: old ++ lib.optionals stdenv.isDarwin [
                    libiconv
                    Security
                  ];
                };
              };
            in
            {
              # Project and dependency overrides:
              nixci = common // { };
              nixci-deps = common;
            };
        };

        # Flake outputs
        packages.default =
          let nixci = config.dream2nix.outputs.nixci.packages.nixci;
          in nixci.overrideAttrs (old: {
            DEVOUR_FLAKE = lib.getExe pkgs.devour-flake;
          });
        overlayAttrs.nixci = self'.packages.default;

        devShells.default = pkgs.mkShell {
          inputsFrom = [
            config.dream2nix.outputs.nixci.devShells.default
            config.treefmt.build.devShell
            config.mission-control.devShell
            config.flake-root.devShell
          ];
          shellHook = ''
            # For rust-analyzer 'hover' tooltips to work.
            export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
            export DEVOUR_FLAKE=${lib.getExe pkgs.devour-flake}
          '';
          nativeBuildInputs = [
            pkgs.cargo-watch
            pkgs.rust-analyzer
            pkgs.devour-flake
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
              url="''${1:-github:juspay/services-flake}"
              cargo watch -x "run -- --rebuild $url"
            '';
            description = "Watch for changes and run the project executable";
          };
        };
      };
    };
}
