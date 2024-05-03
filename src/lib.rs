pub mod cli;
pub mod config;
pub mod github;
pub mod logging;
pub mod nix;

use std::collections::HashSet;
use tokio::sync::OnceCell;

use cli::{BuildConfig, CliArgs};
use colored::Colorize;
use nix::{devour_flake::DevourFlakeOutput, nix_store::StorePath};
use nix_rs::{
    command::{NixCmd, NixStoreCmd},
    flake::url::FlakeUrl,
};
use tracing::instrument;

use crate::nix::devour_flake;

static NIXCMD: OnceCell<NixCmd> = OnceCell::const_new();
static NIX_STORE_CMD: OnceCell<NixStoreCmd> = OnceCell::const_new();

pub async fn nixcmd() -> &'static NixCmd {
    NIXCMD
        .get_or_init(|| async { NixCmd::with_flakes().await.unwrap() })
        .await
}

pub async fn nix_store_cmd() -> &'static NixStoreCmd {
    NIX_STORE_CMD.get_or_init(|| async { NixStoreCmd }).await
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
            all_outs.extend(
                outs.0
                    .into_iter()
                    .map(|devour_flake::BuildOutput(path)| StorePath::BuildOutput(path)),
            );
        } else {
            tracing::info!(
                "🍊 {} {}",
                name,
                "skipped (cannot build on this system)".dimmed()
            );
        }
    }

    if build_cfg.print_all_dependencies {
        let mut all_drvs = Vec::new();
        for out in all_outs.iter() {
            if let StorePath::BuildOutput(drv_out) = out {
                let drv = nix::nix_store::nix_store_query_deriver(drv_out.clone()).await?;
                all_drvs.push(drv);
            }
        }
        for drv in all_drvs {
            let deps = nix::nix_store::nix_store_query_requisites_with_outputs(drv.clone()).await?;
            all_outs.extend(deps.into_iter());
        }
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
