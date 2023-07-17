use std::str::FromStr;

use anyhow::Result;
use clap::Parser;

use crate::github::{self, PullRequest, PullRequestRef};

/// A reference to some flake living somewhere
#[derive(Debug, Clone)]
pub enum FlakeRef {
    /// A github PR
    GithubPR(PullRequestRef),
    /// A flake URL supported by Nix commands
    Flake(String),
}

impl Default for FlakeRef {
    fn default() -> Self {
        let root_flake = FlakeRef::Flake(".".to_string());
        root_flake
    }
}

impl FromStr for FlakeRef {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<FlakeRef, String> {
        let flake_ref = match github::PullRequestRef::from_web_url(&s.to_string()) {
            Some(pr) => FlakeRef::GithubPR(pr),
            None => FlakeRef::Flake(s.to_string()),
        };
        Ok(flake_ref)
    }
}

impl FlakeRef {
    /// Convert the value to a flake URL that Nix command will recognize.
    pub fn to_flake_url(&self) -> Result<String> {
        match self {
            FlakeRef::GithubPR(pr) => Ok(PullRequest::get(&pr)?.flake_url()),
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

    /// Whether to pass --rebuild to nix
    #[arg(long)]
    pub rebuild: bool,

    /// Whether to avoid passing --refresh to nix
    #[arg(long)]
    pub no_refresh: bool,

    /// What system to build the Nix expressions for
    #[arg(long, short)]
    pub system: Option<String>,

    /// Flake URL or github URL
    #[arg(default_value = ".")]
    pub flake_ref: FlakeRef,
}
