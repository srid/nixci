use anyhow::{bail, Context, Result};
use colored::Colorize;
/// Rust support for invoking https://github.com/srid/devour-flake
use std::process::Stdio;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

/// Absolute path to the devour-flake executable
///
/// We expect this environment to be set in Nix build and shell.
const DEVOUR_FLAKE: &str = env!("DEVOUR_FLAKE");

/// Nix derivation output path
pub struct DrvOut(pub String);

#[tokio::main]
pub async fn devour_flake(verbose: bool, args: Vec<String>) -> Result<Vec<DrvOut>> {
    // TODO: Strip devour-flake's "follows" output from stderr
    let mut output_fut = Command::new(DEVOUR_FLAKE)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stderr_handle = output_fut.stderr.take().unwrap();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr_handle).lines();
        while let Some(line) = reader.next_line().await.expect("read stderr") {
            if !verbose {
                if line.starts_with("• Added input") {
                    // Consume the input logging itself
                    reader.next_line().await.expect("read stderr");
                    continue;
                } else if line.starts_with("warning: not writing modified lock file of flake") {
                    continue;
                }
            }
            println!("    {}", line.cyan());
        }
    });
    let output = output_fut
        .wait_with_output()
        .await
        .with_context(|| "Unable to spawn devour-flake process")?;
    if output.status.success() {
        parse_devour_flake_output(output.stdout.clone())
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
        Ok(outs.map(|s| DrvOut(s.to_string())).collect())
    }
}
