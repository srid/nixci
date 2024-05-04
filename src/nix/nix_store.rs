use std::{fmt, path::PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::devour_flake;

/// The `nix-store` command
/// See documentation for [nix-store](https://nixos.org/manual/nix/stable/command-ref/nix-store.html)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct NixStoreCmd;

impl NixStoreCmd {
    pub fn command(&self) -> Command {
        let mut cmd = Command::new("nix-store");
        cmd.kill_on_drop(true);
        cmd
    }
}

/// Represents a path in the Nix store, see: <https://zero-to-nix.com/concepts/nix-store#store-paths>
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub enum StorePath {
    /// Output path, which is the result of building a [StorePath::Drv]
    BuildOutput(PathBuf),
    /// Derivation path, which is a recipe for producing reproducible [StorePath::BuildOutput].
    /// Usually, a derivation path ends with `.drv`.
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

/// Given a [StorePath::BuildOutput], this function queries and returns its [StorePath::Drv].
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

/// Given a [StorePath::Drv], this function recursively queries all its [StorePath::Drv]
/// and [StorePath::BuildOutput] dependencies.
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

/// Fetch all build and runtime dependencies of given [devour_flake::BuildOutput]s
///
/// This is done by querying the deriver of each output path from [devour_flake::BuildOutput] using [nix_store_query_deriver] and
/// then querying all dependencies of each deriver using [nix_store_query_requisites_with_outputs].
/// Finally, all dependencies of each deriver are collected and returned as [Vec<StorePath>].
pub async fn fetch_all_deps(out_paths: Vec<devour_flake::BuildOutput>) -> Result<Vec<StorePath>> {
    let mut all_drvs = Vec::new();
    for out in out_paths.iter() {
        let devour_flake::BuildOutput(out_path) = out;
        let drv = nix_store_query_deriver(out_path.clone()).await?;
        all_drvs.push(drv);
    }
    let mut all_outs = Vec::new();
    for drv in all_drvs {
        let deps = nix_store_query_requisites_with_outputs(drv.clone()).await?;
        all_outs.extend(deps.into_iter());
    }
    Ok(all_outs)
}
