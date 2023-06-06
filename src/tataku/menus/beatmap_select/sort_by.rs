use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Dropdown, Serialize, Deserialize)]
pub enum SortBy {
    Title,
    Artist,
    Creator,
    Difficulty,
}
