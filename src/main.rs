use anyhow::{Context, Result};
use nix_rs::info::NixInfo;
use nixci::cli;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::CliArgs::parse().await?;
    let cfg = args.command.get_config(&args.nixcmd).await?;
    let nix_info = NixInfo::from_nix(&args.nixcmd)
        .await
        .with_context(|| "Unable to gather nix info")?;
    nixci::checks::run_checks(&cfg.flake_url, &nix_info).await?;
    nixci::logging::setup_logging(args.verbose);
    nixci::nixci(args).await?;
    Ok(())
}
