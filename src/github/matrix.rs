/// Enough types to get branch info from Pull Request URL
use itertools::iproduct;
use nix_rs::flake::system::System;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubMatrixRow {
    pub system: System,
    pub subflake: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubMatrix {
    pub include: Vec<GitHubMatrixRow>,
}

impl GitHubMatrix {
    pub fn new(systems: Vec<System>, subflakes: Vec<String>) -> Self {
        let include = iproduct!(systems, subflakes)
            .map(|(system, subflake)| GitHubMatrixRow { system, subflake })
            .collect();
        GitHubMatrix { include }
    }
}

pub(crate) async fn dump_github_actions_matrix(
    cfg: &Config,
    systems: Vec<System>,
) -> anyhow::Result<()> {
    // TODO: Should take into account systems whitelist
    // Ref: https://github.com/srid/nixci/blob/efc77c8794e5972617874edd96afa8dd4f1a75b2/src/config.rs#L104-L105
    let matrix = GitHubMatrix::new(systems, cfg.subflakes.0.keys().cloned().collect());
    println!("{}", serde_json::to_string(&matrix)?);
    Ok(())
}
