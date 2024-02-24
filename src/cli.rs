use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use nix_rs::{
    command::{NixCmd, NixCmdError},
    config::NixConfig,
    flake::{system::System, url::FlakeUrl},
};

use crate::{
    github::{self, PullRequest, PullRequestRef},
    nix::system_list::SystemsList,
};

/// A reference to some flake living somewhere
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlakeRef {
    /// A github PR
    GithubPR(PullRequestRef),
    /// A flake URL supported by Nix commands
    Flake(FlakeUrl),
}

impl FromStr for FlakeRef {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<FlakeRef, String> {
        let flake_ref = match github::PullRequestRef::from_web_url(s) {
            Some(pr) => FlakeRef::GithubPR(pr),
            None => FlakeRef::Flake(FlakeUrl(s.to_string())),
        };
        Ok(flake_ref)
    }
}

impl FlakeRef {
    /// Convert the value to a flake URL that Nix command will recognize.
    pub async fn to_flake_url(&self) -> Result<FlakeUrl> {
        match self {
            FlakeRef::GithubPR(pr) => {
                let pr = PullRequest::get(pr).await?;
                Ok(pr.flake_url())
            }
            FlakeRef::Flake(url) => Ok(url.clone()),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author = "Sridhar Ratnakumar", version, about)]
/// nixci - Define and build CI for Nix projects anywhere <https://github.com/srid/nixci>
pub struct CliArgs {
    /// Whether to be verbose
    ///
    /// If enabled, also the full nix command output is shown.
    #[arg(short = 'v')]
    pub verbose: bool,

    /// Flake URL or github URL
    ///
    /// A specific nixci` configuration can be specified
    /// using '#': e.g. `nixci .#extra-tests`
    #[arg(default_value = ".")]
    pub flake_ref: FlakeRef,

    /// The systems list to build for. If empty, build for current system.
    ///
    /// Must be a flake reference which, when imported, must return a Nix list
    /// of systems. You may use one of the lists from
    /// https://github.com/nix-systems.
    #[arg(long, default_value = "github:nix-systems/empty")]
    pub build_systems_from: FlakeUrl,

    /// Additional arguments to pass through to `nix build`
    #[arg(last = true, default_values_t = vec![
        "--refresh".to_string(),
        "-j".to_string(),
        "auto".to_string(),
    ])]
    pub extra_nix_build_args: Vec<String>,
}

impl CliArgs {
    pub async fn get_build_systems(&self) -> Result<Vec<System>> {
        let systems = SystemsList::from_flake(&self.build_systems_from).await?.0;
        if systems.is_empty() {
            let current_system = get_current_system().await?;
            Ok(vec![current_system])
        } else {
            Ok(systems)
        }
    }
}

async fn get_current_system() -> Result<System, NixCmdError> {
    let config = NixConfig::from_nix(&NixCmd::default()).await?;
    Ok(config.system.value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_pr() {
        assert_eq!(
            FlakeRef::from_str("https://github.com/srid/nixci/pull/19").unwrap(),
            FlakeRef::GithubPR(PullRequestRef {
                owner: "srid".to_string(),
                repo: "nixci".to_string(),
                pr: 19
            })
        );
    }

    #[test]
    fn test_current_dir() {
        assert_eq!(
            FlakeRef::from_str(".").unwrap(),
            FlakeRef::Flake(FlakeUrl(".".to_string()))
        );
    }

    #[test]
    fn test_flake_url() {
        assert_eq!(
            FlakeRef::from_str("github:srid/nixci").unwrap(),
            FlakeRef::Flake(FlakeUrl("github:srid/nixci".to_string()))
        );
    }
}
