pub mod cli;
pub mod config;
pub mod github;
pub mod logging;
pub mod nix;

use std::collections::HashSet;
use tokio::sync::OnceCell;

use cli::{BuildConfig, CliArgs};
use colored::Colorize;
use nix::{
    devour_flake::DevourFlakeOutput,
    nix_store::{NixStoreCmd, StorePath},
};
use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use tracing::instrument;

use crate::nix::devour_flake;

static NIXCMD: OnceCell<NixCmd> = OnceCell::const_new();

pub async fn nixcmd() -> &'static NixCmd {
    NIXCMD
        .get_or_init(|| async { NixCmd::with_flakes().await.unwrap() })
        .await
}

/// Run nixci on the given [CliArgs], returning the built outputs in sorted order.
#[instrument(name = "nixci", skip(args))]
pub async fn nixci(args: CliArgs) -> anyhow::Result<Vec<StorePath>> {
    tracing::debug!("Args: {args:?}");
    let cfg = args.command.get_config().await?;
    match args.command {
        cli::Command::Build(build_cfg) => nixci_build(args.verbose, &build_cfg, &cfg).await,
        cli::Command::DumpGithubActionsMatrix { systems, .. } => {
            let matrix = github::matrix::GitHubMatrix::from(systems, &cfg.subflakes);
            println!("{}", serde_json::to_string(&matrix)?);
            Ok(vec![])
        }
    }
}

async fn nixci_build(
    verbose: bool,
    build_cfg: &BuildConfig,
    cfg: &config::Config,
) -> anyhow::Result<Vec<StorePath>> {
    let mut all_outs = HashSet::new();

    let mut all_devour_flake_outs = HashSet::new();

    let systems = build_cfg.get_systems().await?;

    for (subflake_name, subflake) in &cfg.subflakes.0 {
        let name = format!("{}.{}", cfg.name, subflake_name).italic();
        if cfg
            .selected_subflake
            .as_ref()
            .is_some_and(|s| s != subflake_name)
        {
            tracing::info!("🍊 {} {}", name, "skipped (deselected out)".dimmed());
            continue;
        }
        tracing::info!("🍎 {}", name);
        if subflake.can_build_on(&systems) {
            let outs =
                nixci_subflake(verbose, build_cfg, &cfg.flake_url, subflake_name, subflake).await?;
            all_devour_flake_outs.extend(outs.0);
        } else {
            tracing::info!(
                "🍊 {} {}",
                name,
                "skipped (cannot build on this system)".dimmed()
            );
        }
    }

    if build_cfg.print_all_dependencies {
        let all_deps = NixStoreCmd
            .fetch_all_deps(all_devour_flake_outs.into_iter().collect())
            .await?;
        all_outs.extend(all_deps.into_iter());
    } else {
        let store_paths: HashSet<StorePath> = all_devour_flake_outs
            .into_iter()
            .map(|devour_flake::DrvOut(out_path)| StorePath::Other(out_path))
            .collect();
        all_outs.extend(store_paths);
    }

    for out in &all_outs {
        println!("{}", out);
    }

    Ok(all_outs.into_iter().collect())
}

#[instrument(skip(build_cfg, url))]
async fn nixci_subflake(
    verbose: bool,
    build_cfg: &BuildConfig,
    url: &FlakeUrl,
    subflake_name: &str,
    subflake: &config::SubFlakish,
) -> anyhow::Result<DevourFlakeOutput> {
    if subflake.override_inputs.is_empty() {
        nix::lock::nix_flake_lock_check(&url.sub_flake_url(subflake.dir.clone())).await?;
    }

    let nix_args = subflake.nix_build_args_for_flake(build_cfg, url);
    let outs = nix::devour_flake::devour_flake(verbose, nix_args).await?;
    Ok(outs)
}
