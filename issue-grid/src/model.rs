
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