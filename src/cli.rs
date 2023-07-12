use anyhow::Result;
use argh::{FromArgValue, FromArgs};

use crate::github::{self, PullRequest, PullRequestRef};

/// A reference to some flake living somewhere
#[derive(Debug)]
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

impl FromArgValue for FlakeRef {
    fn from_arg_value(s: &str) -> std::result::Result<FlakeRef, String> {
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

#[derive(FromArgs, Debug)]
/// Application configuration
pub struct CliArgs {
    /// whether to be verbose
    ///
    /// If enabled, also the full nix command output is shown.
    #[argh(switch, short = 'v')]
    pub verbose: bool,

    /// whether to pass --rebuild to nix
    #[argh(switch)]
    pub rebuild: bool,

    /// flake URL or github URL
    #[argh(positional, default = "FlakeRef::Flake(\".\".to_string())")]
    pub flake_ref: FlakeRef,
}
