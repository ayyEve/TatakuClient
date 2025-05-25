use crate::prelude::*;

#[derive(Clone, Debug)]
#[derive(Reflect)]
pub struct LoadingStatus {
    pub name: &'static str,
    pub error: Option<String>,

    pub item_count: usize, // items in the list
    pub items_complete: usize, // items done loading in the list

    pub complete: bool,
}
impl LoadingStatus {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            error: None,
            item_count: 0,
            items_complete: 0,

            complete: false
        }
    }
}
