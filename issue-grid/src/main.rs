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

    macro_rules! serde_as_list {
        (
        struct $struct:ident;
        $($t:tt)*
        )=>{
            impl serde::Serialize for $struct {
                fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeSeq;

                    let mut s_seq = s.serialize_seq(None)?;

                    serde_as_list!(@ser, $struct, s_seq, self, ($($t)*))
                }
            }

            impl<'de> serde::Deserialize<'de> for $struct {
                fn deserialize<D>(d: D) -> Result<Self, D::Error>
                where
                    D: serde::de::Deserializer<'de>
                {
                    use serde::de::{Visitor, SeqAccess, Error};
                    use std::fmt::{self, Formatter};

                    struct V;
                    impl<'de2> Visitor<'de2> for V {
                        type Value = $struct;

                        fn expecting(&self, f: &mut Formatter) -> fmt::Result {
                            f.write_str(concat!(
                                "sequence form of ",
                                stringify!($struct)
                            ))
                        }

                        fn visit_seq<A>(self, mut d_seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: SeqAccess<'de2>
                        {
                            serde_as_list!(@de, $struct, d_seq, self, ($struct {}), ($($t)*))
                        }
                    }

                    d.deserialize_seq(V)
                }
            }
        };

        // ====

        // ser field case
        (
        @ser, $struct:ty, $s_seq:expr, $self:expr,
        (
        field $field:ident;
        $($t:tt)*
        )
        )=>{{
            $s_seq.serialize_element(&$self.$field)?;

            // recurse
            serde_as_list!(@ser, $struct, $s_seq, $self, ($($t)*))
        }};

        // ser option_tail case
        (
        @ser, $struct:ty, $s_seq:expr, $self:expr,
        (
        option_tail $field:ident;
        $($t:tt)*
        )
        )=>{{
            if let Some(ref vec) = $self.$field {
                for elem in vec {
                    $s_seq.serialize_element(elem)?;
                }
            }

            // recurse into base case
            serde_as_list!(@assert_empty_parens, ($($t)*));
            serde_as_list!(@ser, $struct, $s_seq, $self, ($($t)*))
        }};

        // ser base case
        (
        @ser, $struct:ty, $s_seq:expr, $self:expr,
        ()
        )=>{{
            $s_seq.end()
        }};

        // ====

        // de field case
        (
        @de, $struct:ty, $d_seq:expr, $self:expr,
        (   // constructor accumulator
            $struct_cons:ident {
                $($t_cons:tt)*
            }
        ),
        (
        field $field:ident;
        $($t:tt)*
        )
        )=>{{
            let $field = $d_seq.next_element()?
                .ok_or_else(|| A::Error::custom(concat!(
                    stringify!($struct),
                    ".",
                    stringify!($field),
                )))?;

            // recurse
            serde_as_list!(
                @de, $struct, $d_seq, $self,
                (
                    $struct_cons {
                        $($t_cons)*
                        $field: $field,
                    }
                ),
                ($($t)*)
            )
        }};

        // de option_tail case
        (
        @de, $struct:ty, $d_seq:expr, $self:expr,
        (   // constructor accumulator
            $struct_cons:ident {
                $($t_cons:tt)*
            }
        ),
        (
        option_tail $field:ident;
        $($t:tt)*
        )
        )=>{{
            let mut tail = Vec::new();
            while let Some(elem) = $d_seq.next_element()? {
                tail.push(elem);
            }
            let $field = match tail.len() {
                0 => None,
                _ => Some(tail),
            };

            // recurse
            serde_as_list!(@assert_empty_parens, ($($t)*));
            serde_as_list!(
                @de, $struct, $d_seq, $self,
                (
                    $struct_cons {
                        $($t_cons)*
                        $field: $field,
                    }
                ),
                ($($t:tt)*)
            )
        }};

        // de base case
        (
        @de, $struct:ty, $d_seq:expr, $self:expr,
        (   // constructor accumulator
            $struct_cons:ident {
                $($t_cons:tt)*
            }
        ),
        ()
        )=>{{
            Ok($struct_cons {
                $($t_cons)*
            })
        }};

        // ====

        (@assert_empty_parens, ())=>{};
        (@deform, $($t:tt)*)=>{
            $($t)*
        };

    }


    serde_as_list! {
        struct IssueSortInstr;
        field regex;
        option_tail order;
    }

    //trace_macros!(false);


    /*
    impl serde::Serialize for IssueSortInstr {
        fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
        {
            use serde::ser::SerializeSeq;

            let mut s_seq = s.serialize_seq(None)?;

            s_seq.serialize_element(&self.regex)?;
            if let Some(ref vec) = &self.order {
                for elem in vec {
                    s_seq.serialize_element(elem)?;
                }
            }

            s_seq.end()
        }
    }

    impl<'de> serde::Deserialize<'de> for IssueSortInstr {
        fn deserialize<D>(d: D) -> Result<Self,D::Error>
        where
            D: serde::de::Deserializer<'de>
        {
            use serde::de::{Visitor, SeqAccess, Error};
            use std::fmt::{self, Formatter};

            struct V;
            impl<'de2> Visitor<'de2> for V {
                type Value = IssueSortInstr;

                fn expecting(&self, f: &mut Formatter) -> fmt::Result {
                    f.write_str(concat!(
                        "sequence form of ",
                        stringify!(IssueSortInstr)
                    ))
                }

                fn visit_seq<A>(self, mut d_seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de2>
                {
                    let regex = d_seq.next_element()?
                        .ok_or_else(|| A::Error::custom(concat!(
                            stringify!(IssueSortInst),
                            ".",
                            stringify!(regex),
                        )))?;

                    let mut tail = Vec::new();
                    while let Some(elem) = d_seq.next_element()? {
                        tail.push(elem);
                    }
                    let order = match tail.len() {
                        0 => None,
                        _ => Some(tail),
                    };

                    Ok(IssueSortInstr {
                        regex,
                        order,
                    })

                }
            }

            d.deserialize_seq(V)
        }
    }
    */
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

/*
impl RepoData {
    pub fn fetch
}
*/

#[get("/api/list_issues")]
fn list_issues(repo_lock: State<RepoMutex>) -> Resp<Vec<model::IssueSummary>> {
    let repo = repo_lock.read();

    resp(repo.issues.clone())
}


/*
fn fetch() -> Result<(), ()> {
    let auth = GithubAuth::from_env("GITHUB_TOKEN").unwrap();
    let (github, mut core) = Github::from_auth(auth).unwrap();
    let repo = RepoLocation::new("correlation-one", "c1");
    let issues = github.issues(&repo, IssueState::Open);
    /*
    let issues = issues
        .and_then({
            let github = github.clone();
            move |issue_vec| github.issue_comments(issue_vec)
        });
        */
    let issues = core.run(issues).unwrap();
    let issues: Vec<model::IssueSummary> = issues.gh_into();

    println!("{:#?}", issues);

    Ok(())
}
*/

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
