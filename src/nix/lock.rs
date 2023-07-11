use colored::Colorize;
use std::process::Command;

use anyhow::{bail, Result};

pub fn nix_flake_lock_check(url: &String) -> Result<()> {
    println!(
        "{}",
        format!("🧷 checking flake.lock is up to date for {}", url)
            .blue()
            .bold()
    );
    let output = Command::new("nix")
        .args(["flake", "lock", "--no-update-lock-file"])
        .arg(url)
        .spawn()?
        .wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        let exit_code = output.status.code().unwrap_or(1);
        bail!("nix eval failed to run (exited: {})", exit_code);
    }
}
