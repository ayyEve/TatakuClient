use crate::prelude::*;


#[derive(Clone, Debug)]
pub enum OnlineContentAction {
    Search(Box<OnlineContentSearch>),
    Download(usize),
    AudioPreview(usize),

    NextPage,
    PreviousPage,
    SetPage(usize),
}
impl From<OnlineContentAction> for TatakuAction {
    fn from(value: OnlineContentAction) -> Self {
        Self::OnlineContent(value)
    }
}


#[derive(Reflect)]
#[derive(Clone, Debug)]
#[reflect(display="display")]
pub struct OnlineContentSearchData {
    pub display: String,
    pub value: String,
}
impl OnlineContentSearchData {
    pub fn new(display: impl ToString, value: impl ToString) -> Self {
        Self {
            display: display.to_string(),
            value: value.to_string(),
        }
    }
}
impl Display for OnlineContentSearchData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display.fmt(f)
    }
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub struct OnlineContentSearch {
    /// What "engine" to use to search
    pub engine_id: String,

    /// What type of search to perform
    pub search_type: Vec<OnlineContentType>,
    
    /// What page of results are we on?
    pub page: u32,

    /// What search-specific settings were provided
    pub search_values: HashMap<String, String>,

    /// What query
    pub query: Option<String>,
}
