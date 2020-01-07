#![feature(proc_macro_hygiene, decl_macro)]
#![feature(trace_macros)]

extern crate github_issues_export_lib;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_cache_response;
extern crate futures;
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate regex;

use crate::remodel::{GithubInto};

use std::{
    env,
    path::PathBuf,
    sync::RwLock,
    ops::{Deref, DerefMut},
};

use github_issues_export_lib::prelude::*;

use failure::Error;
use futures::prelude::*;
use serde::{
    Serialize,
};
use rocket::{
    State,
    response::{
        Redirect,
    },
};
use rocket_contrib::{
    json::Json,
    serve::StaticFiles,
};
use rocket_cache_response::CacheResponse;

/// Conversions between HTTP resource models.
pub mod remodel;

/// Serde utility macro.
#[macro_use]
pub mod serde_util;

#[get("/")]
fn root() -> Redirect {
    Redirect::to("/static/index.html")
}

/// No-cache JSON response wrapper type.
pub type Resp<R> = CacheResponse<Json<R>>;

/// No-cache JSON response wrapper function.
pub fn resp<R: Serialize>(inner: R) -> Resp<R> {
    CacheResponse::NoCache(Json(inner))
}

/// Server config.
pub struct Config {
    pub auth: GithubAuth,
    pub repo: RepoLocation,
}

/// Model for the config file.
mod cfg_model {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ConfigFile {
        // in the user/repo notation
        pub repo: String,
        pub organize: Vec<OrganizeInstr>
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum OrganizeInstr {
        #[serde(rename = "bin")]
        Bin(IssueSortInstr),
        #[serde(rename = "sort")]
        Sort(IssueSortInstr),
    }

    #[derive(Debug, Clone)]
    pub struct IssueSortInstr {
        regex: String,
        order: Option<Vec<String>>,
    }

    serde_as_list! {
        struct IssueSortInstr;
        field regex;
        option_tail order;
    }
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

/// Mutable global repo data.
pub struct Repo {
    pub issues: Vec<model::IssueSummary>,
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
        let issues: Vec<model::IssueSummary> = issues.gh_into();

        // done
        Ok(Repo {
            issues
        })
    }
}

#[get("/api/list_issues")]
fn list_issues(repo_lock: State<RepoMutex>) -> Resp<Vec<model::IssueSummary>> {
    let repo = repo_lock.read();

    resp(repo.issues.clone())
}

fn main() {


    let yaml = r#####"
repo: "gretchenfrage/reflex"
organize:
    - bin:
        - "foo**(*&)("
        - "bar"
        - "bar"
        - "bar"
    - sort:
        - "dklfhjgkl"
        - "baz"
    - sort:
        - "zamboni!"
        "#####;

    let cfg = serde_yaml::from_str::<cfg_model::ConfigFile>(yaml).unwrap();
    println!("{:#?}", cfg);
    let yml = serde_yaml::to_string(&cfg).unwrap();
    println!("{}", yml);
    return;



    let config = Config::new();
    let repo = Repo::fetch(&config).unwrap();
    let repo_lock = RepoMutex::new(repo);

    let path = env::var("CARGO_MANIFEST_DIR").ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("static");

    rocket::ignite()
        .manage(config)
        .manage(repo_lock)
        .mount("/static", StaticFiles::from(path))
        .mount("/", routes!(
            root,
            list_issues,
        ))
        .launch();

    //fetch().unwrap();

    /*
    let path = env::var("CARGO_MANIFEST_DIR").ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("static");

    rocket::ignite()
        .mount("/static", StaticFiles::from(path))
        .mount("/", routes!(root))
        .launch();
        */
}

/// HTTP resource model.
pub mod model {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct IssueSummary {
        pub id: u64,
        pub hyperlink: String,
        pub title: String,
        pub labels: Vec<Label>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Label {
        pub name: String,
        pub color: Color,
    }

    // == re-usable models ==

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct User {
        pub id: u64,
        pub name: String,
        pub icon_url: String,
        pub hyperlink: String,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Color(
        // valid CSS color, includes the pound.
        pub String
    );
}
