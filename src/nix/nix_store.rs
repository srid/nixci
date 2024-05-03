use std::{fmt, path::PathBuf};

use anyhow::{bail, Result};

/// Encompasses both derivation and derivation output paths
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub enum StorePath {
    BuildOutput(PathBuf),
    Drv(PathBuf),
}

impl fmt::Display for StorePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorePath::BuildOutput(out_path) => write!(f, "{}", out_path.display()),
            StorePath::Drv(drv_path) => write!(f, "{}", drv_path.display()),
        }
    }
}

/// Query the deriver of an output path
pub async fn nix_store_query_deriver(out_path: PathBuf) -> Result<PathBuf> {
    let nix_store = crate::nix_store_cmd().await;
    let mut cmd = nix_store.command();
    cmd.args([
        "--query",
        "--deriver",
        &out_path.to_string_lossy().to_string(),
    ]);
    nix_rs::command::trace_cmd(&cmd);
    let out = cmd.output().await?;
    if out.status.success() {
        let drv_path = String::from_utf8(out.stdout)?.trim().to_string();
        Ok(PathBuf::from(drv_path))
    } else {
        let exit_code = out.status.code().unwrap_or(1);
        bail!(
            "nix-store --query --deriver failed to run (exited: {})",
            exit_code
        );
    }
}

/// Query the requisites of a derivation path, including outputs
pub async fn nix_store_query_requisites_with_outputs(drv_path: PathBuf) -> Result<Vec<StorePath>> {
    let nix_store = crate::nix_store_cmd().await;
    let mut cmd = nix_store.command();
    cmd.args([
        "--query",
        "--requisites",
        "--include-outputs",
        &drv_path.to_string_lossy().to_string(),
    ]);
    nix_rs::command::trace_cmd(&cmd);
    let out = cmd.output().await?;
    if out.status.success() {
        let out = String::from_utf8(out.stdout)?;
        let out = out
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .map(|l| {
                if l.ends_with(".drv") {
                    StorePath::Drv(PathBuf::from(l))
                } else {
                    StorePath::BuildOutput(PathBuf::from(l))
                }
            })
            .collect();
        Ok(out)
    } else {
        let exit_code = out.status.code().unwrap_or(1);
        bail!(
            "nix-store --query --requisites --include-outputs failed to run (exited: {})",
            exit_code
        );
    }
}
