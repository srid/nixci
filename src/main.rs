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
    println!("Running nixci on {}", args.url.to_string());

    // TODO: Run `nix flake lock --no-update-lock-file` and report on it.

    let cfgs = config::Config::from_flake_url(args.url.clone())?;
    if args.verbose {
        println!("DEBUG {cfgs:?}");
    }
    for (cfg_name, cfg) in &cfgs.0 {
        let nix_args = cfg.nix_build_args_for_flake(args.url.clone());
        println!(
            "{}",
            format!("> FLAKE[{}] devour-flake {}", cfg_name, nix_args.join(" ")).blue()
        );

        let outs = nix::devour_flake::devour_flake(nix_args)?;
        if outs.len() == 0 {
            bail!("No outputs produced by devour-flake")
        } else {
            for out in outs {
                println!("{}", out.green().bold());
            }
        }
    }
    Ok(())
}
