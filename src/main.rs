use argh::FromArgs;
use std::process::{Command, Stdio};

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

type AppError = Box<dyn std::error::Error>;
type AppResult<T> = Result<T, AppError>;

fn main() -> AppResult<()> {
    let cfg = argh::from_env::<Config>();
    if cfg.verbose {
        println!("DEBUG {cfg:?}");
    }
    println!("Running nixci on {}", cfg.url.to_string());
    let output = Command::new("devour-flake")
        .arg(cfg.url)
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    if output.status.success() {
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
