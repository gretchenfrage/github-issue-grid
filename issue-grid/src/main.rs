#![feature(proc_macro_hygiene, decl_macro)]
#![feature(trace_macros)]
#![feature(vec_remove_item)]

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

use crate::{
    repo::{Repo, RepoMutex, FetchLock},
    config::Config,
    model::{IssueSummary, BinSummary, ProfileMeta},
};
use std::{
    env,
    path::PathBuf,
};
use serde::{
    Serialize,
};
use rocket::{
    State,
    response::{
        Redirect,
        status::NotFound,
    },
};
use rocket_contrib::{
    json::Json,
    serve::StaticFiles,
};
use rocket_cache_response::CacheResponse;
use failure::Error;

/// Serde utility macros.
#[macro_use]
pub mod serde_util;

/// Server config file.
pub mod config;

/// This server's HTTP resource model.
pub mod model;

/// Conversions between resource models.
pub mod remodel;

/// Issue sorting.
pub mod sort;

/// Data cache of the github repo.
pub mod repo;


/// No-cache JSON response wrapper type.
pub type Resp<R> = CacheResponse<Json<R>>;

/// No-cache JSON response wrapper function.
pub fn resp_wrap<R: Serialize>(inner: R) -> Resp<R> {
    CacheResponse::NoCache(Json(inner))
}

#[get("/")]
fn root() -> Redirect {
    Redirect::to("/static/index.html")
}

#[get("/api/repo_name")]
fn repo_name(config: State<Config>) -> Resp<String> {
    resp_wrap(config.repo.to_string())
}

#[get("/api/repo_hyperlink")]
fn repo_hyperlink(config: State<Config>) -> Resp<String> {
    let url = format!(
        "https://github.com/{}/{}",
        config.repo.user,
        config.repo.repo
    );
    resp_wrap(url)
}

#[get("/api/list_issues")]
fn list_issues(repo_lock: State<RepoMutex>) -> Resp<Vec<IssueSummary>> {
    let repo = repo_lock.read();
    resp_wrap(repo.issues.clone())
}

#[get("/api/list_profiles")]
fn list_profiles(config: State<Config>) -> Resp<Vec<ProfileMeta>> {
    let list = config.profiles.iter()
        .enumerate()
        .map(|(i, profile)| ProfileMeta {
            name: profile.name.clone(),
            number: i,
        })
        .collect();
    resp_wrap(list)
}

#[get("/api/<profile_name>/bin_issues?<remove_main_labels>")]
fn bin_issues(
    config: State<Config>,
    repo_lock: State<RepoMutex>,
    fetch_lock: State<FetchLock>,
    profile_name: String,
    remove_main_labels: bool,
) -> Result<Resp<Vec<BinSummary>>, NotFound<String>> {
    /*
    let (guard, first) = fetch_lock.acquire();
    if first {
        match Repo::fetch(&config) {
            Ok(repo) => {
                let mut store = repo_lock.write();
                *store = repo;
            },
            Err(e) => {
                eprintln!("[ERROR] fetch repo: {}", e);
            },
        }
    }
    drop(guard);
    */

    let repo = repo_lock.read();

    // resolve
    let profile_index = config.profiles.iter()
        .enumerate()
        .find(|&(_, profile)| profile.name == profile_name)
        .map(|(i, _)| i)
        .ok_or_else(||
            NotFound(format!("profile not found: {:?}", profile_name)))?;

    let mut bins = repo.repo_profiles[profile_index].issue_bins.clone();

    // process query params
    if remove_main_labels {
        for bin in &mut bins {
            if let Some(main_label) = bin.main_label.clone() {
                for issue in &mut bin.issues {
                    issue.labels.remove_item(&main_label);
                }
            }
        }
    }

    Ok(resp_wrap(bins))
}

fn try_main() -> Result<!, Error> {
    let base = env::var("CARGO_MANIFEST_DIR").ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let config = Config::from_file(base.join("config.yaml"))?;
    let repo = Repo::fetch(&config)?;
    let repo_lock = RepoMutex::new(repo);

    let err = rocket::ignite()
        .manage(config)
        .manage(repo_lock)
        .manage(FetchLock::new())
        .mount("/static", StaticFiles::from(base.join("static")))
        .mount("/", routes!(
            root,
            repo_name,
            repo_hyperlink,
            list_issues,
            list_profiles,
            bin_issues,
        ))
        .launch();
    Err(Error::from(err))
}

fn main() {
    try_main().unwrap();
}
