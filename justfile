default:
    @just --list

# Auto-format the source tree
fmt:
    treefmt

alias f := fmt

# Run the project locally (eg: `j w build ~/code/yourproject`)
w *ARGS:
    cargo watch -s "cargo run -- {{ARGS}}"

# Run tests
test:
    cargo watch -s "cargo test -F integration_test"

# Run docs server (live reloading)
doc:
    cargo-doc-live
