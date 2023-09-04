use serde::Deserialize;

/// A valid flake URL
///
/// See [syntax here](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html#url-like-syntax).
#[derive(Debug, Clone, Deserialize)]
pub struct FlakeUrl(pub String);

/// The attribute output part of a [FlakeUrl]
///
/// Example: `foo` in `.#foo`.
#[derive(Debug, Clone)]
pub struct FlakeAttr(Option<String>);

impl FlakeUrl {
    /// Get the [FlakeAttr] pointed by this flake.
    pub fn get_attr(&self) -> FlakeAttr {
        self.split_attr().1
    }

    /// Get the url without the .# attribute part.
    pub fn without_attr(&self) -> FlakeUrl {
        self.split_attr().0
    }

    fn split_attr(&self) -> (Self, FlakeAttr) {
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
