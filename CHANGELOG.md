
# Unreleased

- Pass the rest of CLI arguments after `--` as-is to `nix build`
    - Consequently, remove `--rebuild`, `--no-refresh` and `--system` options, because these can be specified using the new CLI spec.
- Iterate configs in a deterministic order

# 0.1.3

- Pass `-j auto` to nix builds.

# 0.1.2

Initial release
