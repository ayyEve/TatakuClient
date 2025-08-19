use crate::prelude::*;

#[derive(Reflect)]
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[reflect(from_string = "auto", display = "display")]
pub enum SortBy {
    #[default]
    Title,
    Artist,
    Creator,
    Difficulty,
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
}
impl Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
