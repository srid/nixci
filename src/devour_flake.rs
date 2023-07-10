use crate::app::AppResult;
/// Rust support for invoking https://github.com/srid/devour-flake
use std::{process::{Command, Stdio}};

/// Absolute path to the devour-flake executable
///
/// We expect this environment to be set in Nix build and shell.
const DEVOUR_FLAKE: &str = env!("DEVOUR_FLAKE");

pub type DrvOut = String;

pub fn devour_flake(url: String) -> AppResult<Vec<DrvOut>> {
    // TODO: Strip devour-flake's follows-logging output
    let output = Command::new(DEVOUR_FLAKE)
        .arg(url)
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    if output.status.success() {
        parse_devour_flake_output(output.stdout)
    } else {
        println!("ERROR: devour-flake failed");
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

/// Parse stdout of `devour-flake` executable
///
/// It spits out drv outs built separated by whitespace.
fn parse_devour_flake_output(stdout: Vec<u8>) -> AppResult<Vec<DrvOut>> {
    let raw_output = String::from_utf8(stdout)?;
    let outs = raw_output.split_ascii_whitespace();
    if outs.clone().count() == 0 {
        println!("ERROR: No outputs produced by devour-flake");
        std::process::exit(1);
    } else {
        Ok(outs.map(|s| s.to_string()).collect())
    }
}
