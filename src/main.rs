mod cli;
mod config;
mod nix;

use anyhow::{bail, Result};
use colored::Colorize;

fn main() -> Result<()> {
    let args = argh::from_env::<cli::CliArgs>();
    if args.verbose {
        println!("DEBUG {args:?}");
    }
    // TODO: Handle github urls, in particular PR urls

    let cfgs = config::Config::from_flake_url(args.url.clone())?;
    if args.verbose {
        println!("DEBUG {cfgs:?}");
    }

    for (_cfg_name, cfg) in &cfgs.0 {
        let nix_args = cfg.nix_build_args_for_flake(&args.url, args.rebuild);
        println!("🍎");
        if cfg.override_inputs.is_empty() {
            nix::lock::nix_flake_lock_check(&cfg.sub_flake_url(&args.url))?;
        }

        let outs = nix::devour_flake::devour_flake(args.verbose, nix_args)?;
        if outs.len() == 0 {
            bail!("No outputs produced by devour-flake")
        } else {
            for out in outs {
                println!("{}", out.0.bold());
            }
        }
    }
    Ok(())
}
