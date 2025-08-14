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


#[derive(Clone, Debug)]
#[derive(Reflect)]
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

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct OnlineContentSearch {
    /// What "engine" to use to search
    pub engine_id: String,

    /// What type of search to perform
    pub search_type: Vec<OnlineContentType>,
    
    /// What page of results are we on?
    pub page: u32,

    /// What search-specific settings were provided
    pub search_values: OnlineContentSearchValueCollection,

    /// What query
    pub query: Option<String>,
}


#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct OnlineContentSearchValue {
    pub id: String,
    pub value: String,
} 
impl OnlineContentSearchValue {
    pub fn new(id: impl ToString, value: impl ToString) -> Self {
        Self {
            id: id.to_string(),
            value: value.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(transparent)]
pub struct OnlineContentSearchValueCollection(Vec<OnlineContentSearchValue>);
impl OnlineContentSearchValueCollection {
    pub fn get_value(&self, id: &str) -> Option<&String> {
        self.0
            .iter()
            .find(|i| i.id == id)
            .map(|i| &i.value)
    }
    pub fn get_value_or_default(&self, id: &str, default: impl ToString) -> Cow<'_, String> {
        self.0
            .iter()
            .find(|i| i.id == id)
            .map_or_else(
                || Cow::Owned(default.to_string()), 
                |i| Cow::Borrowed(&i.value)
            )
    }
    pub fn get_values(&self, id: &str) -> Vec<&String> {
        self.0
            .iter()
            .filter(|i| i.id == id)
            .map(|i| &i.value)
            .collect()
    }

}
impl From<Vec<OnlineContentSearchValue>> for OnlineContentSearchValueCollection {
    fn from(value: Vec<OnlineContentSearchValue>) -> Self {
        Self(value)
    }
}
