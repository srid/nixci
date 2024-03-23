pub mod cli;
pub mod config;
pub mod github;
pub mod logging;
pub mod nix;

use std::collections::HashSet;

use cli::{BuildConfig, CliArgs};
use colored::Colorize;
use nix::devour_flake::{DevourFlakeOutput, DrvOut};
use nix_rs::flake::url::FlakeUrl;
use tracing::instrument;

/// Run nixci on the given [CliArgs], returning the built outputs in sorted order.
#[instrument(name = "nixci", skip(args))]
pub async fn nixci(args: CliArgs) -> anyhow::Result<Vec<DrvOut>> {
    tracing::debug!("Args: {args:?}");
    match args.command {
        cli::Command::Build(build_cfg) => {
            let url = build_cfg.flake_ref.to_flake_url().await?;
            tracing::info!("{}", format!("🍏 {}", url.0).bold());
            let cfg = config::Config::from_flake_url(&url).await?;
            tracing::debug!("Config: {cfg:?}");
            nixci_build(args.verbose, &build_cfg, &url, &cfg).await
        }
        cli::Command::DumpGithubActionsMatrix { flake_ref, systems } => {
            let url = flake_ref.to_flake_url().await?;
            let cfg = config::Config::from_flake_url(&url).await?;
            github::dump_github_actions_matrix(&cfg, systems).await?;
            // TODO: Return something meaningful, or break the function.
            Ok(vec![])
        }
    }
}

async fn nixci_build(
    verbose: bool,
    build_cfg: &BuildConfig,
    url: &FlakeUrl,
    cfg: &config::Config,
) -> anyhow::Result<Vec<DrvOut>> {
    let mut all_outs = HashSet::new();

    let systems = build_cfg.get_build_systems().await?;

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
            let outs = nixci_subflake(verbose, build_cfg, url, subflake_name, subflake).await?;
            all_outs.extend(outs.0);
        } else {
            tracing::info!(
                "🍊 {} {}",
                name,
                "skipped (cannot build on this system)".dimmed()
            );
        }
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
    for out in &outs.0 {
        println!("{}", out.0.bold());
    }
    Ok(outs)
}
