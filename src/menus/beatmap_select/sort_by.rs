use crate::prelude::*;


#[derive(Copy, Clone, Debug, PartialEq, Eq, ayyeve_piston_ui::prelude::Dropdown)]
pub enum SortBy {
    Title,
    Artist,
    Creator,
    Difficulty,
}
