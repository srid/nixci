use std::fmt;

use anyhow::{bail, Result};

use super::devour_flake::DrvOut;

/// Nix derivation path
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub struct Drv(pub String);

/// Encompasses both derivation and derivation output paths
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub enum DrvPaths {
    DrvOut(DrvOut),
    Drv(Drv),
}

impl fmt::Display for DrvPaths {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DrvPaths::DrvOut(drv_out) => write!(f, "{}", drv_out.0),
            DrvPaths::Drv(drv) => write!(f, "{}", drv.0),
        }
    }
}

/// Query the deriver of an output path
pub async fn nix_store_query_deriver(out_path: DrvOut) -> Result<Drv> {
    let nix_store = crate::nix_store_cmd().await;
    let mut cmd = nix_store.command();
    cmd.args(["--query", "--deriver", &out_path.0]);
    nix_rs::command::trace_cmd(&cmd);
    let drv_out = cmd.output().await?;
    if drv_out.status.success() {
        let drv = String::from_utf8(drv_out.stdout)?.trim().to_string();
        Ok(Drv(drv))
    } else {
        let exit_code = drv_out.status.code().unwrap_or(1);
        bail!(
            "nix-store --query --deriver failed to run (exited: {})",
            exit_code
        );
    }
}

/// Query the requisites of a derivation path, including outputs
pub async fn nix_store_query_requisites_with_outputs(drv: Drv) -> Result<Vec<DrvPaths>> {
    let nix_store = crate::nix_store_cmd().await;
    let mut cmd = nix_store.command();
    cmd.args(["--query", "--requisites", "--include-outputs", &drv.0]);
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
                    DrvPaths::Drv(Drv(l))
                } else {
                    DrvPaths::DrvOut(DrvOut(l))
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
