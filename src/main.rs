mod app;
mod config;
mod devour_flake;

use anyhow::{bail, Result};

fn main() -> Result<()> {
    let args = argh::from_env::<app::CliArgs>();
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
    for (cfg_name, cfg) in cfgs.0 {
        println!("FLAKE: {}", cfg_name);
        // TODO: override inputs
        let flake_url = format!("{}?dir={}", args.url, cfg.dir);
        let extra_args = cfg
            .override_inputs
            .iter()
            .flat_map(|(k, v)| {
                [
                    "--override-input".to_string(),
                    format!("flake/{}", k),
                    v.to_string(),
                ]
            })
            .collect::<Vec<String>>();
        println!("extra_args: {extra_args:?}");

        let outs = devour_flake::devour_flake(flake_url, extra_args)?;
        if outs.len() == 0 {
            bail!("No outputs produced by devour-flake")
        } else {
            for out in outs {
                println!("out: {}", out);
            }
        }
    }
    Ok(())
}
