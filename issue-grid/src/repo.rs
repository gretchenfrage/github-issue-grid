
use crate::{
    model::IssueSummary,
    config::Config,
};
use github_issues_export_lib::{Github, IssueState};
use std::{
    sync::RwLock,
    ops::{Deref, DerefMut},
};

/// Mutable global repo data.
pub struct Repo {
    pub issues: Vec<IssueSummary>,
}

impl Repo {
    /// This is a blocking function.
    pub fn fetch(config: &Config) -> Result<Self, ()> {
        // fetch
        let (github, mut core) = Github::from_auth(config.auth.clone())
            .unwrap();
        let issues = github
            .issues(&config.repo, IssueState::Open);
        let issues = core.run(issues)
            .unwrap();

        // remodel
        let issues: Vec<IssueSummary> = unimplemented!(); // TODO

        // done
        Ok(Repo {
            issues
        })
    }
}

// == repo mutex ==

/// Convenience wrapper.
pub struct RepoMutex(RwLock<Repo>);

impl RepoMutex {
    pub fn new(data: Repo) -> Self {
        RepoMutex(RwLock::new(data))
    }

    pub fn read<'a>(&'a self) -> impl Deref<Target=Repo> + 'a {
        self.0.read().unwrap()
    }

    pub fn write<'a>(&'a self) -> impl Deref<Target=Repo> + DerefMut + 'a {
        self.0.write().unwrap()
    }
}
