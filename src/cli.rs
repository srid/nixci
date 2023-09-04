use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use serde::Deserialize;

use crate::github::{self, PullRequest, PullRequestRef};

/// A reference to some flake living somewhere
#[derive(Debug, Clone)]
pub enum FlakeRef {
    /// A github PR
    GithubPR(PullRequestRef),
    /// A flake URL supported by Nix commands
    Flake(FlakeUrl),
}

/// A valid flake URL
///
/// See [syntax here](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html#url-like-syntax).
#[derive(Debug, Clone, Deserialize)]
pub struct FlakeUrl(pub String);

/// The attribute output part of a [FlakeUrl]
///
/// Example: `foo` in `.#foo`.
#[derive(Debug, Clone)]
pub struct FlakeAttr(Option<String>);

impl FlakeUrl {
    /// Get the [FlakeAttr] pointed by this flake.
    pub fn get_attr(&self) -> FlakeAttr {
        self.split_attr().1
    }

    pub fn without_attr(&self) -> FlakeUrl {
        self.split_attr().0
    }

    fn split_attr(&self) -> (Self, FlakeAttr) {
        match self.0.split_once('#') {
            Some((url, attr)) => (FlakeUrl(url.to_string()), FlakeAttr(Some(attr.to_string()))),
            None => (self.clone(), FlakeAttr(None)),
        }
    }

    /// Return the flake URL pointing to the sub-flake
    pub fn sub_flake_url(&self, dir: String) -> FlakeUrl {
        if dir == "." {
            self.clone()
        } else {
            FlakeUrl(format!("{}?dir={}", self.0, dir))
        }
    }
}

impl FlakeAttr {
    /// Get the attribute name.
    ///
    /// If attribute exists, then return "default".
    pub fn get_name(&self) -> String {
        self.0.clone().unwrap_or_else(|| "default".to_string())
    }
}

impl FromStr for FlakeRef {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<FlakeRef, String> {
        Ok(match github::PullRequestRef::from_web_url(s) {
            Some(pr) => FlakeRef::GithubPR(pr),
            None => FlakeRef::Flake(FlakeUrl(s.to_string())),
        })
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
