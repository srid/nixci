use std::str::FromStr;

use anyhow::Result;
use nix_rs::{
    command::{NixCmd, NixCmdError},
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
    pub async fn from_flake(cmd: &NixCmd, url: &SystemsListFlakeRef) -> Result<Self> {
        // Nix eval, and then return the systems
        let systems = nix_import_flake::<Vec<System>>(cmd, &url.0).await?;
        Ok(SystemsList(systems))
    }
}

pub async fn nix_import_flake<T>(cmd: &NixCmd, url: &FlakeUrl) -> Result<T, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    let flake_path =
        nix_eval_impure_expr::<String>(cmd, format!("builtins.getFlake \"{}\"", url.0)).await?;
    let v = nix_eval_impure_expr(cmd, format!("import {}", flake_path)).await?;
    Ok(v)
}

async fn nix_eval_impure_expr<T>(cmd: &NixCmd, expr: String) -> Result<T, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    let v = cmd
        .run_with_args_expecting_json::<T>(&["eval", "--impure", "--json", "--expr", &expr])
        .await?;
    Ok(v)
}

#[cfg(test)]
#[cfg(feature = "integration_test")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_systems_list() {
        let systems = SystemsList::from_flake(
            &NixCmd::default(),
            &SystemsListFlakeRef(FlakeUrl("github:nix-systems/empty".to_string())),
        )
        .await
        .unwrap();
        assert_eq!(systems.0, vec![]);
    }

    #[tokio::test]
    async fn test_systems_list() {
        let systems = SystemsList::from_flake(
            &NixCmd::default(),
            &SystemsListFlakeRef(FlakeUrl("github:nix-systems/default-darwin".to_string())),
        )
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
