use std::collections::BTreeMap;

use anyhow::Result;
use nix_rs::flake::{eval::nix_eval_attr_json, system::System, url::FlakeUrl};
use serde::Deserialize;

use crate::cli::CliArgs;

/// The `nixci` configuration encoded in flake.nix
///
/// Example flake.nix:
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
pub struct Config {
    /// The flake.nix configuration
    pub subflakes: BTreeMap<String, SubFlakish>,

    /// The URL to the flake containing this configuration
    pub flake_url: FlakeUrl,

    /// Configuration name (nixci.<name>)
    pub name: String,

    /// Selected sub-flake if any.
    ///
    /// Must be a ke in `subflakes`.
    pub selected_subflake: Option<String>,
}

impl Config {
    /// Read a flake URL with config attr, and return the original flake url along with the config.
    pub async fn from_flake_url(url: &FlakeUrl) -> Result<Config> {
        let (flake_url, attr) = url.split_attr();
        let nested_attr = attr.as_list();
        let (name, selected_subflake) = match nested_attr.as_slice() {
            [] => ("default".to_string(), None),
            [name] => (name.clone(), None),
            [name, sub_flake] => (name.clone(), Some(sub_flake.to_string())),
            _ => anyhow::bail!("Invalid flake URL (too many nested attr): {}", flake_url.0),
        };
        let nixci_url = FlakeUrl(format!("{}#nixci.{}", flake_url.0, name));
        let subflakes =
            nix_eval_attr_json::<BTreeMap<String, SubFlakish>>(&nixci_url, attr.is_none()).await?;
        if let Some(sub_flake_name) = selected_subflake.clone() {
            if !subflakes.contains_key(&sub_flake_name) {
                anyhow::bail!(
                    "Sub-flake '{}' not found in nixci configuration '{}'",
                    sub_flake_name,
                    nixci_url
                )
            }
        }
        let cfg = Config {
            subflakes,
            flake_url,
            name,
            selected_subflake,
        };
        Ok(cfg)
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
