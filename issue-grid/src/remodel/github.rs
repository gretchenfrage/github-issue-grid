
use super::{Remodel, Conv};

use github_issues_export_lib::model as fr;
use crate::model as to;

remodel! {
    type Github remodels (T) -> (T);

    (from: String) -> to::Color {
        to::Color(format!("#{}", from))
    }

    (from: fr::User) -> to::User {
        to::User {
            id: from.id,
            name: from.login,
            icon_url: from.avatar_url,
            hyperlink: from.html_url,
        }
    }

    (from: fr::Label) -> to::Label {
        to::Label {
            name: from.name,
            color: conv(from.color),
        }
    }

    (from: fr::Issue) -> to::IssueSummary {
        to::IssueSummary {
            id: from.id,
            hyperlink: from.html_url,
            title: from.title,
            labels: conv(from.labels),
        }
    }
}