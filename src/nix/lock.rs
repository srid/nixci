use std::process::{Command, Stdio};

use anyhow::{bail, Result};

use crate::nix::util::print_shell_command;

pub fn nix_flake_lock_check(url: &str) -> Result<()> {
    print_shell_command(
        "nix",
        ["flake", "lock", "--no-update-lock-file", url].into_iter(),
    );
    let status = Command::new("nix")
        .args(["flake", "lock", "--no-update-lock-file"])
        .arg(url)
        .stdin(Stdio::null())
        .spawn()?
        .wait()?;
    if status.success() {
        Ok(())
    } else {
        let exit_code = status.code().unwrap_or(1);
        bail!("nix eval failed to run (exited: {})", exit_code);
    }
}
