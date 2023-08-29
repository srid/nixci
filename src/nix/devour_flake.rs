//! Rust support for invoking https://github.com/srid/devour-flake

use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::{
    io::{stdout, BufWriter, Write},
    process::Stdio,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use super::util::print_shell_command;

/// Absolute path to the devour-flake executable
///
/// We expect this environment to be set in Nix build and shell.
pub const DEVOUR_FLAKE: &str = env!("DEVOUR_FLAKE");

#[tokio::main]
pub async fn devour_flake(verbose: bool, args: Vec<String>) -> Result<()> {
    print_shell_command(DEVOUR_FLAKE, args.iter().map(|s| &**s));
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
            eprintln!("{}", line);
        }
    });
    let output = output_fut
        .wait_with_output()
        .await
        .context("Unable to spawn devour-flake process")?;
    if output.status.success() {
        print_devour_flake_output(output.stdout)
    } else {
        let exit_code = output.status.code().unwrap_or(1);
        bail!("devour-flake failed to run (exited: {})", exit_code);
    }
}

/// Parse stdout of `devour-flake` executable
///
/// It spits out drv outs built separated by whitespace.
fn print_devour_flake_output(raw_output: Vec<u8>) -> Result<()> {
    if raw_output.is_empty() {
        eprintln!("Warn: devour-flake produced no outputs");
        return Ok(());
    }

    let raw_output =
        String::from_utf8(raw_output).context("Failed to decode devour-flake output as UTF-8")?;

    let mut stdout = BufWriter::new(stdout().lock());
    for out in raw_output.split_ascii_whitespace() {
        writeln!(stdout, "{}", out.bold())?;
    }

    stdout.flush()?;
    Ok(())
}
