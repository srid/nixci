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
    let output = devour_flake::devour_flake(cfg.url)?;
    if output.status.success() {
        // TODO: Strip devour-flake's follows-logging output
        let raw_output = String::from_utf8(output.stdout)?;
        let outs = raw_output.split_ascii_whitespace();
        if outs.clone().count() == 0 {
            println!("ERROR: No outputs produced by devour-flake");
            std::process::exit(1);
        } else {
            outs.for_each(|out| println!("out: {}", out));
            Ok(())
        }
    } else {
        println!("ERROR: devour-flake failed");
        std::process::exit(output.status.code().unwrap_or(1));
    }
}
