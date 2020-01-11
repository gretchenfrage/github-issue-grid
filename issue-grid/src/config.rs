
use crate::{
    sort::PatternList,
    model::Color,
};
use std::{
    str::FromStr,
    path::Path,
    fs,
};
use github_issues_export_lib::prelude::*;
use failure::Error;

/// Server config.
#[derive(Debug, Clone)]
pub struct Config {
    pub auth: GithubAuth,
    pub repo: RepoLocation,
    pub bins: PatternList<BinConfig>,
}

#[derive(Debug, Clone)]
pub struct BinConfig {
    pub name: Option<String>,
    pub color: Option<Color>,
    pub sort: Option<PatternList<()>>,
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::remodel::{
            Conv,
            config::{
                fr::Config as ConfigModel,
                Config as ConfigRemodel,
            },
        };

        // 1. deserialize
        let model: ConfigModel = serde_yaml::from_str(s)?;

        // 2. convert
        let config: Config = ConfigRemodel::conv(model)?;

        Ok(config)
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        // 1. read
        let string = fs::read_to_string(path)?;

        // 2. parse
        Self::from_str(&string)
    }
}

#[cfg(test)]
static EXAMPLE_CFG_YAML: &str = r###"
auth_var: GITHUB_TOKEN
repo: gretchenfrage/reflex
bins:
  - filter: "^Type: Enhancement"
    name: "nenhancements"
  - filter: "^Type: SDLKfjhsdlkfh"
    order:
      - "A"
      - "B"
      - "C"
    name: awfulness
  - filter: "^.*$"
"###;

#[test]
fn cfg_parse_test() {
    let cfg = Config::from_str(EXAMPLE_CFG_YAML).unwrap();
    println!("{:#?}", cfg);
}