use tataku_client_proc_macros::ChainableInitializer;


#[derive(ChainableInitializer)]
#[derive(Clone, Default)]
pub struct SearchParams {
    // used by osu/common
    #[chain] pub mode: Option<String>,
    #[chain] pub page: u16,
    #[chain] pub sort: Option<SortMethod>,
    #[chain] pub map_status: Option<MapStatus>,

    // used by quaver
    #[chain] pub min_diff: Option<f32>,
    #[chain] pub max_diff: Option<f32>,
    #[chain] pub min_length: Option<f32>,
    #[chain] pub max_length: Option<f32>,
    #[chain] pub min_lns: Option<f32>,
    #[chain] pub max_lns: Option<f32>,
    // excluding date stuff for now
    #[chain] pub min_combo: Option<f32>,
    #[chain] pub max_combo: Option<f32>,

    // text to search
    #[chain] pub text: Option<String>
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
