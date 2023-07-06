use argh::FromArgs;
use std::io;
use std::process::Command;

#[derive(FromArgs, Debug)]
/// Application configuration
struct Config {
    /// whether to be verbose
    #[argh(switch, short = 'v')]
    verbose: bool,

    /// flake URL or github URL
    #[argh(positional, default = r#" ".".to_string() "#)]
    url: String,
}

fn main() -> io::Result<()> {
    let cfg = argh::from_env::<Config>();
    if cfg.verbose {
        println!("DEBUG {cfg:?}");
    }
    println!("Running nixci on {}", cfg.url.to_string());
    let c = Command::new("devour-flake").arg(cfg.url).spawn()?;
    let output = c.wait_with_output()?;
    println!("\noutput: {:?}", output);
    Ok(())
}
