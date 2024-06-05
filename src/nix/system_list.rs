use std::str::FromStr;

use crate::cli::BuildConfig;
use anyhow::Result;
use nix_rs::{
    command::NixCmdError,
    flake::{system::System, url::FlakeUrl},
};

/// A flake URL that references a list of systems ([SystemsList])
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemsListFlakeRef(pub FlakeUrl);

impl FromStr for SystemsListFlakeRef {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<SystemsListFlakeRef, String> {
        // Systems lists recognized by `github:nix-system/*`
        let known_nix_systems = [
            "aarch64-darwin",
            "aarch64-linux",
            "x86_64-darwin",
            "x86_64-linux",
        ];
        let url = if known_nix_systems.contains(&s) {
            format!("github:nix-systems/{}", s)
        } else {
            s.to_string()
        };
        Ok(SystemsListFlakeRef(FlakeUrl(url)))
    }
}

pub struct SystemsList(pub Vec<System>);

impl SystemsList {
    pub async fn from_flake(url: &SystemsListFlakeRef, option: &Vec<String>) -> Result<Self> {
        // Nix eval, and then return the systems
        let systems = nix_import_flake::<Vec<System>>(&url.0, option).await?;
        Ok(SystemsList(systems))
    }
}

pub async fn nix_import_flake<T>(url: &FlakeUrl, option: &Vec<String>) -> Result<T, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    let flake_path =
        nix_eval_impure_expr::<String>(format!("builtins.getFlake \"{}\"", url.0), option).await?;
    let v = nix_eval_impure_expr(format!("import {}", flake_path), option).await?;
    Ok(v)
}

async fn nix_eval_impure_expr<T>(expr: String, option: &Vec<String>) -> Result<T, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    // Base arguments for the nix command
    let base_args = vec![
        "eval".to_string(),
        "--impure".to_string(),
        "--json".to_string(),
        "--expr".to_string(),
        expr,
    ];

    // Combined arguments
    let mut combined_args: Vec<String> = base_args;

    // Conditionally append "--option" and option values
    if !option.is_empty() {
        combined_args.push("--option".to_string());
        combined_args.extend(option.clone());
    }

    let args_slice: Vec<&str> = combined_args.iter().map(AsRef::as_ref).collect();

    let nix = crate::nixcmd().await;
    let v = nix.run_with_args_expecting_json::<T>(&args_slice).await?;
    Ok(v)
}

#[cfg(test)]
#[cfg(feature = "integration_test")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_systems_list() {
        let systems = SystemsList::from_flake(&SystemsListFlakeRef(FlakeUrl(
            "github:nix-systems/empty".to_string(),
        )))
        .await
        .unwrap();
        assert_eq!(systems.0, vec![]);
    }

    #[tokio::test]
    async fn test_systems_list() {
        let systems = SystemsList::from_flake(&SystemsListFlakeRef(FlakeUrl(
            "github:nix-systems/default-darwin".to_string(),
        )))
        .await
        .unwrap();
        assert_eq!(
            systems.0,
            vec![
                System::Darwin(nix_rs::flake::system::Arch::Aarch64),
                System::Darwin(nix_rs::flake::system::Arch::X86_64)
            ]
        );
    }
}
