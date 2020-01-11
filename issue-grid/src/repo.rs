
use crate::{
    model::{IssueSummary, BinSummary},
    config::{Config, BinConfig},
    remodel::{
        Conv,
        github::Github as GithubRemodel,
    }
};
use github_issues_export_lib::{Github, IssueState};
use std::{
    sync::RwLock,
    ops::{Deref, DerefMut},
    borrow::Cow,
};
use failure::{Error, format_err};

/// Mutable global repo data.
pub struct Repo {
    pub issues: Vec<IssueSummary>,
    pub issue_bins: Vec<BinSummary>,
}

impl Repo {
    /// This is a blocking function.
    pub fn fetch(config: &Config) -> Result<Self, Error> {
        // fetch issues
        let (
            github,
            mut core
        ) = Github::from_auth(config.auth.clone())
            .map_err(|e| format_err!("{}", e))?;
        let issues = github
            .issues(&config.repo, IssueState::Open);
        let issues = core.run(issues)
            .map_err(|e| format_err!("{}", e))?;

        // remodel
        let issues: Vec<IssueSummary> = GithubRemodel::conv(issues);

        // bin
        let issue_bins = config.bins.bin(issues.clone(), true)
            .into_iter()
            .map(|(issues, bin_cfg)| {
                // 1. sort
                let sorter =
                    bin_cfg.and_then(|bin_cfg| bin_cfg.sort.as_ref());
                let issues = match sorter  {
                    Some(pat_list) => pat_list.sort(issues),
                    None => issues,
                };

                // 2. generate model
                match bin_cfg {
                    Some(bin_cfg) => BinSummary {
                        name: bin_cfg.name.clone(),
                        color: bin_cfg.color.clone(),
                        issues,
                        is_overflow: false,
                    },
                    None => BinSummary {
                        name: None,
                        color: None,
                        issues,
                        is_overflow: true,
                    },
                }
            })
            .collect();

        // done
        Ok(Repo {
            issues,
            issue_bins,
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
