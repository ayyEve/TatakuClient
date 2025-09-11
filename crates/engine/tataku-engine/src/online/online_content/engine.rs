use crate::*;
use common::reflect::*;

pub trait OnlineContentEngine: Send + Sync {
    fn capabilities(&self) -> &OnlineContentCapabilities;

    fn search(
        &self, 
        settings: &Settings, 
        search: online::online_content::OnlineContentSearch,
    ) -> io::AsyncLoader<OnlineContentSearchResults>; 
}

#[derive(Debug)]
pub struct OnlineContentSearchResults {
    pub items: Vec<OnlineContentItem>,
    pub count: usize,
}



#[derive(Reflect)]
#[derive(Clone, Debug)]
#[reflect(display="display")]
pub struct OnlineContentItem {
    /// Internal id
    pub id: usize,

    /// What type is this?
    pub item_type: OnlineContentItemType,

    /// Text to display to user
    pub display: String,

    /// Function to run to perform download
    #[reflect(skip)]
    pub download: Downloadable,

    /// Url to an audio preview (if map)
    pub audio_preview: Option<String>,
}
impl std::fmt::Display for OnlineContentItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display.fmt(f)
    }
}

#[derive(Reflect)]
#[derive(Clone, Debug)]
#[reflect(display="debug")]
pub enum OnlineContentItemType {
    Map {
        artist: String,
        title: String,
        creator: String,
        map_hashes: Vec<common::Md5Hash>,
    },
}



#[derive(Reflect)]
#[derive(Clone, Debug)]
#[reflect(display="display")]
pub struct OnlineContentCapabilities {
    #[reflect(alias("id"))]
    pub engine_id: String,
    pub display_name: String,
    pub available_types: Vec<OnlineContentType>,

    pub search_options: HashMap<String, SearchOption>,
}
impl std::fmt::Display for OnlineContentCapabilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.engine_id.fmt(f)
    }
}

#[derive(Reflect)]
#[derive(Clone, Debug)]
pub struct SearchOption {
    pub display: String,
    pub values: SearchOptionType,
}
impl SearchOption {
    pub fn new(
        display: impl ToString,
        values: impl Into<SearchOptionType>,
    ) -> Self {
        Self {
            display: display.to_string(),
            values: values.into(),
        }
    }
}

#[derive(Reflect)]
#[derive(Clone, Debug)]
#[reflect(display="display")]
pub enum SearchOptionType {
    Integer {
        min: i32,
        max: i32,
        step: Option<i32>,
    },
    Float {
        min: f32,
        max: f32,
        step: Option<f32>,
    },
    List {
        options: Vec<OnlineContentSearchData>,
    }
}
impl SearchOptionType {
    pub fn get_value(
        &self,
        path: &str,
        values: &dyn Reflect,
    ) -> Option<String> {
        match &self {
            SearchOptionType::Integer { .. }
            | SearchOptionType::Float { .. } => {
                match values.reflect_as_number(path) {
                    Ok(n) => {
                        let num: f32 = n.into();
                        return Some(num.to_string())
                    }
                    Err(e) => {
                        error!("Error getting search value (number): {e:?}. path: {path}");
                    }
                }
            }

            SearchOptionType::List { .. } => {
                match values.reflect_get::<OnlineContentSearchData>(path) {
                    Ok(v) => {
                        return Some(v.value.clone())
                    }
                    Err(ReflectError::ValueWrongType { .. }) => {
                        match values.reflect_get::<String>(path) {
                            Ok(v) => {
                                return Some(v.to_string())
                            }
                            Err(e) => {
                                error!("Error getting search value (string): {e:?}. path: {path}");
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error getting search value (OnlineContentSearchData): {e:?}. path: {path}");
                    }
                }
            }
        }

        None
    }
}

impl From<Vec<OnlineContentSearchData>> for SearchOptionType {
    fn from(value: Vec<OnlineContentSearchData>) -> Self {
        Self::List {
            options: value
        }
    }
}
impl From<Range<i32>> for SearchOptionType {
    fn from(value: Range<i32>) -> Self {
        Self::Integer {
            min: value.start,
            max: value.end,
            step: None,
        }
    }
}
impl From<Range<f32>> for SearchOptionType {
    fn from(value: Range<f32>) -> Self {
        Self::Float {
            min: value.start,
            max: value.end,
            step: None,
        }
    }
}

impl std::fmt::Display for SearchOptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer { .. } => "Integer".fmt(f),
            Self::Float { .. } => "Float".fmt(f),
            Self::List { .. } => "List".fmt(f),
        }
    }
}


#[derive(Reflect)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OnlineContentType {
    Maps,
    // Skins,
    // Widgets,
    // Visualizations,
}
impl std::str::FromStr for OnlineContentType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "maps" => Ok(Self::Maps),
            // "skins" => Ok(Self::Skins),
            // "widgets" => Ok(Self::Widgets),

            _ => Err(())
        }
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
impl std::fmt::Display for OnlineContentSearchData {
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
    pub search_type: Vec<online_content::OnlineContentType>,
    
    /// What page of results are we on?
    pub page: u32,

    /// What search-specific settings were provided
    pub search_values: HashMap<String, String>,

    /// What query
    pub query: Option<String>,
}



#[test]
fn test() {
    use crate::*;

    #[derive(Reflect)]
    #[reflect(dont_clone)]
    enum A {
        Variant {
            list: Vec<String>
        }
    }

    let a = A::Variant {
        list: vec![
            "hi".to_owned(),
        ]
    };

    let list = a
        .as_dyn()
        .reflect_get::<Vec<String>>("Variant.list")
        .unwrap();

    println!("{:?}", &list);
}
