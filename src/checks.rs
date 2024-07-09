// Health checks to run before running `nixci`

use colored::Colorize;
use nix_health::{
    traits::{CheckResult, Checkable},
    NixHealth,
};
use nix_rs::{flake::url::FlakeUrl, info::NixInfo};

pub async fn run_checks(flake_url: &FlakeUrl, nix_info: &NixInfo) -> anyhow::Result<()> {
    let nix_health_url = flake_url.with_fully_qualified_root_attr("nix-health");

    let nix_health = NixHealth::from_flake(nix_health_url.clone()).await?;

    let checks = nix_health
        .nix_version
        .check(&nix_info, Some(nix_health_url));

    let min_nix_version_check = checks.first().unwrap();

    match &min_nix_version_check.result {
        CheckResult::Green => {
            println!(
                "{}",
                format!("✅ {}", min_nix_version_check.title).green().bold()
            );
            println!("   {}", min_nix_version_check.info.blue());
        }
        CheckResult::Red { msg, suggestion } => {
            if min_nix_version_check.required {
                println!(
                    "{}",
                    format!("❌ {}", min_nix_version_check.title).red().bold()
                );
            } else {
                println!(
                    "{}",
                    format!("🟧 {}", min_nix_version_check.title)
                        .yellow()
                        .bold()
                );
            }
            println!("   {}", min_nix_version_check.info.blue());
            println!("   {}", msg.yellow());
            println!("   {}", suggestion);
            if min_nix_version_check.required {
                std::process::exit(1)
            }
        }
    }
    Ok(())
}
