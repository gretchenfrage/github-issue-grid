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

use crate::{
    repo::{Repo, RepoMutex},
    config::Config,
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
pub fn resp<R: Serialize>(inner: R) -> Resp<R> {
    CacheResponse::NoCache(Json(inner))
}


#[get("/")]
fn root() -> Redirect {
    Redirect::to("/static/index.html")
}

#[get("/api/list_issues")]
fn list_issues(repo_lock: State<RepoMutex>) -> Resp<Vec<model::IssueSummary>> {
    let repo = repo_lock.read();

    resp(repo.issues.clone())
}

#[get("/api/bin_issues")]
fn bin_issues(repo_lock: State<RepoMutex>) -> Resp<Vec<Vec<model::IssueSummary>>> {
    let repo = repo_lock.read();

    resp(repo.issue_bins.clone())
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
        .mount("/static", StaticFiles::from(base.join("static")))
        .mount("/", routes!(
            root,
            list_issues,
            bin_issues,
        ))
        .launch();
    Err(Error::from(err))
}

fn main() {
    try_main().unwrap();
}
