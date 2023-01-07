#![allow(unused, dead_code)]
use crate::prelude::*;

#[derive(Clone, Default)]
pub struct SearchParams {
    // used by osu/common
    pub mode: Option<PlayMode>,
    pub page: u16,
    pub sort: Option<SortMethod>,
    pub map_status: Option<MapStatus>,

    // used by quaver
    pub min_diff: Option<f32>,
    pub max_diff: Option<f32>,
    pub min_length: Option<f32>,
    pub max_length: Option<f32>,
    pub min_lns: Option<f32>,
    pub max_lns: Option<f32>,
    // excluding date stuff for now
    pub min_combo: Option<f32>,
    pub max_combo: Option<f32>,

    // text to search
    pub text: Option<String>
}

#[derive(Clone)]
pub enum SortMethod {
    Default
}
impl Default for SortMethod {
    fn default() -> Self {Self::Default}
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapStatus {
    All,
    Ranked,
    Pending,
    Graveyarded,
    Approved,
    Loved,
}
impl Default for MapStatus {
    fn default() -> Self {Self::Ranked}
}
