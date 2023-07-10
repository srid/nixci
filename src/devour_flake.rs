use anyhow::{bail, Context, Result};
/// Rust support for invoking https://github.com/srid/devour-flake
use std::process::{Command, Stdio};

/// Absolute path to the devour-flake executable
///
/// We expect this environment to be set in Nix build and shell.
const DEVOUR_FLAKE: &str = env!("DEVOUR_FLAKE");

pub type DrvOut = String;

pub fn devour_flake(url: String) -> Result<Vec<DrvOut>> {
    // TODO: Strip devour-flake's follows-logging output from stderr
    //
    // What is the best way to achieve this?
    let output = Command::new(DEVOUR_FLAKE)
        .arg(url)
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()
        .with_context(|| "Unable to spawn devour-flake process")?;
    if output.status.success() {
        parse_devour_flake_output(output.stdout)
    } else {
        let exit_code = output.status.code().unwrap_or(1);
        bail!("devour-flake failed to run (exited: {})", exit_code);
    }
}

/// Parse stdout of `devour-flake` executable
///
/// It spits out drv outs built separated by whitespace.
fn parse_devour_flake_output(stdout: Vec<u8>) -> Result<Vec<DrvOut>> {
    let raw_output = String::from_utf8(stdout)
        .with_context(|| format!("Failed to decode devour-flake output as UTF-8"))?;
    let outs = raw_output.split_ascii_whitespace();
    if outs.clone().count() == 0 {
        bail!("devour-flake produced no outputs (the flake has none?)");
    } else {
        Ok(outs.map(|s| s.to_string()).collect())
    }
}
