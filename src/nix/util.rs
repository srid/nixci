use colored::Colorize;

/// Print a shell command that the user can run as-is.
pub fn print_shell_command<'a>(program: &str, args: impl Iterator<Item = &'a str>) {
    let color = |s: &str| s.blue().bold();
    eprintln!(
        "> {} {}",
        color(program),
        color(
            &args
                .map(|x| if x.contains('?') {
                    format!("\"{}\"", x)
                } else {
                    x.to_string()
                })
                .collect::<Vec<_>>()
                .join("_")
        )
    );
}
