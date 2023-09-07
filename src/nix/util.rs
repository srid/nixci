use colored::Colorize;
use tokio::process::Command;

/// Print a shell command that the user can run as-is.
///
/// FIXME: Well, this doesn't produce a copyable CLI. Just a debug dump of
/// `Command`.
pub fn print_shell_command(cmd: &Command) {
    let cmdline = format!("{:?}", cmd);
    eprintln!("> {}", cmdline.blue().bold());
}
