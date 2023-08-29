/// Build a shell command that the user can run as-is.
pub fn build_shell_command<'a>(program: String, args: impl Iterator<Item = &'a str>) -> String {
    format!(
        "{} {}",
        program,
        args.map(|x|
                // If the argument contains a special character, it must be
                // quoted. We check only for "?", though.
                if x.contains('?') { format!("\"{}\"", x) } else { x.to_string() })
            .collect::<Vec<String>>()
            .join(" ")
    )
}
