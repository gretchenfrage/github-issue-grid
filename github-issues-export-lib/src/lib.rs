#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate handlebars;
extern crate hyper;
extern crate hyper_tls;
extern crate native_tls;
#[macro_use]
extern crate redacted_debug;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate slug;
extern crate docopt;
extern crate markdown;

pub extern crate tokio_core;

use crate::{
    error::*,
    auth::GithubAuth,
};

use std::{
    env,
    fmt::{self, Display, Formatter},
    str::FromStr,
    sync::Arc,
};
use futures::{
    {Future, Stream},
    future,
};
use hyper::{
    {Client, Method, Request, Uri},
    header::{
        Authorization,
        ContentLength,
        ContentType,
        UserAgent,
    },
};

pub use tokio_core::reactor::Core as TokioCore;

/// Github resource data model.
pub mod model;

/// Error types for this crate.
pub mod error;

/// Github auth token handling.
pub mod auth;

/// Rendering issues to markdown.
pub mod render;

pub mod prelude {
    pub use crate::{
        Github,
        IssueState,
        RepoLocation,
        render::IssueRenderer,
        auth::GithubAuth,
        model as gh_model,
        error as gh_error,
    };
    pub use tokio_core::reactor::Core as TokioCore;
}

/// Github access service.
#[derive(Clone)]
pub struct Github {
    client: Arc<Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>>,
    user_agent: UserAgent,
    token: Authorization<String>,
}

const GITHUB_API_ENDPOINT: &'static str = "https://api.github.com";

impl Github {
    /// Low level constructor. Consider using `from_auth`.
    pub fn new(
        handle: &tokio_core::reactor::Handle,
        agent: UserAgent,
        token: &str,
    ) -> Result<Self> {
        let client = hyper::Client::configure()
            .connector(hyper_tls::HttpsConnector::new(4, handle)?)
            .build(handle);
        Ok(Self {
            client: Arc::new(client),
            user_agent: agent,
            token: Authorization(format!("token {}", token)),
        })
    }

    /// High level constructor.
    ///
    /// Creates and returns a tokio core.
    pub fn from_auth<A>(auth: A) -> Result<(Self, TokioCore)>
        where
            A: Into<GithubAuth>
    {
        // resolve user agent at compile time
        let user_agent = UserAgent::new(concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
        ));

        // create the tokio reactor core
        let tokio_core = TokioCore::new()?;

        // delegate to low-level constructor
        Github::new(
            &tokio_core.handle(),
            user_agent,
            &auth.into().token,
        ).map(move |github|
            (github, tokio_core)
        )
    }

    /// GET request, retrieve and parse.
    ///
    /// Other methods exist as typed helpers.
    pub fn get<T>(&self, endpoint: &str) -> impl Future<Item=T, Error=Error>
        where
            T: serde::de::DeserializeOwned,
    {
        let url = Uri::from_str(endpoint).expect("Could not parse uri");
        let mut req = Request::new(Method::Get, url);
        req.headers_mut().set(self.user_agent.clone());
        req.headers_mut().set(self.token.clone());
        req.headers_mut().set(ContentType::json());
        req.headers_mut().set(ContentLength(0));
        let resp = self.client.request(req);
        resp.map_err(Error::from).and_then(|resp| {
            let status_code = resp.status();
            let body = resp.body().concat2().from_err();
            body.and_then(move |chunk| {
                if !status_code.is_success() {
                    let resp = String::from(::std::str::from_utf8(&chunk)?);
                    Err(ErrorKind::Request(resp).into())
                } else {
                    let value: T = ::serde_json::from_slice(&chunk)
                        .chain_err(|| "Could not parse response from server")?;
                    Ok(value)
                }
            })
        })
    }

    /// GET a github issue.
    pub fn issue(
        &self,
        repo: &RepoLocation,
        number: usize,
    ) -> impl Future<Item=model::Issue, Error=Error> {
        self.get(&format!(
            "{}/repos/{owner}/{repo}/issues/{number}",
            GITHUB_API_ENDPOINT,
            owner = repo.user,
            repo = repo.repo,
            number = number
        ))
    }

    /// GET all github issues in a repo.
    pub fn issues(
        &self,
        repo: &RepoLocation,
        issue_state: IssueState,
    ) -> impl Future<Item=Vec<model::Issue>, Error=Error> {
        self.get(&format!(
            "{}/repos/{}/{}/issues?state={}",
            GITHUB_API_ENDPOINT,
            &repo.user,
            &repo.repo,
            issue_state
        ))
    }

    /// Given a vector of issues already fetched from a repository,
    /// fetch their comments.
    pub fn issue_comments(
        &self,
        issues: Vec<model::Issue>,
    ) -> impl Future<Item=Vec<model::IssueWithComments>, Error=Error> {
        let github = self.clone();

        future::join_all({
            issues.into_iter()
                .map(move |issue| {
                    let get_comment = github
                        .get::<Vec<model::Comment>>(&issue.comments_url);
                    Future::join(
                        future::ok(issue),
                        get_comment,
                    )
                })
        })
            .map(|issues| issues
                .into_iter()
                .map(|(issue, comments)| model::IssueWithComments {
                    issue,
                    comments,
                })
                .collect::<Vec<model::IssueWithComments>>()
            )
    }

    pub fn user_details(
        &self,
        login: &str
    ) -> impl Future<Item=model::UserDetails, Error=Error> {
        self.get(&format!("{}/users/{}", GITHUB_API_ENDPOINT, login))
    }
}

/// Possible states to fetch issues by.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub enum IssueState {
    Open,
    Closed,
    All,
}

/// Identifying information for a github repository.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct RepoLocation {
    pub user: String,
    pub repo: String,
}

impl RepoLocation {
    pub fn new<S1, S2>(user: S1, repo: S2) -> Self
        where
            S1: ToString,
            S2: ToString,
    {
        RepoLocation {
            user: user.to_string(),
            repo: repo.to_string(),
        }
    }
}

impl IssueState {
    fn to_str(&self) -> &'static str {
        match *self {
            IssueState::Open => "open",
            IssueState::Closed => "closed",
            IssueState::All => "all",
        }
    }
}

impl Display for IssueState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}
