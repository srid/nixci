use anyhow::Result;
use clap::Parser;
use nixci::cli;

fn main() -> Result<()> {
    let args = cli::CliArgs::parse();
    let _outs = nixci::nixci(args)?;
    Ok(())
}
