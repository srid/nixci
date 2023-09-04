
# Unreleased

- Change flake schema: evaluates `nixci.default` instead of `nixci`; this allows more than one configuration (#20)
- Pass the rest of CLI arguments after `--` as-is to `nix build`
    - Consequently, remove `--rebuild`, `--no-refresh` and `--system` options, because these can be specified using the new CLI spec.
- Iterate configs in a deterministic order
- Bug fixes
    - Fix nixci breaking if branch name of a PR has `#` (#17)

# 0.1.3

- Pass `-j auto` to nix builds.

# 0.1.2

Initial release
