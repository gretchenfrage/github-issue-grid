
use super::{Conv, Remodel};
use crate::{
    sort::PatternList,
    model::Color,
};
use github_issues_export_lib::prelude::*;
use failure::{Error, format_err};
use regex::Regex;

pub mod fr {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Config {
        pub auth_var: String,
        pub repo: String,
        pub bins: Vec<BinConfig>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BinConfig {
        pub filter: String,
        pub order: Option<Vec<String>>,
        pub name: Option<String>,
        pub color: Option<String>,
        pub main_label: Option<String>,
    }
}
use crate::config as to;

remodel! {
    type Config remodels (T) -> (Result<T, Error>);

    (from: String) -> Regex {
        Regex::new(&from).map_err(Error::from)
    }

    (from: String) -> RepoLocation {
        let parts: Vec<&str> = from.split("/").collect();
        match parts.as_slice() {
            &[user, repo] => Ok(
                RepoLocation::new(user, repo)
            ),
            _ => Err(
                format_err!("invalid repository: {:?}", from)
            ),
        }
    }

    (from: Vec<String>) -> PatternList<()> {
        from.into_iter()
            .map(|pat| conv(pat)
                .map(|regex: Regex| (regex, ())))
            .collect::<Result<PatternList<()>, Error>>()
    }

    (from: fr::BinConfig) -> (Regex, to::BinConfig) {
        let fr::BinConfig {
            filter,
            order,
            name,
            color,
            main_label,
        } = from;

        let tuple = (
            conv(filter)?,
            to::BinConfig {
                name,
                main_label,
                color: color.map(Color),
                sort: order.map(conv).transpose()?,
            }
        );
        Ok(tuple)
    }

    (from: fr::Config) -> to::Config {
        let fr::Config {
            auth_var,
            repo,
            bins,
        } = from;

        let auth = GithubAuth::from_env(&auth_var)
            .map_err(|e| format_err!("{}", e))?;

        let bins = bins.into_iter()
            .map(conv)
            .collect::<Result<PatternList<to::BinConfig>, Error>>()?;

        let cfg = to::Config {
            auth,
            repo: conv(repo)?,
            bins,
        };
        Ok(cfg)
    }
}