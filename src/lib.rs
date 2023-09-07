pub mod cli;
pub mod config;
pub mod github;
pub mod nix;

use anyhow::bail;
use cli::CliArgs;
use colored::Colorize;
use nix::devour_flake::DrvOut;

/// Run nixci on the given [CliArgs], returning the built outputs in sorted order.
pub async fn nixci(args: CliArgs) -> anyhow::Result<Vec<DrvOut>> {
    if args.verbose {
        eprintln!("DEBUG {args:?}");
    }
    let url = args.flake_ref.to_flake_url()?;
    eprintln!("{}", format!("🍏 {}", url.0).bold());

    let ((cfg_name, cfg), url) = config::Config::from_flake_url(&url).await?;
    if args.verbose {
        eprintln!("DEBUG {cfg:?}");
    }

    let mut all_outs = vec![];

    for (subflake_name, subflake) in &cfg.0 {
        let nix_args = subflake.nix_build_args_for_flake(&args, &url);
        eprintln!("🍎 {}", format!("{}.{}", cfg_name, subflake_name).italic());
        if subflake.override_inputs.is_empty() {
            nix::lock::nix_flake_lock_check(&url.sub_flake_url(subflake.dir.clone()))?;
        }

        let outs = nix::devour_flake::devour_flake(args.verbose, nix_args).await?;
        if outs.is_empty() {
            bail!("Warn: devour-flake produced no outputs");
        } else {
            for out in &outs {
                println!("{}", out.0.bold());
            }
            all_outs.extend(outs);
        }
    }
    all_outs.sort();
    Ok(all_outs)
}
