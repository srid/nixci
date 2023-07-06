use argh::FromArgs;
use std::io;
use std::process::Command;

const CURRENT_FLAKE: &str = ".";

#[derive(FromArgs, Debug)]
/// Application configuration
struct Config {
    /// whether to be verbose
    #[argh(switch, short = 'v')]
    verbose: bool,

    /// flake URL or github URL
    #[argh(positional, default = "CURRENT_FLAKE.to_string()")]
    url: String,
}

fn main() -> io::Result<()> {
    let cfg = argh::from_env::<Config>();
    if cfg.verbose {
        println!("DEBUG {cfg:?}");
    }
    println!("Running nixci on {}", cfg.url.to_string());
    let output = Command::new("devour-flake")
        .arg(cfg.url)
        .spawn()?
        .wait_with_output()?;
    if output.status.success() {
        println!("\n"); // devour-flake doesn't end in a newline.
        Ok(())
    } else {
        println!("\nERROR: devour-flake failed");
        std::process::exit(output.status.code().unwrap_or(1));
    }
}
