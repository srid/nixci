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

impl SubFlakish {
    /// Return the `nix build` arguments for building all the outputs in this
    /// subflake configuration.
    pub fn build_nix_build_args_for_flake(&self, flake_url: String) -> Vec<String> {
        let sub_flake_url = format!("{}?dir={}", flake_url, self.dir);
        let mut extra_args = self
            .override_inputs
            .iter()
            .flat_map(|(k, v)| {
                [
                    "--override-input".to_string(),
                    // We must prefix the input with "flake" because
                    // devour-flake uses that input name to refer to the user's
                    // flake.
                    format!("flake/{}", k),
                    v.to_string(),
                ]
            })
            .collect::<Vec<String>>();
        extra_args.insert(0, sub_flake_url);
        extra_args
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
