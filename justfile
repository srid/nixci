# See flake.nix (just-flake)
import 'just-flake.just'

# Display the list of recipes
default:
    @just --list
    
# Run the project locally (eg: `j w build ~/code/yourproject`)
w *ARGS:
    cargo watch -s "cargo run -- {{ARGS}}"

# Run tests
test:
    cargo watch -s "cargo test -F integration_test"

# Run docs server (live reloading)
doc:
    cargo-doc-live
