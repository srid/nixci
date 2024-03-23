pub mod cli;
pub mod config;
pub mod github;
pub mod logging;
pub mod nix;

use std::collections::HashSet;

use cli::CliArgs;
use colored::Colorize;
use nix::devour_flake::{DevourFlakeOutput, DrvOut};
use nix_rs::flake::url::FlakeUrl;
use tracing::instrument;

/// Run nixci on the given [CliArgs], returning the built outputs in sorted order.
#[instrument(name = "nixci", skip(args))]
pub async fn nixci(args: CliArgs) -> anyhow::Result<Vec<DrvOut>> {
    tracing::debug!("Args: {args:?}");
    let url = args.flake_ref.to_flake_url().await?;
    tracing::info!("{}", format!("🍏 {}", url.0).bold());

    let cfg = config::Config::from_flake_url(&url).await?;
    tracing::debug!("Config: {cfg:?}");

    if args.dump_github_actions_matrix {
        github::dump_github_actions_matrix(&cfg).await?;
        Ok(vec![])
    } else {
        nixci_build(&args, &url, &cfg).await
    }
}

async fn nixci_build(
    args: &CliArgs,
    url: &FlakeUrl,
    cfg: &config::Config,
) -> anyhow::Result<Vec<DrvOut>> {
    let mut all_outs = HashSet::new();

    let systems = args.get_build_systems().await?;

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
            let outs = nixci_subflake(args, url, subflake_name, subflake).await?;
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

#[instrument(skip(cli_args, url))]
async fn nixci_subflake(
    cli_args: &CliArgs,
    url: &FlakeUrl,
    subflake_name: &str,
    subflake: &config::SubFlakish,
) -> anyhow::Result<DevourFlakeOutput> {
    if subflake.override_inputs.is_empty() {
        nix::lock::nix_flake_lock_check(&url.sub_flake_url(subflake.dir.clone())).await?;
    }

    let nix_args = subflake.nix_build_args_for_flake(cli_args, url);
    let outs = nix::devour_flake::devour_flake(cli_args.verbose, nix_args).await?;
    for out in &outs.0 {
        println!("{}", out.0.bold());
    }
    Ok(outs)
}
