pub mod cli;
pub mod config;
pub mod github;
pub mod logging;
pub mod nix;

use std::collections::HashSet;

use cli::CliArgs;
use colored::Colorize;
use nix::devour_flake::{DevourFlakeOutput, DrvOut};
use nix_rs::{
    command::{NixCmd, NixCmdError},
    config::NixConfig,
    flake::{system::System, url::FlakeUrl},
};
use tracing::instrument;

/// Run nixci on the given [CliArgs], returning the built outputs in sorted order.
#[instrument(name = "nixci", skip(args))]
pub async fn nixci(args: CliArgs) -> anyhow::Result<Vec<DrvOut>> {
    tracing::debug!("Args: {args:?}");
    let url = args.flake_ref.to_flake_url().await?;
    tracing::info!("{}", format!("🍏 {}", url.0).bold());

    let ((cfg_name, cfg), url) = config::Config::from_flake_url(&url).await?;
    tracing::debug!("Config: {cfg:?}");

    let mut all_outs = HashSet::new();

    let system = get_current_system().await?;

    for (subflake_name, subflake) in &cfg.0 {
        tracing::info!("🍎 {}", format!("{}.{}", cfg_name, subflake_name).italic());
        if subflake.can_build_on(&system) {
            let outs = nixci_subflake(&args, &url, &subflake_name, &subflake).await?;
            all_outs.extend(outs.0);
        } else {
            tracing::info!("🍊 {}", "skipped".dimmed());
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
    let nix_args = subflake.nix_build_args_for_flake(cli_args, url);
    if subflake.override_inputs.is_empty() {
        nix::lock::nix_flake_lock_check(&url.sub_flake_url(subflake.dir.clone())).await?;
    }

    let outs = nix::devour_flake::devour_flake(cli_args.verbose, nix_args).await?;
    for out in &outs.0 {
        println!("{}", out.0.bold());
    }
    Ok(outs)
}

async fn get_current_system() -> Result<System, NixCmdError> {
    let config = NixConfig::from_nix(&NixCmd::default()).await?;
    Ok(config.system.value)
}
