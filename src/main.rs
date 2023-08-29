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
    eprintln!("{}", format!("🍏 {}", url).bold());

    let cfgs = config::Config::from_flake_url(url.clone())?;
    if args.verbose {
        eprintln!("DEBUG {cfgs:?}");
    }

    for (cfg_name, cfg) in &cfgs.0 {
        let nix_args = cfg.nix_build_args_for_flake(&args, &url);
        eprintln!("🍎 {}", cfg_name.italic());
        if cfg.override_inputs.is_empty() {
            nix::lock::nix_flake_lock_check(&cfg.sub_flake_url(&url))?;
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
