use crate::prelude::*;
use common::reflect::*;

#[derive(Reflect)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BeatmapGroupValue {
    Set(String),
    Collection(String),
}
impl BeatmapGroupValue {
    pub fn get_name(&self) -> &String {
        match self {
            Self::Set(name) => name,
            Self::Collection(name) => name,
        }
    }
}
