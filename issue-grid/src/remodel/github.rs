
use super::{Remodel, Conv};
use std::collections::HashMap;

use github_issues_export_lib::model as fr;
use crate::model as to;

remodel! {
    type Github remodels (T) -> (T);

    (from: String) -> to::Color {
        to::Color(format!("#{}", from))
    }

    ((
        from: fr::User,
        user_details: &HashMap<String, fr::UserDetails>,
    )) -> to::User {
        to::User {
            id: from.id,
            name: user_details[&from.login].name.clone(),
            login: from.login,
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

    ((
        from: fr::Issue,
        user_details: &HashMap<String, fr::UserDetails>,
    )) -> to::IssueSummary {
        to::IssueSummary {
            number: from.number,
            hyperlink: from.html_url,
            title: from.title,
            labels: conv(from.labels),
            creator: conv((from.user, user_details)),
        }
    }
}