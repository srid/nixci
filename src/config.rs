use std::collections::BTreeMap;

use anyhow::Result;
use nix_rs::flake::{eval::nix_eval_attr_json, system::System, url::FlakeUrl};
use serde::Deserialize;

use crate::cli::CliArgs;

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
        m.insert("<root>".to_string(), SubFlakish::default());
        Config(m)
    }
}

impl Config {
    /// Read a flake URL with config attr, and return the original flake url along with the config.
    pub async fn from_flake_url(
        url: &FlakeUrl,
    ) -> Result<(((String, Option<String>), Self), FlakeUrl)> {
        let (url, attr) = url.split_attr();
        let nested_attr = attr.as_list();
        let (name, sub_flake) = match nested_attr.as_slice() {
            [] => ("default", None),
            [name] => (name.as_str(), None),
            [name, sub_flake] => (name.as_str(), Some(sub_flake)),
            _ => anyhow::bail!("Invalid flake URL (too many nested attr): {}", url.0),
        };
        let nixci_url = FlakeUrl(format!("{}#nixci.{}", url.0, name));
        let cfg = nix_eval_attr_json::<Config>(&nixci_url, attr.is_none()).await?;
        if let Some(sub_flake_name) = sub_flake {
            if !cfg.0.contains_key(sub_flake_name) {
                anyhow::bail!(
                    "Sub-flake '{}' not found in nixci configuration '{}'",
                    sub_flake_name,
                    nixci_url
                )
            }
        }
        Ok((((name.to_string(), sub_flake.cloned()), cfg), url))
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
    // NB: we use BTreeMap instead of HashMap here so that we always iterate
    // inputs in a determinitstic (i.e. asciibetical) order
    #[serde(rename = "overrideInputs", default)]
    pub override_inputs: BTreeMap<String, FlakeUrl>,

    /// An optional whitelist of systems to build on (others are ignored)
    pub systems: Option<Vec<System>>,
}

impl Default for SubFlakish {
    /// The default `SubFlakish` is the root flake.
    fn default() -> Self {
        SubFlakish {
            dir: ".".to_string(),
            override_inputs: BTreeMap::default(),
            systems: None,
        }
    }
}

impl SubFlakish {
    pub fn can_build_on(&self, systems: &Vec<System>) -> bool {
        match self.systems.as_ref() {
            Some(systems_whitelist) => systems_whitelist.iter().any(|s| systems.contains(s)),
            None => true,
        }
    }

    /// Return the devour-flake `nix build` arguments for building all the outputs in this
    /// subflake configuration.
    pub fn nix_build_args_for_flake(
        &self,
        cli_args: &CliArgs,
        flake_url: &FlakeUrl,
    ) -> Vec<String> {
        std::iter::once(flake_url.sub_flake_url(self.dir.clone()).0)
            .chain(self.override_inputs.iter().flat_map(|(k, v)| {
                [
                    "--override-input".to_string(),
                    // We must prefix the input with "flake" because
                    // devour-flake uses that input name to refer to the user's
                    // flake.
                    format!("flake/{}", k),
                    v.0.to_string(),
                ]
            }))
            .chain([
                "--override-input".to_string(),
                "systems".to_string(),
                cli_args.build_systems.0.clone(),
            ])
            .chain(cli_args.extra_nix_build_args.iter().cloned())
            .collect()
    }
}
