use anyhow::Result;
use clap::Parser;
use nixci::cli;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::CliArgs::parse();
    let _outs = nixci::nixci(args).await?;
    Ok(())
}
