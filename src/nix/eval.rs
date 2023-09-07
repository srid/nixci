use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use super::url::FlakeUrl;

/// Run 'nix eval <url> --json` and parse its JSON
///
/// If the flake does not output the given attribute, use the `Default`
/// implementation of `T`.
pub fn nix_eval_attr_json<T>(url: FlakeUrl) -> Result<T>
where
    T: Default + serde::de::DeserializeOwned,
{
    let output = Command::new("nix")
        .arg("--extra-experimental-features")
        .arg("nix-command flakes")
        .arg("eval")
        .arg(url.0)
        .arg("--json")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()
        .context("No 'nixci' config?")?;
    if output.status.success() {
        let raw_output = String::from_utf8(output.stdout)
            .context("Failed to decode 'nix eval' stdout as UTF-8")?;
        Ok(serde_json::from_str::<T>(&raw_output)?)
    } else {
        let raw_output = String::from_utf8(output.stderr)
            .context("Failed to decode 'nix eval' stderr as UTF-8")?;
        if raw_output.contains("does not provide attribute") {
            // The 'nixci' flake output attr is missing. User wants the default config.
            Ok(T::default())
        } else {
            let exit_code = output.status.code().unwrap_or(1);
            bail!(
                "nix eval failed to run (exited: {}). raw_output = {}",
                exit_code,
                raw_output
            );
        }
    }
}
