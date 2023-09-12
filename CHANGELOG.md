
# Unreleased

- Breaking changes
    - Change flake schema: evaluates `nixci.default` instead of `nixci`; this allows more than one configuration (#20)
- Pass the rest of CLI arguments after `--` as-is to `nix build`
    - Consequently, remove `--rebuild`, `--no-refresh` and `--system` options, because these can be specified using the new CLI spec.
- Bug fixes
    - Fix nixci breaking if branch name of a PR has `#` (#17)
- Misc changes
    - Iterate configs in a deterministic order
    - stdout outputs are uniquely printed, in sorted order
    - stderr output is now logged using the `tracing` crate.
    - Pass `--extra-experimental-features` to enable flakes
    - `nixci` can now be used as a Rust library
    - `nixci` no longer depends on `devour-flake` the *executable package*, only on the flake.

# 0.1.3

- Pass `-j auto` to nix builds.

# 0.1.2

Initial release
