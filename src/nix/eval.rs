use nix_rs::command::{CommandError, NixCmd, NixCmdError};

use super::url::FlakeUrl;

/// Run 'nix eval <url> --json` and parse its JSON
///
/// If the flake does not output the given attribute, use the `Default`
/// implementation of `T`.
pub async fn nix_eval_attr_json<T>(url: FlakeUrl) -> Result<T, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    let cmd = NixCmd::default();
    match cmd
        .run_with_args_expecting_json(&["eval", url.0.as_str(), "--json"])
        .await
    {
        Ok(v) => Ok(v),
        Err(err) => {
            if error_is_missing_attribute(&err) {
                // The 'nixci' flake output attr is missing. User wants the default config.
                Ok(T::default())
            } else {
                Err(err)
            }
        }
    }
}

fn error_is_missing_attribute(err: &NixCmdError) -> bool {
    match err {
        NixCmdError::CmdError(CommandError::ProcessFailed { stderr, .. }) => {
            stderr.contains("does not provide attribute")
        }
        _ => false,
    }
}
