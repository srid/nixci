use std::{
    fmt::{self, Display},
    path::PathBuf,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::Command;
/// Nix derivation output path
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub struct DrvOut(pub PathBuf);

impl DrvOut {
    pub fn as_store_path(self) -> StorePath {
        StorePath::Other(self.0)
    }
}

/// Represents a path in the Nix store, see: <https://zero-to-nix.com/concepts/nix-store#store-paths>
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub enum StorePath {
    /// Derivation path (ends with `.drv`).
    Drv(PathBuf),
    /// Other paths in the Nix store, such as build outputs.
    /// This won't be a derivation path.
    Other(PathBuf),
}

impl StorePath {
    pub fn new(path: PathBuf) -> Self {
        if path.ends_with(".drv") {
            StorePath::Drv(path)
        } else {
            StorePath::Other(path)
        }
    }

    pub fn as_path(&self) -> &PathBuf {
        match self {
            StorePath::Drv(p) => p,
            StorePath::Other(p) => p,
        }
    }
}

impl fmt::Display for StorePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_path().display())
    }
}

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

impl NixStoreCmd {
    /// Fetch all build and runtime dependencies of given [devour_flake::DrvOut]s
    ///
    /// This is done by querying the deriver of each output path from [devour_flake::DrvOut] using [nix_store_query_deriver] and
    /// then querying all dependencies of each deriver using [nix_store_query_requisites_with_outputs].
    /// Finally, all dependencies of each deriver are collected and returned as [Vec<StorePath>].
    pub async fn fetch_all_deps(
        &self,
        out_paths: Vec<DrvOut>,
    ) -> Result<Vec<StorePath>, NixCmdError> {
        let mut all_drvs = Vec::new();
        for out in out_paths.iter() {
            let DrvOut(out_path) = out;
            let drv = self.nix_store_query_deriver(out_path.clone()).await?;
            all_drvs.push(drv);
        }
        let mut all_outs = Vec::new();
        for drv in all_drvs {
            let deps = self
                .nix_store_query_requisites_with_outputs(drv.clone())
                .await?;
            all_outs.extend(deps.into_iter());
        }
        Ok(all_outs)
    }

    /// Return the derivation used to build the given build output.
    async fn nix_store_query_deriver(&self, out_path: PathBuf) -> Result<DrvOut, NixCmdError> {
        let mut cmd = self.command();
        cmd.args(["--query", "--deriver", &out_path.to_string_lossy().as_ref()]);
        nix_rs::command::trace_cmd(&cmd);
        let out = cmd
            .output()
            .await
            .map_err(CommandError::ChildProcessError)?;
        if out.status.success() {
            let drv_path = String::from_utf8(out.stdout)?.trim().to_string();
            if drv_path == "unknown-deriver" {
                return Err(NixCmdError::UnknownDeriverError);
            }
            Ok(DrvOut(PathBuf::from(drv_path)))
        } else {
            let stderr = Some(String::from_utf8_lossy(&out.stderr).to_string());
            let exit_code = out.status.code();
            Err(CommandError::ProcessFailed { stderr, exit_code }.into())
        }
    }

    /// Given a [StorePath::Drv], this function recursively queries and return all
    /// of its dependencies in the Nix store.
    async fn nix_store_query_requisites_with_outputs(
        &self,
        drv_path: DrvOut,
    ) -> Result<Vec<StorePath>, NixCmdError> {
        let mut cmd = self.command();
        cmd.args([
            "--query",
            "--requisites",
            "--include-outputs",
            &drv_path.0.to_string_lossy().as_ref(),
        ]);
        nix_rs::command::trace_cmd(&cmd);
        let out = cmd
            .output()
            .await
            .map_err(CommandError::ChildProcessError)?;
        if out.status.success() {
            let out = String::from_utf8(out.stdout)?;
            let out = out
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .map(PathBuf::from)
                .map(StorePath::new)
                .collect();
            Ok(out)
        } else {
            let stderr = Some(String::from_utf8_lossy(&out.stderr).to_string());
            let exit_code = out.status.code();
            Err(CommandError::ProcessFailed { stderr, exit_code }.into())
        }
    }
}

/// Errors when running and interpreting the output of a nix command
#[derive(Error, Debug)]
pub enum NixCmdError {
    #[error("Command error: {0}")]
    CmdError(#[from] CommandError),

    #[error("Failed to decode command stdout (utf8 error): {0}")]
    DecodeErrorUtf8(#[from] std::string::FromUtf8Error),

    #[error("Failed to decode command stdout (from_str error): {0}")]
    DecodeErrorFromStr(#[from] FromStrError),

    #[error("Failed to decode command stdout (json error): {0}")]
    DecodeErrorJson(#[from] serde_json::Error),

    #[error("Unknown deriver in drv_path")]
    UnknownDeriverError,
}

#[derive(Debug)]
pub struct FromStrError(String);

impl Display for FromStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse string: {}", self.0)
    }
}

impl std::error::Error for FromStrError {}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Child process error: {0}")]
    ChildProcessError(#[from] std::io::Error),
    #[error(
        "Process exited unsuccessfully. exit_code={:?}{}",
        exit_code,
        match stderr {
            Some(s) => format!(" stderr={}", s),
            None => "".to_string()
        },
    )]
    ProcessFailed {
        stderr: Option<String>,
        exit_code: Option<i32>,
    },
    #[error("Failed to decode command stderr: {0}")]
    Decode(#[from] std::string::FromUtf8Error),
}
