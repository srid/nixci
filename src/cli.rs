use std::str::FromStr;

use anyhow::Result;
use clap::Parser;

use crate::github::{self, PullRequest, PullRequestRef};

/// A reference to some flake living somewhere
#[derive(Debug, Clone)]
pub struct FlakeRef {
    kind: FlakeRefKind,
    config: String,
}

#[derive(Debug, Clone)]
pub enum FlakeRefKind {
    /// A github PR
    GithubPR(PullRequestRef),
    /// A flake URL supported by Nix commands
    Flake(String),
}

impl FromStr for FlakeRef {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<FlakeRef, String> {
        let (s, config) = s.split_once('#').unwrap_or((s, "default"));

        Ok(Self {
            kind: match github::PullRequestRef::from_web_url(s) {
                Some(pr) => FlakeRefKind::GithubPR(pr),
                None => FlakeRefKind::Flake(s.to_string()),
            },
            config: config.to_owned(),
        })
    }
}

impl FlakeRef {
    /// Convert the value to a flake URL that Nix command will recognize.
    pub fn to_flake_url(&self) -> Result<String> {
        match &self.kind {
            FlakeRefKind::GithubPR(pr) => Ok(PullRequest::get(pr)?.flake_url()),
            FlakeRefKind::Flake(url) => Ok(url.clone()),
        }
    }

    pub fn config(&self) -> &str {
        &self.config
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
    #[arg(default_value = ".#default")]
    pub flake_ref: FlakeRef,

    /// Additional arguments to pass through to `nix build`
    #[arg(last = true, default_values_t = vec![
        "--refresh".to_string(),
        "-j".to_string(),
        "auto".to_string(),
    ])]
    pub extra_nix_build_args: Vec<String>,
}
