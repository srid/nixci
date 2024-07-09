use std::str::FromStr;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use nix_rs::{
    command::{NixCmd, NixCmdError},
    config::NixConfig,
    flake::{system::System, url::FlakeUrl},
};

use crate::{
    config,
    github::pull_request::{PullRequest, PullRequestRef},
    nix::{
        devour_flake,
        system_list::{SystemsList, SystemsListFlakeRef},
    },
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
        let flake_ref = match PullRequestRef::from_web_url(s) {
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
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Nix command global options
    #[command(flatten)]
    pub nixcmd: NixCmd,

    #[clap(subcommand)]
    pub command: Command,
}

impl CliArgs {
    /// Parse `CliArgs` from command-line args
    pub async fn parse() -> Result<Self, NixCmdError> {
        let mut args = <Self as Parser>::parse();
        args.nixcmd = args.nixcmd.with_flakes().await?;
        Ok(args)
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Build all outputs of a flake
    Build(BuildConfig),

    /// Print the Github Actions matrix configuration as JSON
    #[clap(name = "gh-matrix")]
    DumpGithubActionsMatrix {
        /// Flake URL or github URL
        ///
        /// A specific nixci configuration can be specified
        /// using '#': e.g. `nixci .#extra-tests`
        #[arg(default_value = ".")]
        flake_ref: FlakeRef,

        /// Systems to include in the matrix
        #[arg(long, value_parser, value_delimiter = ',')]
        systems: Vec<System>,
    },
}

impl Command {
    /// Get the nixci [config::Config] associated with this subcommand
    pub async fn get_config(&self, cmd: &NixCmd) -> anyhow::Result<config::Config> {
        let flake_ref = match self {
            Command::Build(build_cfg) => &build_cfg.flake_ref,
            Command::DumpGithubActionsMatrix { flake_ref, .. } => flake_ref,
        };
        let url = flake_ref.to_flake_url().await?;
        tracing::info!("{}", format!("🍏 {}", url.0).bold());
        let cfg = config::Config::from_flake_url(cmd, &url).await?;
        tracing::debug!("Config: {cfg:?}");
        Ok(cfg)
    }
}

#[derive(Parser, Debug)]
pub struct BuildConfig {
    /// The systems list to build for. If empty, build for current system.
    ///
    /// Must be a flake reference which, when imported, must return a Nix list
    /// of systems. You may use one of the lists from
    /// https://github.com/nix-systems.
    #[arg(long, default_value = "github:nix-systems/empty")]
    pub systems: SystemsListFlakeRef,

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

    /// Print build and runtime dependencies along with out paths
    ///
    /// By default, `nixci build` prints only the out paths. This option is
    /// useful to explicitly push all dependencies to a cache.
    #[clap(long, short = 'd')]
    pub print_all_dependencies: bool,
}

impl BuildConfig {
    pub async fn get_systems(&self, cmd: &NixCmd) -> Result<Vec<System>> {
        let systems = SystemsList::from_flake(cmd, &self.systems).await?.0;
        if systems.is_empty() {
            let current_system = get_current_system(cmd).await?;
            Ok(vec![current_system])
        } else {
            Ok(systems)
        }
    }

    pub fn preprocess(&self) -> anyhow::Result<BuildConfig> {
        let preprocessed_extra_nix_build_args =
            devour_flake::transform_override_inputs(&self.extra_nix_build_args)?;
        Ok(BuildConfig {
            flake_ref: self.flake_ref.clone(),
            systems: self.systems.clone(),
            extra_nix_build_args: preprocessed_extra_nix_build_args,
            print_all_dependencies: self.print_all_dependencies,
        })
    }
}

async fn get_current_system(cmd: &NixCmd) -> Result<System, NixCmdError> {
    let config = NixConfig::from_nix(cmd).await?;
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
