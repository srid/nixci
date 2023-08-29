use std::str::FromStr;

use anyhow::Result;
use clap::Parser;

use crate::github::{self, PullRequest, PullRequestRef};

/// A reference to some flake living somewhere
#[derive(Debug, Clone)]
pub struct FlakeRef {
    /// Which flake attribute to evaluate for the configuration
    pub config_attr: String,
    /// What kind of flake ref we're dealing with
    kind: FlakeKind,
}

#[derive(Debug, Clone)]
pub enum FlakeKind {
    /// A github PR
    GithubPR(PullRequestRef),
    /// A flake URL supported by Nix commands
    Flake(String),
}

impl FromStr for FlakeRef {
    type Err = String;
    fn from_str(input: &str) -> std::result::Result<FlakeRef, String> {
        let mut split = input.split('#');
        let s = split.next().unwrap_or(input);
        let config_attr = split.next().unwrap_or("nixci").to_owned();

        Ok(Self {
            config_attr,
            kind: match github::PullRequestRef::from_web_url(s) {
                Some(pr) => FlakeKind::GithubPR(pr),
                None => FlakeKind::Flake(s.to_string()),
            },
        })
    }
}

impl FlakeRef {
    /// Convert the value to a flake URL that Nix command will recognize.
    pub fn to_flake_url(&self) -> Result<String> {
        match &self.kind {
            FlakeKind::GithubPR(pr) => Ok(PullRequest::get(&pr)?.flake_url()),
            FlakeKind::Flake(url) => Ok(url.clone()),
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

    /// Flake URL or github URL. If no configuration attribute is specified, `nixci` will be used
    /// by default.
    #[arg(default_value = ".#nixci")]
    pub flake_ref: FlakeRef,

    /// Additional arguments to pass through to `nix build`
    #[arg(last = true, default_values_t = vec![
        "--refresh".to_string(),
        "-j".to_string(),
        "auto".to_string(),
    ])]
    pub extra_nix_build_args: Vec<String>,
}
