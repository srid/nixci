mod cli;
mod config;
mod nix;

use anyhow::{bail, Result};

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
        println!("FLAKE: {}", cfg_name);
        let nix_args = cfg.nix_build_args_for_flake(args.url.clone());
        println!("extra_args: {nix_args:?}");

        let outs = nix::devour_flake::devour_flake(nix_args)?;
        if outs.len() == 0 {
            bail!("No outputs produced by devour-flake")
        } else {
            for out in outs {
                println!("out: {}", out);
            }
        }
    }
    println!("Finished building flakes:");
    for (cfg_name, _) in &cfgs.0 {
        println!("FLAKE: {}", cfg_name);
    }
    Ok(())
}
