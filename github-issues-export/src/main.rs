
extern crate github_issues_export_lib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate futures;

use github_issues_export_lib::{
    prelude::*,
    error::*,
};

use std::{
    path::PathBuf,
    fs::File,
    io::Write,
};

use futures::{
    prelude::*,
    future::Either,
};
use docopt::Docopt;

/// Main function wrapper.
fn main() {
    if let Err(ref e) = run() {
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "Error: {}", e).expect(errmsg);
        for e in e.iter().skip(1) {
            writeln!(stderr, "Caused by: {}", e).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

/// Main function meat.
fn run() -> Result<()> {
    // parse
    let args = parse_args();
    let (
        github,
        mut core
    ) = Github::from_auth(args.env_token.clone())?;
    let repo = RepoLocation::new(&args.arg_username, &args.arg_repo);

    // fetch issues
    let issues = match args.arg_issue {
        Some(issue_number) => Either::A(
            github.issue(&repo, issue_number)
                .map(|issue| vec![issue])
        ),
        None => Either::B(
            github.issues(&repo, args.flag_state)
        ),
    };
    let issues = issues
        .and_then(|issue_vec| github.issue_comments(issue_vec));
    let issues = core.run(issues)?;

    // render and save
    let render = IssueRenderer::new();

    mkdir(&args.flag_path)?;
    for issue in &issues {
        let (md, path) = render.render_md(issue)?;
        let path = PathBuf::from(&args.flag_path).join(path);

        let mut f = File::create(&path)?;
        println!("Writing name {}", path.to_str().unwrap());
        f.write_all(md.as_bytes())?;
    }

    // done
    Ok(())
}

/// CLI usage string.
const USAGE: &'static str = r#"
Export issues from GitHub into markdown files.

Usage:
  github-issues-export [options] <query>
  github-issues-export (-h | --help)
  github-issues-export --version

<query> is of the form: username/repo[#issue_number].

Environment variables:
  GITHUB_TOKEN      Authorization token for GitHub.

Options:
  -h --help                         Show this screen.
  --version                         Show version.
  -p --path=<directory>             Output directory [default: ./md].
  -s --state=<open|closed|all>      Fetch issues that are open, closed, or
                                    both [default: open].
"#;

/// CLI arguments.
#[derive(Debug, Deserialize)]
struct Args {
    flag_version: bool,
    #[serde(skip)]
    env_token: String,
    arg_query: String,
    #[serde(skip)]
    arg_username: String,
    #[serde(skip)]
    arg_repo: String,
    #[serde(skip)]
    arg_issue: Option<usize>,
    flag_path: String,
    flag_state: IssueState,
}

/// Parse CLI arguments.
fn parse_args() -> Args {
    let mut args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    args.env_token = std::env::var("GITHUB_TOKEN").unwrap_or_else(|_| {
        eprintln!("Missing obligatory environment variable GITHUB_TOKEN");
        std::process::exit(1);
    });

    {
        let parts: Vec<_> = args.arg_query.split("/").collect();
        if parts.len() != 2 {
            eprintln!("Wrong argument: {}.\n\n{}", args.arg_query, USAGE);
            std::process::exit(1);
        }
        args.arg_username = String::from(parts[0]);
        let parts: Vec<_> = parts[1].split("#").collect();
        if parts.len() == 1 {
            args.arg_repo = String::from(parts[0]);
        } else if parts.len() == 2 {
            args.arg_repo = String::from(parts[0]);
            args.arg_issue = match parts[1].parse::<usize>() {
                Ok(value) => Some(value),
                Err(_) => {
                    eprintln!("Wrong argument: {}.\n\n{}", args.arg_query, USAGE);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Wrong argument: {}.\n\n{}", args.arg_query, USAGE);
            std::process::exit(1);
        }
    }

    args
}

/// Private helper function.
fn mkdir(path: &str) -> Result<()> {
    if let Err(err) = std::fs::create_dir(path) {
        match err.kind() {
            std::io::ErrorKind::AlreadyExists => (),
            _ => {
                return Err(Error::from(err));
            }
        }
    };
    Ok(())
}
