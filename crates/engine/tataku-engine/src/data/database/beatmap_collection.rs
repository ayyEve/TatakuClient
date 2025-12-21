use crate::*;
use common::{
    Md5Hash,
    reflect::*,
};


#[derive(Reflect)]
#[derive(Clone, Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BeatmapCollection {
    pub name: String,
    pub beatmaps: Vec<Md5Hash>,
}
