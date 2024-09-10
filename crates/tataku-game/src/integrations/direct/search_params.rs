
#[derive(Clone, Default)]
pub struct SearchParams {
    // used by osu/common
    pub mode: Option<String>,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MapStatus {
    All,
    #[default]
    Ranked,
    Pending,
    Graveyarded,
    Approved,
    Loved,
}


#[derive(Clone, Default)]
pub enum SortMethod {
    #[default]
    Default
}
