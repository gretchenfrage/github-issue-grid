
use crate::sort::RegexMatch;
use regex::Regex;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct IssueSummary {
    pub number: u64,
    pub hyperlink: String,
    pub title: String,
    pub labels: Vec<Label>,
    pub creator: User,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Label {
    pub name: String,
    pub color: Color,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BinSummary {
    pub name: Option<String>,
    pub color: Option<Color>, // currently ignored
    pub main_label: Option<Label>,
    pub issues: Vec<IssueSummary>,
    pub is_overflow: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub icon_url: String,
    pub hyperlink: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct Color(
    // valid CSS color, includes the pound.
    pub String
);

// == impls ==

impl RegexMatch for IssueSummary {
    fn is_match(&self, regex: &Regex) -> bool {
        self.labels.iter()
            .any(|label| regex.is_match(&label.name))
    }
}