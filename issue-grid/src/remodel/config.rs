
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

serde_as_list! {
    struct IssueSortInstr;
    field regex;
    option_tail order;
}