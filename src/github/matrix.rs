/// Enough types to get branch info from Pull Request URL
use itertools::iproduct;
use nix_rs::flake::system::System;
use serde::{Deserialize, Serialize};

use crate::config::{Config, SubFlakish};

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
    let include = iproduct!(
        systems,
        cfg.subflakes
            .0
            .iter()
            .collect::<Vec<(&String, &SubFlakish)>>()
    )
    .flat_map(|(system, (k, v))| {
        if v.can_build_on(&vec![system.clone()]) {
            Some(GitHubMatrixRow {
                system,
                subflake: k.to_string(),
            })
        } else {
            None
        }
    })
    .collect();
    let matrix = GitHubMatrix { include };
    println!("{}", serde_json::to_string(&matrix)?);
    Ok(())
}
