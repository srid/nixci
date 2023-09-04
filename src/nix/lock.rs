use std::process::{Command, Stdio};

use anyhow::{bail, Result};

use crate::nix::util::print_shell_command;

use super::url::FlakeUrl;

pub fn nix_flake_lock_check(url: &FlakeUrl) -> Result<()> {
    let args = ["flake", "lock", "--no-update-lock-file", &url.0];
    print_shell_command("nix", args.into_iter());
    let status = Command::new("nix")
        .args(args)
        .stdin(Stdio::null())
        .spawn()?
        .wait()?;
    if status.success() {
        Ok(())
    } else {
        let exit_code = status.code().unwrap_or(1);
        bail!("nix flake lock failed to run (exited: {})", exit_code);
    }
}
