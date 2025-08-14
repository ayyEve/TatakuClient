use crate::prelude::*;

pub trait OnlineContentEngine: Send + Sync {
    fn capabilities(&self) -> &OnlineContentCapabilities;

    fn search(
        &self, 
        settings: &Settings, 
        search: OnlineContentSearch,
    ) -> AsyncLoader<OnlineContentSearchResults>; 
}

#[derive(Debug)]
pub struct OnlineContentSearchResults {
    pub items: Vec<OnlineContentItem>,
    pub count: usize,
}



#[derive(Clone, Debug)]
#[derive(Reflect)]
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

#[derive(Clone, Debug)]
#[derive(Reflect)]
#[reflect(display="debug")]
pub enum OnlineContentItemType {
    Map {
        artist: String,
        title: String,
        creator: String,
        map_hashes: Vec<Md5Hash>,
    },
}



#[derive(Clone, Debug)]
#[derive(Reflect)]
#[reflect(display="display")]
pub struct OnlineContentCapabilities {
    #[reflect(alias("id"))]
    pub engine_id: String,
    pub display_name: String,
    pub available_types: Vec<OnlineContentType>,

    pub search_options: Vec<SearchOption>,
}
impl Display for OnlineContentCapabilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.engine_id.fmt(f)
    }
}

#[derive(Clone, Debug)]
#[derive(Reflect)]
pub struct SearchOption {
    pub id: String,
    pub display: String,
    pub values: SearchOptionType,
}
impl SearchOption {
    pub fn new(
        id: impl ToString,
        display: impl ToString,
        values: impl Into<SearchOptionType>,
    ) -> Self {
        Self {
            id: id.to_string(),
            display: display.to_string(),
            values: values.into(),
        }
    }
}

#[derive(Clone, Debug)]
#[derive(Reflect)]
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

impl Display for SearchOptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer { .. } => "Integer".fmt(f),
            Self::Float { .. } => "Float".fmt(f),
            Self::List { .. } => "List".fmt(f),
        }
    }
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[derive(Reflect)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
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



#[test]
fn test() {
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

    use crate::prelude::*;

    let list = a
        .as_dyn()
        .reflect_get::<Vec<String>>("Variant.list")
        .unwrap();

    println!("{:?}", &list);
}
