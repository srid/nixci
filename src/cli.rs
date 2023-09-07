use std::str::FromStr;

use anyhow::Result;
use clap::Parser;

use crate::{
    github::{self, PullRequest, PullRequestRef},
    nix::url::FlakeUrl,
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
    pub fn to_flake_url(&self) -> Result<FlakeUrl> {
        match self {
            FlakeRef::GithubPR(pr) => Ok(PullRequest::get(pr)?.flake_url()),
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

    /// Additional arguments to pass through to `nix build`
    #[arg(last = true, default_values_t = vec![
        "--refresh".to_string(),
        "-j".to_string(),
        "auto".to_string(),
    ])]
    pub extra_nix_build_args: Vec<String>,
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
