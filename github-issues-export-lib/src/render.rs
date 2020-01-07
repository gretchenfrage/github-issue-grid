
use crate::{
    model,
    error::*,
};

use std::path::PathBuf;

use handlebars::Handlebars;

/// Issue to markdown rendering service.
pub struct IssueRenderer {
    hb: Handlebars,
}

macro_rules! suggest_path {
    ($issue:expr, $ext:expr)=>{{
        fn as_issue_with_comments(
            issue: &model::IssueWithComments
        ) -> &model::IssueWithComments {
            issue
        }
        let issue = as_issue_with_comments(&$issue);

        PathBuf::from(format!(
            concat!("{:03}-{}", $ext),
            issue.issue.number,
            slug::slugify(&issue.issue.title),
        ))
    }};
}

impl IssueRenderer {
    pub fn new() -> Self {
        let mut reg = Handlebars::new();
        reg.register_template_string("issue", TEMPLATE)
            .expect("unexpected handlebars template compilation failure");

        IssueRenderer {
            hb: reg,
        }
    }

    /// Render an issue into markdown.
    ///
    /// Also produce the suggested relative file path to save at.
    pub fn render_md(
        &self,
        issue: &model::IssueWithComments,
    ) -> Result<(String, PathBuf)> {
        let md = self.hb.render("issue", &issue)?;
        let path = suggest_path!(&issue, ".md");
        Ok((md, path))
    }

    /// Render an issue into markdown, then into HTML.
    ///
    /// Also produce the suggested relative file path to save at.
    pub fn render_html(
        &self,
        issue: &model::IssueWithComments,
    ) -> Result<(String, PathBuf)> {
        let md = self.hb.render("issue", &issue)?;
        let html = markdown::to_html(&md);
        let path = suggest_path!(&issue, ".html");
        Ok((html, path))
    }
}

/// Handlebars template for rendering issue to markdown.
const TEMPLATE: &'static str = include_str!("template.hb");

#[test]
fn new_issue_renderer() {
    // make sure this doesn't panic from handlebars failure
    IssueRenderer::new();
}

