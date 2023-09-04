mod cli;
mod config;
mod github;
mod nix;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

fn main() -> Result<()> {
    let args = cli::CliArgs::parse();
    if args.verbose {
        eprintln!("DEBUG {args:?}");
    }
    let url = args.flake_ref.to_flake_url()?;
    eprintln!("{}", format!("🍏 {}", url.0).bold());

    let ((cfg_name, cfg), url) = config::Config::from_flake_url(&url)?;
    if args.verbose {
        eprintln!("DEBUG {cfg:?}");
    }

    for (subflake_name, subflake) in &cfg.0 {
        let nix_args = subflake.nix_build_args_for_flake(&args, &url);
        eprintln!("🍎 {}", format!("{}.{}", cfg_name, subflake_name).italic());
        if subflake.override_inputs.is_empty() {
            nix::lock::nix_flake_lock_check(&url.sub_flake_url(subflake.dir.clone()))?;
        }

        let outs = nix::devour_flake::devour_flake(args.verbose, nix_args)?;
        if outs.is_empty() {
            eprintln!("Warn: devour-flake produced no outputs")
        } else {
            for out in outs {
                println!("{}", out.0.bold());
            }
        }
    }
    Ok(())
}
