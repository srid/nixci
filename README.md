# nixci

`nixci` builds all outputs in a flake, or optionally its [sub-flakes](https://github.com/hercules-ci/flake-parts/issues/119), which can in turn be used either in CI or locally. Using [devour-flake] it will automatically build the following outputs:

| Type | Output Key |
| -- | -- |
| Standard flake outputs | `packages`, `apps`, `checks`, `devShells` |
| NixOS | `nixosConfigurations.*` |
| nix-darwin | `darwinConfigurations.*` |
| home-manager | `legacyPackages.${system}.homeConfigurations.*` |

## Usage

Use `nix run github:srid/nixci` to run `nixci` directly off this repo. Or install it using `nix profile install github:srid/nixci`.

```sh
# Run nixci on a local flake (default is $PWD)
$ nixci ~/code/myproject

# Run nixci on a github repo
$ nixci github:hercules-ci/hercules-ci-agent

# Run nixci on a github PR
$ nixci https://github.com/srid/emanote/pull/451
```

## Configuring

By default, `nixci` will build the top-level flake, but you can tell it to build sub-flakes by adding the following output to your top-level flake:

```nix
# myproject/flake.nix
{
  nixci = {
    dir1 = {
        dir = "dir1";
    };
    dir2 = {
        dir = "dir2";
        overrideInputs.myproject = ./.;
    };
  }
}
```

### Examples

Some real-world examples of how nixci is used with specific configurations:

- [services-flake](https://github.com/juspay/services-flake/blob/197fc1c4d07d09f4e01dd935450608c35393b102/flake.nix#L10-L24)
- [nixos-flake](https://github.com/srid/nixos-flake/blob/4af32875e7cc6df440c5f5cf93c67af41902768b/flake.nix#L29-L45)

## What it does

- Accepts a flake config (`nixci`) that optionally specifies all the sub-flakes to build, along with their input overrides
- Checks that `flake.lock` is in sync
- Runs [devour-flake](https://github.com/srid/devour-flake) to build all flake outputs
- Prints the built outputs

## TODO

- [x] Initial stablization
- [x] Accept Github PR urls
- [ ] Normalize entire console output in some aggreable fashion

[devour-flake]: https://github.com/srid/devour-flake

## See also

- [jenkins-nix-ci](https://github.com/juspay/jenkins-nix-ci) - Jenkins NixOS module that supports `nixci` as a Groovy function
