// Health checks to run before running `nixci`

use nix_health::traits::Checkable;
use nix_health::NixHealth;
use nix_rs::{flake::url::FlakeUrl, info::NixInfo};

pub async fn run_checks(flake_url: &FlakeUrl, nix_info: &NixInfo) -> anyhow::Result<()> {
    // Works only on `nix-health.default` configuration. To use a different configuration, modify the flake_url to <flake_url>#<attr>
    let nix_health_url = flake_url.with_fully_qualified_root_attr("nix-health");

    let nix_health = NixHealth::from_flake(nix_health_url.clone()).await?;

    let items: Vec<&dyn Checkable> = vec![&nix_health.nix_version];

    let checks = NixHealth::run_checks_with(items.into_iter(), nix_info, Some(nix_health_url));

    let exit_code = NixHealth::print_report_returning_exit_code(&checks, false);

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}
