mod app;
mod devour_flake;

fn main() -> app::AppResult<()> {
    // TODO: Subflake support: parse `.envrc`? Or `nixci.json`?
    let cfg = argh::from_env::<app::Config>();
    if cfg.verbose {
        println!("DEBUG {cfg:?}");
    }
    // TODO: Handle github urls, in particular PR urls
    println!("Running nixci on {}", cfg.url.to_string());
    // TODO: Run `nix flake lock --no-update-lock-file` and report on it.
    let outs = devour_flake::devour_flake(cfg.url)?;
    if outs.len() == 0 {
        println!("ERROR: No outputs produced by devour-flake");
        std::process::exit(1);
    } else {
        for out in outs {
            println!("out: {}", out);
        }
        Ok(())
    }
}
