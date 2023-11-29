use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Dropdown, Serialize, Deserialize)]
pub enum SortBy {
    Title,
    Artist,
    Creator,
    Difficulty,
}
impl Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl SortBy {
    pub fn list() -> Vec<Self> {
        vec![
            Self::Title,
            Self::Artist,
            Self::Creator,
            Self::Difficulty
        ]
    }

    pub fn to_string(&self) -> String {
        format!("{self:?}")
    }

    pub fn from_str(s: &String) -> Option<Self> {
        for i in Self::list() {
            if s == &format!("{i:?}") { return Some(i) };
        }
        None
    }
}