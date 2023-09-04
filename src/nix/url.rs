use serde::Deserialize;

/// A valid flake URL
///
/// See [syntax here](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html#url-like-syntax).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FlakeUrl(pub String);

/// The attribute output part of a [FlakeUrl]
///
/// Example: `foo` in `.#foo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlakeAttr(Option<String>);

impl FlakeUrl {
    /// Split the [FlakeAttr] out of the [FlakeUrl]
    pub fn split_attr(&self) -> (Self, FlakeAttr) {
        match self.0.split_once('#') {
            Some((url, attr)) => (FlakeUrl(url.to_string()), FlakeAttr(Some(attr.to_string()))),
            None => (self.clone(), FlakeAttr(None)),
        }
    }

    /// Return the flake URL pointing to the sub-flake
    pub fn sub_flake_url(&self, dir: String) -> FlakeUrl {
        if dir == "." {
            self.clone()
        } else {
            FlakeUrl(format!("{}?dir={}", self.0, dir))
        }
    }
}

impl FlakeAttr {
    /// Get the attribute name.
    ///
    /// If attribute exists, then return "default".
    pub fn get_name(&self) -> String {
        self.0.clone().unwrap_or_else(|| "default".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flake_url() {
        let url = FlakeUrl("github:srid/nixci".to_string());
        assert_eq!(url.split_attr(), (url.clone(), FlakeAttr(None)));

        let url = FlakeUrl("github:srid/nixci#extra-tests".to_string());
        assert_eq!(
            url.split_attr(),
            (
                FlakeUrl("github:srid/nixci".to_string()),
                FlakeAttr(Some("extra-tests".to_string()))
            )
        );
    }

    #[test]
    fn test_sub_flake_url() {
        let url = FlakeUrl("github:srid/nixci".to_string());
        assert_eq!(url.sub_flake_url(".".to_string()), url.clone());
        assert_eq!(
            url.sub_flake_url("dev".to_string()),
            FlakeUrl("github:srid/nixci?dir=dev".to_string())
        );
    }

    #[test]
    fn test_sub_flake_url_with_query() {
        let url = FlakeUrl("git+https://example.org/my/repo?ref=master".to_string());
        assert_eq!(url.sub_flake_url(".".to_string()), url.clone());
        assert_eq!(
            url.sub_flake_url("dev".to_string()),
            FlakeUrl("git+https://example.org/my/repo?ref=master&dir=dev".to_string())
        );
    }
}
