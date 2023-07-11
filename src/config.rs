use std::{
    collections::HashMap,
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

/// Rust type for the `nixci` flake output
///
/// Example `flake.nix` output this type expects:
/// ```nix
/// {
///   nixci.root = {
///     flakeDir = ".";
///     overrideInputs = { "foo" = "./foo"; };
///   };
/// }
#[derive(Debug, Deserialize)]
pub struct Config(pub HashMap<String, SubFlakish>);

impl Default for Config {
    fn default() -> Self {
        let mut m = HashMap::new();
        m.insert("root".to_string(), SubFlakish::default());
        Config(m)
    }
}

/// Not really a subflake, but it depends on "follows" on project root.
#[derive(Debug, Deserialize)]
pub struct SubFlakish {
    pub dir: String,

    #[serde(rename = "overrideInputs", default)]
    pub override_inputs: HashMap<String, String>,
}

impl Default for SubFlakish {
    fn default() -> Self {
        SubFlakish {
            dir: ".".to_string(),
            override_inputs: HashMap::new(),
        }
    }
}

impl Config {
    pub fn from_flake_url(url: String) -> Result<Self> {
        nix_eval_attr_json::<Config>("nixci", url)
    }
}

/// Run 'nix eval .#attr --json` and parse its JSON
///
/// If the flake does not output the given attribute, use the `Default`
/// implementation of `T`.
pub fn nix_eval_attr_json<T>(attr: &str, url: String) -> Result<T>
where
    T: Default + serde::de::DeserializeOwned,
{
    let output = Command::new("nix")
        .arg("--refresh")
        .arg("eval")
        .arg(format!("{}#{}", url, attr))
        .arg("--json")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()
        .with_context(|| "No 'nixci' config?")?;
    if output.status.success() {
        let raw_output = String::from_utf8(output.stdout)
            .with_context(|| format!("Failed to decode 'nix eval' stdout as UTF-8"))?;
        Ok(serde_json::from_str::<T>(&raw_output)?)
    } else {
        let raw_output = String::from_utf8(output.stderr)
            .with_context(|| format!("Failed to decode 'nix eval' stderr as UTF-8"))?;
        if raw_output.contains("does not provide attribute") {
            // The 'nixci' flake output attr is missing. User wants the default config.
            Ok(T::default())
        } else {
            let exit_code = output.status.code().unwrap_or(1);
            bail!("nix eval failed to run (exited: {})", exit_code);
        }
    }
}
