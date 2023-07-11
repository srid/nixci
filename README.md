# WIP: nixci

## Dev

In nix shell,

```
, watch
```

## What it does

- Accepts a flake config (`nixci`) that optionally specifies all the sub-flakes to build, along with their input overrides
- Runs [devour-flake](https://github.com/srid/devour-flake) to build all flake outputs
- Checks that `flake.lock` is in sync
- Prints the built outputs

## TODO

- [ ] Initial stablization
- [ ] Accept Github PR urls
- [ ] Sanitize entire console output (debug logging, etc.)