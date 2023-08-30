use colored::Colorize;

/// Print a shell command that the user can run as-is.
pub fn print_shell_command<'a>(program: &str, args: impl Iterator<Item = &'a str>) {
    let color = |s: &str| s.blue().bold();
    eprintln!(
        "> {} {}",
        color(program),
        color(
            &args
                // If the argument contains a special character, it must be
                // quoted. We currently check only for "?", though.
                .map(|x| if x.contains('?') {
                    format!("\"{}\"", x)
                } else {
                    x.to_string()
                })
                .collect::<Vec<_>>()
                .join(" ")
        )
    );
}
