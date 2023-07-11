use argh::FromArgs;

const CURRENT_FLAKE_URL: &str = ".";

#[derive(FromArgs, Debug)]
/// Application configuration
pub struct CliArgs {
    /// whether to be verbose
    ///
    /// If enabled, also the full nix command output it shown.
    #[argh(switch, short = 'v')]
    pub verbose: bool,

    /// whether to pass --rebuild to nix
    #[argh(switch)]
    pub rebuild: bool,

    /// flake URL or github URL
    #[argh(positional, default = "CURRENT_FLAKE_URL.to_string()")]
    pub url: String,
}
