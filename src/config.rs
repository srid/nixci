use std::collections::{BTreeMap, HashMap};

use anyhow::Result;
use serde::Deserialize;

use crate::{cli::CliArgs, nix};

/// Rust type for the `nixci` flake output
///
/// Example `flake.nix` output this type expects:
/// ```nix
/// {
///   nixci.test = {
///     dir = "./test";
///     overrideInputs = { "mymod" = "."; };
///   };
/// }
// NB: we use BTreeMap instead of HashMap here so that we always iterate
// configs in a determinitstic (i.e. asciibetical) order
#[derive(Debug, Deserialize)]
pub struct Config(pub BTreeMap<String, SubFlakish>);

impl Default for Config {
    /// Default value contains a single entry for the root flake.
    fn default() -> Self {
        let mut m = BTreeMap::new();
        m.insert("root".to_string(), SubFlakish::default());
        Config(m)
    }
}

impl Config {
    pub fn from_flake_url(url: &str) -> Result<Self> {
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
    /// Return the flake URL pointing to the sub-flake
    pub fn sub_flake_url(&self, root_flake_url: &String) -> String {
        if self.dir == "." {
            root_flake_url.clone()
        } else {
            format!("{}?dir={}", root_flake_url, self.dir)
        }
    }

    /// Return the `nix build` arguments for building all the outputs in this
    /// subflake configuration.
    pub fn nix_build_args_for_flake(&self, cli_args: &CliArgs, flake_url: &String) -> Vec<String> {
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

        extra_args.insert(0, self.sub_flake_url(flake_url));
        extra_args.extend(cli_args.extra_nix_build_args.iter().cloned());
        extra_args
    }
}
