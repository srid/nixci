use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

use crate::nix;

/// Rust type for the `nixci` flake output
///
/// Example `flake.nix` output this type expects:
/// ```nix
/// {
///   nixci.test = {
///     flakeDir = "./test";
///     overrideInputs = { "mymod" = "."; };
///   };
/// }
#[derive(Debug, Deserialize)]
pub struct Config(pub HashMap<String, SubFlakish>);

impl Default for Config {
    /// Default value contains a single entry for the root flake.
    fn default() -> Self {
        let mut m = HashMap::new();
        m.insert("root".to_string(), SubFlakish::default());
        Config(m)
    }
}

impl Config {
    pub fn from_flake_url(url: String) -> Result<Self> {
        nix::eval::nix_eval_attr_json::<Config>("nixci", url)
    }
}

/// Represents a sub-flake look-alike.
///
/// "Look-alike" because its inputs may be partial, thus requiring explicit
/// --override-inputs when evaluating the flake.
#[derive(Debug, Deserialize)]
pub struct SubFlakish {
    /// Subdirectory in which the flake lives
    pub dir: String,

    /// Inputs to override (via --override-input)
    #[serde(rename = "overrideInputs", default)]
    pub override_inputs: HashMap<String, String>,
}

impl Default for SubFlakish {
    /// The default `SubFlakish` is the root flake.
    fn default() -> Self {
        SubFlakish {
            dir: ".".to_string(),
            override_inputs: HashMap::default(),
        }
    }
}

impl SubFlakish {
    /// Return the `nix build` arguments for building all the outputs in this
    /// subflake configuration.
    pub fn nix_build_args_for_flake(&self, flake_url: String) -> Vec<String> {
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
