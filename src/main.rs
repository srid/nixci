use anyhow::{Context, Result};
use nix_health::{traits::Checkable, NixHealth};
use nix_rs::{flake::url::FlakeUrl, info::NixInfo};
use nixci::cli;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::CliArgs::parse().await?;
    let cfg = args.command.get_config(&args.nixcmd).await?;
    let nix_info = NixInfo::from_nix(&args.nixcmd)
        .await
        .with_context(|| "Unable to gather nix info")?;
    check_nix_version(&cfg.flake_url, &nix_info).await?;
    nixci::logging::setup_logging(args.verbose);
    nixci::nixci(args).await?;
    Ok(())
}

pub async fn check_nix_version(flake_url: &FlakeUrl, nix_info: &NixInfo) -> anyhow::Result<()> {
    let nix_health = NixHealth::from_flake(flake_url).await?;
    let checks = nix_health.nix_version.check(nix_info, Some(&flake_url));
    let exit_code = NixHealth::print_report_returning_exit_code(&checks, false);

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}
