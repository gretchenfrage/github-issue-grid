
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

impl IssueRenderer {
    pub fn new() -> Self {
        let mut reg = Handlebars::new();
        reg.register_template_string(
            "issue",
            include_str!("template.hb")
        ).expect("unexpected handlebars template compilation failure");

        IssueRenderer {
            hb: reg,
        }
    }

    /// Render an issue into markdown.
    ///
    /// Also produce the suggested relative file path to save at.
    pub fn render_md(
        &self,
        issue: &model::IssueWithComments
    ) -> Result<(String, PathBuf)> {
        let md = self.hb.render("issue", &issue)?;
        let path = PathBuf::from(format!(
            "{:03}-{}.md",
            issue.issue.number,
            slug::slugify(&issue.issue.title),
        ));
        Ok((md, path))
    }
}

/// Handlebars template for rendering issue to markdown.
pub const TEMPLATE: &'static str = include_str!("template.hb");

#[test]
fn new_issue_renderer() {
    // make sure this doesn't panic from handlebars failure
    IssueRenderer::new();
}

