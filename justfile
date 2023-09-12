default:
    @just --list 

# Auto-format the source tree
fmt:
    treefmt

alias f := fmt

# Run the project locally
watch *ARGS:
    cargo watch -s "cargo run -- {{ARGS}}"

alias w := watch

# Run tests
test:
    cargo watch -s "cargo test -F integration_test"

# Run docs server (live reloading)
doc:
    cargo-doc-live
