/// Enough types to get branch info from Pull Request URL
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

pub(crate) async fn dump_github_actions_matrix(
    cfg: &Config,
    systems: Vec<System>,
) -> anyhow::Result<()> {
    let include: Vec<GitHubMatrixRow> = systems
        .iter() // Assuming systems can be iterated over by reference.
        .flat_map(|system| {
            cfg.subflakes.0.iter().filter_map(|(k, v)| {
                if v.can_build_on(&[system.clone()]) {
                    // Assuming this doesn't need to be a Vec anymore.
                    Some(GitHubMatrixRow {
                        system: system.clone(), // Only clone system here if necessary.
                        subflake: k.clone(),    // Assuming k needs to be owned here.
                    })
                } else {
                    None
                }
            })
        })
        .collect();
    let matrix = GitHubMatrix { include };
    println!("{}", serde_json::to_string(&matrix)?);
    Ok(())
}
