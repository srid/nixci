/// Enough types to get branch info from Pull Request URL
use anyhow::{bail, Context};
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use try_guard::guard;
use url::{Host, Url};

/// A reference to a Github Pull Request
#[derive(Debug, Clone)]
pub struct PullRequestRef {
    owner: String,
    repo: String,
    pr: u64,
}

impl PullRequestRef {
    fn api_url(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            self.owner, self.repo, self.pr
        )
    }
    /// Parse a Github PR URL into its owner, repo, and PR number
    pub fn from_web_url(url: &String) -> Option<Self> {
        let url = Url::parse(url).ok()?;
        guard!(url.scheme() == "https" && url.host() == Some(Host::Domain("github.com")));
        let paths = url.path_segments().map(|c| c.collect::<Vec<_>>())?;
        match paths[..] {
            [user, repo, "pull", pr_] => {
                let pr = pr_.parse::<u64>().ok()?;
                Some(PullRequestRef {
                    owner: user.to_string(),
                    repo: repo.to_string(),
                    pr,
                })
            }
            _ => None,
        }
    }
}

/// Github Pull Request API Response type
#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub url: String,
    pub head: Head,
}

#[derive(Debug, Deserialize)]
pub struct Head {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub repo: Repo,
}

#[derive(Debug, Deserialize)]
pub struct Repo {
    /// `<owner>/<repo>`
    pub full_name: String,
}

impl PullRequest {
    /// Fetch the given PR using Github's API
    pub fn get(ref_: &PullRequestRef) -> anyhow::Result<Self> {
        api_get::<PullRequest>(ref_.api_url())
    }

    /// The flake URL referencing the branch of this PR
    pub fn flake_url(&self) -> String {
        format!("github:{}/{}", self.head.repo.full_name, self.head.ref_)
    }
}

/// Get an API response, parsing the response into the given type
fn api_get<T>(url: String) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&url)
        // Github API requires a user agent
        .header(USER_AGENT, "github.com/srid/nixci")
        .send()
        .with_context(|| format!("cannot create request: {}", &url))?;
    if resp.status().is_success() {
        let v = resp
            .json::<T>()
            .with_context(|| format!("cannot parse response: {}", &url))?;
        Ok(v)
    } else {
        bail!("cannot make request: {}", resp.status())
    }
}
