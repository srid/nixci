// Health checks to run before running `nixci`

use nix_health::traits::Checkable;
use nix_health::NixHealth;
use nix_rs::{flake::url::FlakeUrl, info::NixInfo};

pub async fn run_checks(flake_url: &FlakeUrl, nix_info: &NixInfo) -> anyhow::Result<()> {
    let nix_health = NixHealth::from_flake(flake_url).await?;

    let checks = nix_health
        .nix_version
        .check(nix_info, Some(flake_url.clone()));

    let exit_code = NixHealth::print_report_returning_exit_code(&checks, false);

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}
