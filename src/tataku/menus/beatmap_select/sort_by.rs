use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Dropdown, Serialize, Deserialize)]
pub enum SortBy {
    #[default]
    Title,
    Artist,
    Creator,
    Difficulty,
}
