
# Unreleased

- New features
    - Add new config `nixci.*.*.systems` acting as a whitelist of systems to build that subflake.
    - Add `--build-systems` option to build on an arbitrary systems (\#39)
    - Allow selecting sub-flake to build, e.g.: `nixci .#default.myflake`  (\#45)
- Fixes
    - Fix regression in Nix 2.19+ (`devour-flake produced an outpath with no outputs`) (\#35)
    - Evaluate OS configurations for current system only (\#38)
    - Fail correctly if nixci is passed a missing flake attribute (\#44)

# 0.2.0

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
