
use crate::{
    model::{IssueSummary, Label, BinSummary},
    config::{Config, Profile},
    remodel::{
        Conv,
        github::Github as GithubRemodel,
    }
};
use github_issues_export_lib::{Github, IssueState, model as gh};
use std::{
    sync::RwLock,
    ops::{Deref, DerefMut},
    collections::HashMap,
};
use failure::{Error, format_err};
use futures::{
    prelude::*,
    stream::iter_ok,
};

/// Mutable global repo data.
pub struct Repo {
    pub issues: Vec<IssueSummary>,
    pub repo_profiles: Vec<RepoProfile>,
}

/// Repo data for a specific profile.
pub struct RepoProfile {
    pub issue_bins: Vec<BinSummary>,
}

fn find_label<'a, I>(issues: I, name: &str) -> Option<Label>
where
    I: IntoIterator<Item=&'a IssueSummary>,
{
    issues.into_iter()
        .flat_map(|issue| issue.labels.iter())
        .find(|label| label.name == name)
        .cloned()
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

        // fetch usernames
        let user_details = issues.iter()
            .map(|issue| issue.user.login.clone())
            .map(|login| {
                github.user_details(&login)
                    .map(move |details| (login, details))
            });
        let user_details = iter_ok(user_details)
            .buffer_unordered(16)
            .collect();
        let user_details = core.run(user_details)
            .map_err(|e| format_err!("{}", e))?;
        let user_details: HashMap<String, gh::UserDetails> = user_details
            .into_iter()
            .collect();

        // remodel
        let issues: Vec<IssueSummary> = issues.into_iter()
            .map(|issue|
                GithubRemodel::conv((issue, &user_details)))
            .collect();

        // build profiles
        let repo_profiles = config.profiles.iter()
            .map(|profile| RepoProfile::build(
                &config,
                profile,
                issues.as_slice(),
            ))
            .collect();

        // done
        Ok(Repo {
            issues,
            repo_profiles,
        })
    }
}

impl RepoProfile {
    fn build(_config: &Config, profile: &Profile, issues: &[IssueSummary]) -> Self {
        // bin
        let issue_bins = profile.bins.bin(issues.to_vec(), true)
            .into_iter()
            .map(|(bin_issues, bin_cfg)| {
                // 1. sort
                let sorter =
                    bin_cfg.and_then(|bin_cfg| bin_cfg.sort.as_ref());
                let bin_issues = match sorter  {
                    Some(pat_list) => pat_list.sort(bin_issues),
                    None => bin_issues,
                };

                // 2. generate model
                match bin_cfg {
                    Some(bin_cfg) => {
                        let main_label = bin_cfg.main_label.as_ref()
                            .and_then(|label| find_label(issues, label));

                        BinSummary {
                            name: bin_cfg.name.clone(),
                            color: bin_cfg.color.clone(),
                            main_label,
                            issues: bin_issues,
                            is_overflow: false,
                        }
                    },
                    None => BinSummary {
                        name: None,
                        color: None,
                        main_label: None,
                        issues: bin_issues,
                        is_overflow: true,
                    },
                }
            })
            .collect();

        RepoProfile { issue_bins }
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
