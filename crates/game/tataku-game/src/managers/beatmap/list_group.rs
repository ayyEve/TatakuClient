use crate::prelude::*;
use common::{
    Md5Hash,
    reflect::*,
};

#[derive(Reflect)]
#[derive(Debug, Clone)]
pub(crate) struct BeatmapListGroup {
    pub id: usize,
    pub selected: bool,
    pub name: String,
    pub maps: Vec<Md5Hash>,
}
impl BeatmapListGroup {
    pub fn has_hash(&self, hash: &Md5Hash) -> Option<usize> {
        self
            .maps
            .iter()
            .position(|i| i == hash)
    }
}
