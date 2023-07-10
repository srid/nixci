use argh::FromArgs;

const CURRENT_FLAKE_URL: &str = ".";

#[derive(FromArgs, Debug)]
/// Application configuration
pub struct Config {
    /// whether to be verbose
    #[argh(switch, short = 'v')]
    pub verbose: bool,

    /// flake URL or github URL
    #[argh(positional, default = "CURRENT_FLAKE_URL.to_string()")]
    pub url: String,
}

