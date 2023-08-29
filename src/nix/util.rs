use colored::Colorize;
use std::io::{stderr, BufWriter, Write};

/// Print a shell command that the user can run as-is.
pub fn print_shell_command<'a>(program: &str, args: impl Iterator<Item = &'a str>) {
    let mut stderr = BufWriter::new(stderr().lock());

    let color = |s: &str| s.blue().bold();

    let _ = write!(stderr, "> {}", color(program));

    for arg in args {
        let _ = write!(stderr, " {}", color(arg));
    }

    let _ = writeln!(stderr);
    let _ = stderr.flush();
}
