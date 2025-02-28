
use crate::{Config, sort};

use std::{
    fs,
    path::Path,
};

use github_issues_export_lib::{
    RepoLocation,
    auth::GithubAuth,
};
use regex::Regex;

pub fn parse_config(config: &str) -> Result<Config, ()> {
    let de: cfg_model::ConfigFile = serde_yaml::from_str(config)
        .map_err(|e| {
            eprintln!("could not parse config: {}", e);
        })?;
    de.cfg_parse()
}

pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Config, ()> {
    let contents = fs::read_to_string(path)
        .map_err(|e| {
            eprintln!("could not read config file: {}", e);
        })?;
    parse_config(&contents)
}

/// Model for the config file.
pub mod cfg_model {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ConfigFile {
        // env var where github auth token is stored
        pub auth_var: String,
        // in the user/repo notation
        pub repo: String,
        pub organize: Vec<OrganizeScopeElem>
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OrganizeScopeElem {
        pub filter: Option<String>,
        pub order: Option<Vec<String>>,
        pub target: Target,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Target {
        Bin(Bin),
        SubScope1(Box<OrganizeScopeElem>),
        SubScopeN(Vec<OrganizeScopeElem>),
    }

    #[derive(Debug, Clone, Default)]
    pub struct Bin;
    serde_string_literal!(Bin = "bin");
}

// ==== traits ====

pub trait ParseCfg<T>: Sized {
    fn parse_cfg(old: T) -> Result<Self, ()>;
}

impl<A, B: ParseCfg<A>> ParseCfg<Vec<A>> for Vec<B> {
    fn parse_cfg(old: Vec<A>) -> Result<Self, ()> {
        let mut new = Vec::with_capacity(old.len());
        for elem in old {
            new.push(elem.cfg_parse()?);
        }
        Ok(new)
    }
}

impl<A, B: ParseCfg<A>> ParseCfg<Option<A>> for Option<B> {
    fn parse_cfg(old: Option<A>) -> Result<Self, ()> {
        Ok(match old {
            Some(inner) => Some(inner.cfg_parse()?),
            None => None,
        })
    }
}

pub trait CfgParse<T>: Sized {
    fn cfg_parse(self) -> Result<T, ()>;
}

impl<A, B: ParseCfg<A>> CfgParse<B> for A {
    fn cfg_parse(self) -> Result<B, ()> {
        B::parse_cfg(self)
    }
}

// ==== impl ====

impl<S: AsRef<str>> ParseCfg<S> for RepoLocation {
    fn parse_cfg(old: S) -> Result<Self, ()> {
        let parts: Vec<&str> = old.as_ref().split("/").collect();
        match AsRef::<[&str]>::as_ref(&parts) {
            &[user, repo] => Ok(RepoLocation::new(user, repo)),
            _ => {
                eprintln!("[error] cannot parse repo location:\n{:?}", old.as_ref());
                Err(())
            }
        }
    }
}

impl<S: AsRef<str>> ParseCfg<S> for Regex {
    fn parse_cfg(old: S) -> Result<Self, ()> {
        Regex::new(old.as_ref())
            .map_err(|e| {
                eprintln!("[error] invalid regex ({:?}):\n{}", old.as_ref(), e);
            })
    }
}

impl ParseCfg<Vec<String>> for sort::PatternSequence {
    fn parse_cfg(old: Vec<String>) -> Result<Self, ()> {
        Ok(sort::PatternSequence {
            patterns: old.cfg_parse()?
        })
    }
}

impl ParseCfg<cfg_model::OrganizeScopeElem> for (sort::FilterSort, sort::SortTarget) {
    fn parse_cfg(old: cfg_model::OrganizeScopeElem) -> Result<Self, ()> {

        let filter: Regex = old.filter
            .map(String::as_str)
            .unwrap_or(".*")
            .cfg_parse()?;
        let sorter: Option<sort::PatternSequence> = old.order.cfg_parse()?;

        let sort_target: sort::SortTarget = match old.target {
            cfg_model::Target::Bin(cfg_model::Bin) => {
                sort::SortTarget::Bin
            }
            cfg_model::Target::SubScope1(elem) => {
                sort::SortTarget::Recurse(vec![elem].cfg_parse()?)
            },
            cfg_model::Target::SubScopeN(elem) => {
                sort::SortTarget::Recurse(elem.cfg_parse()?)
            }
        };

        Ok((sort::FilterSort { filter, sorter }, sort_target))
    }
}

impl ParseCfg<Vec<cfg_model::OrganizeScopeElem>> for sort::OrganizeScope {
    fn parse_cfg(old: Vec<cfg_model::OrganizeScopeElem>) -> Result<Self, ()> {
        Ok(sort::OrganizeScope {
            targets: old.cfg_parse()?
        })
    }
}

/*
impl ParseCfg<cfg_model::IssueSortInstr> for sort::FilterSort {
    fn parse_cfg(old: cfg_model::IssueSortInstr) -> Result<Self, ()> {
        Ok(sort::FilterSort {
            filter: old.regex.cfg_parse()?,
            sorter: old.order.cfg_parse()?,
        })
    }
}

impl ParseCfg<cfg_model::OrganizeInstr> for sort::OrganizeInstr {
    fn parse_cfg(old: cfg_model::OrganizeInstr) -> Result<Self, ()> {
        Ok(match old {
            cfg_model::OrganizeInstr::Bin(inner) =>
                sort::OrganizeInstr::Bin(inner.cfg_parse()?),
            cfg_model::OrganizeInstr::Sort(inner) =>
                sort::OrganizeInstr::Sort(inner.cfg_parse()?),
        })
    }
}

impl ParseCfg<cfg_model::ConfigFile> for Config {
    fn parse_cfg(old: cfg_model::ConfigFile) -> Result<Self, ()> {
        let auth = GithubAuth::from_env(&old.auth_var)
            .map_err(|e| {
                eprintln!("[error] cannot find github auth token: {}", e);
            })?;

        Ok(Config {
            auth,
            repo: old.repo.cfg_parse()?,
            organize: old.organize.cfg_parse()?,
        })
    }
}
*/

#[cfg(test)]
const TEST_CFG_YAML: &'static str = r#####"
repo: gretchenfrage/reflex
auth_var: GITHUB_TOKEN
organize:
    - bin:
        - "foo*.*"
        - "bar"
        - "bar"
        - "bar"
    - sort:
        - "dklfhjgkl"
        - "baz"
    - sort:
        - "zamboni!"
        "#####;

#[test]
fn cfg_serde_test() {
    let cfg: cfg_model::ConfigFile = serde_yaml::from_str(TEST_CFG_YAML).unwrap();
    println!("{:#?}", cfg);
    let yml = serde_yaml::to_string(&cfg).unwrap();
    println!("{}", yml);
}

#[test]
fn cfg_parse_test() {
    parse_config(TEST_CFG_YAML).unwrap();
}