use crate::prelude::*;
use common::{
    Md5Hash,
    reflect::*,
};

/// A group of beatmaps
#[derive(Reflect)]
#[derive(Debug, Clone)]
pub struct BeatmapGroup {
    pub name: String,
    pub group_value: BeatmapGroupValue,
    pub maps: Vec<Md5Hash>,
}
impl BeatmapGroup {
    pub fn new(group: BeatmapGroupValue) -> Self {
        Self {
            name: group.get_name().clone(),
            group_value: group,
            maps: Vec::new()
        }
    }

    pub fn get_name(&self) -> &String {
        self.group_value.get_name()
    }
}
