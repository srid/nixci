pub fn build_shell_command(program: String, args: &Vec<String>) -> String {
    format!(
        "{} {}",
        program,
        args.iter()
            .map(|x| if x.contains("?") {
                format!("\"{}\"", x)
            } else {
                x.to_string()
            })
            .collect::<Vec<String>>()
            .join(" ")
    )
}
