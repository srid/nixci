use anyhow::Result;
use nix_rs::{
    command::{NixCmd, NixCmdError},
    flake::{system::System, url::FlakeUrl},
};

pub struct SystemsList(pub Vec<System>);

impl SystemsList {
    pub async fn from_flake(url: &FlakeUrl) -> Result<Self> {
        // Nix eval, and then return the systems
        let systems = nix_import_flake::<Vec<System>>(url).await?;
        Ok(SystemsList(systems))
    }
}

pub async fn nix_import_flake<T>(url: &FlakeUrl) -> Result<T, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    let flake_path =
        nix_eval_impure_expr::<String>(format!("builtins.getFlake \"{}\"", url.0)).await?;
    let v = nix_eval_impure_expr(format!("import {}", flake_path)).await?;
    Ok(v)
}

async fn nix_eval_impure_expr<T>(expr: String) -> Result<T, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    let nix = NixCmd::default();
    let v = nix
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
        let systems = SystemsList::from_flake(&FlakeUrl("github:nix-systems/empty".to_string()))
            .await
            .unwrap();
        assert_eq!(systems.0, vec![]);
    }

    #[tokio::test]
    async fn test_systems_list() {
        let systems =
            SystemsList::from_flake(&FlakeUrl("github:nix-systems/default-darwin".to_string()))
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
