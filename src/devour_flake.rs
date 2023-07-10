/// Rust support for invoking https://github.com/srid/devour-flake
use std::process::{Command, Stdio, Output};
use crate::app::{AppResult};

/// Absolute path to the devour-flake executable
///
/// We expect this environment to be set in Nix build and shell.
const DEVOUR_FLAKE: &str = env!("DEVOUR_FLAKE");

pub fn devour_flake(url: String) -> AppResult<Output> {
    let out = Command::new(DEVOUR_FLAKE)
        .arg(url)
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    Ok(out)
}