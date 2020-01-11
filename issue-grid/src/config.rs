
use github_issues_export_lib::prelude::*;

/// Server config.
pub struct Config {
    pub auth: GithubAuth,
    pub repo: RepoLocation,
}

impl Config {
    pub fn new() -> Self {
        let auth = GithubAuth::from_env("GITHUB_TOKEN").unwrap();
        let repo = RepoLocation::new("gretchenfrage", "reflex");

        Config {
            auth,
            repo,
        }
    }
}
