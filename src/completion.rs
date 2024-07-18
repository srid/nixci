use crate::cli::CliArgs;
use async_trait::async_trait;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
// use color_eyre::Result;
use anyhow::Result;
use std::io::stdout;
use std::process::ExitCode;

/// Prints completion for shells to use.
#[derive(Parser)]
pub(crate) struct CompletionSubcommand {
    /// Shell
    #[arg(value_enum)]
    shell: Shell,
}

#[async_trait]
pub trait CommandExecute {
    async fn execute(self) -> Result<ExitCode>;
}

impl CompletionSubcommand {
    pub fn new(shell: Shell) -> Self {
        Self { shell }
    }
}

#[async_trait]
impl CommandExecute for CompletionSubcommand {
    async fn execute(self) -> Result<ExitCode> {
        let mut cli = CliArgs::command();
        let cli_name = cli.get_name().to_string();
        generate(self.shell, &mut cli, cli_name, &mut stdout());

        Ok(ExitCode::SUCCESS)
    }
}
