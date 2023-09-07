use colored::Colorize;
use tokio::process::Command;

/// Print a shell command that the user can run as-is.
///
/// TODO: well.
pub fn print_shell_command(cmd: &Command) {
    let cmdline = format!("{:?}", cmd);
    eprintln!("> {}", cmdline.blue().bold());
}
