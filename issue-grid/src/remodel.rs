
use crate::model;
use github_issues_export_lib::model as gh;

// ==== traits ====

pub trait FromGithub<T>: Sized {
    fn from_gh(old: T) -> Self;
}

impl<A, B: FromGithub<A>> FromGithub<Vec<A>> for Vec<B> {
    fn from_gh(old: Vec<A>) -> Self {
        old.into_iter().map(B::from_gh).collect()
    }
}

pub trait GithubInto<T>: Sized {
    fn gh_into(self) -> T;
}

impl<A, B: FromGithub<A>> GithubInto<B> for A {
    fn gh_into(self) -> B {
        B::from_gh(self)
    }
}

// ==== impl ====

impl FromGithub<String> for model::Color {
    fn from_gh(old: String) -> Self {
        model::Color(format!("#{}", old))
    }
}

impl FromGithub<gh::User> for model::User {
    fn from_gh(old: gh::User) -> Self {
        model::User {
            id: old.id,
            name: old.login,
            icon_url: old.avatar_url,
            hyperlink: old.html_url,
        }
    }
}

impl FromGithub<gh::Label> for model::Label {
    fn from_gh(old: gh::Label) -> Self {
        model::Label {
            name: old.name,
            color: old.color.gh_into(),
        }
    }
}

impl FromGithub<gh::Issue> for model::IssueSummary {
    fn from_gh(old: gh::Issue) -> Self {
        model::IssueSummary {
            id: old.id,
            hyperlink: old.html_url,
            title: old.title,
            labels: old.labels.gh_into(),
        }
    }
}