# nixci

> **Note**
>
> 🚧 Work in Progress

`nixci` builds all outputs in a flake (via [devour-flake]) which can in turn be used in either CI or locally.

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

[Here's a real-world example](https://github.com/juspay/services-flake/blob/197fc1c4d07d09f4e01dd935450608c35393b102/flake.nix#L10-L24).

## What it does

- Accepts a flake config (`nixci`) that optionally specifies all the sub-flakes to build, along with their input overrides
- Checks that `flake.lock` is in sync
- Runs [devour-flake](https://github.com/srid/devour-flake) to build all flake outputs
- Prints the built outputs

## TODO

- [x] Initial stablization
- [x] Accept Github PR urls
- [ ] Sanitize entire console output (debug logging, etc.)

[devour-flake]: https://github.com/srid/devour-flake