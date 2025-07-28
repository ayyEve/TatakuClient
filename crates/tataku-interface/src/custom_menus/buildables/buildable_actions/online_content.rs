use crate::prelude::*;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum BuildableOnlineContentAction {
    #[serde(rename_all="camelCase")]
    Search {
        /// What "engine" to use to search
        engine_id: BuildableValueTag,

        /// What type of search to perform
        search_type: BuildableValueTag,
        
        /// What page of results are we on?
        page: BuildableValueTag,

        /// What search-specific settings were provided
        #[serde(alias="values", default)]
        search_values: Option<Vec<BuildableSearchValue>>,

        #[serde(alias="@valuesMapPath", default)]
        search_values_map_path: Option<VariablePathResolver>,

        #[serde(alias="@valuesKeyValuePath", default)]
        search_values_key_value_path: Option<VariablePathResolver>,

        /// What query
        #[serde(default)]
        query: Option<BuildableValueTag>,
    },

    NextPage,
    PreviousPage,
    
    SetPage {
        #[serde(rename="@page")]
        #[serde(default)]
        page_property: Option<usize>,

        #[serde(rename="$value")]
        #[serde(default)]
        page_tag: Option<BuildableValue>,
    },

    #[serde(alias="download")]
    StartDownload {
        #[serde(rename="@index")]
        #[serde(default)]
        index_property: Option<usize>,

        #[serde(rename="$value")]
        index_tag: Option<BuildableValue>,
    },

    AudioPreview {
        #[serde(rename="@index")]
        #[serde(default)]
        index_property: Option<usize>,

        #[serde(rename="$value")]
        index_tag: Option<BuildableValue>,
    },
}
impl BuildableOnlineContentAction {

    fn index(
        index_property: Option<usize>,
        index_tag: Option<BuildableValue>,

        values: &dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<usize> {
        index_tag
            .and_then(|i| 
                i.resolve(values, passed_in)
                .and_then(|i| i.as_u64().ok())
            )
            .map(|i| i as usize)
            .or(index_property)
    }

    pub fn into_action(
        self,
        values: &dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<OnlineContentAction> {
        match self {
            Self::NextPage => Some(OnlineContentAction::NextPage),
            Self::PreviousPage => Some(OnlineContentAction::PreviousPage),

            Self::SetPage { 
                page_property, 
                page_tag 
            } => Some(OnlineContentAction::SetPage(Self::index(
                page_property, 
                page_tag, 
                values, 
                passed_in
            )?)),

            Self::AudioPreview { 
                index_property, 
                index_tag 
            } => Some(OnlineContentAction::AudioPreview(Self::index(
                index_property, 
                index_tag, 
                values, 
                passed_in
            )?)),
            
            Self::StartDownload { 
                index_property, 
                index_tag 
            } => Some(OnlineContentAction::Download(Self::index(
                index_property, 
                index_tag, 
                values, 
                passed_in
            )?)),
            
            Self::Search { 
                engine_id, 
                search_type, 
                page, 
                search_values, 
                search_values_map_path, 
                search_values_key_value_path, 
                query 
            } => Some(OnlineContentAction::Search(
                BuildableOnlineContentSearch {
                    engine_id, 
                    search_type, 
                    page, 
                    search_values, 
                    search_values_map_path, 
                    search_values_key_value_path, 
                    query 
                }.resolve(values, passed_in)?
            )),
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct BuildableSearchValue {
    pub id: BuildableValueTag,
    pub value: BuildableValueTag,
}
impl BuildableSearchValue {
    pub fn resolve(
        &self,
        values: &dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<OnlineContentSearchValue> {
        Some(OnlineContentSearchValue {
            id: self.id.resolve(values, passed_in)?.as_string(),
            value: self.value.resolve(values, passed_in)?.as_string()
        })
    }
}


struct BuildableOnlineContentSearch {
    /// What "engine" to use to search
    engine_id: BuildableValueTag,

    /// What type of search to perform
    search_type: BuildableValueTag,
    
    /// What page of results are we on?
    page: BuildableValueTag,

    /// What search-specific settings were provided
    search_values: Option<Vec<BuildableSearchValue>>,
    search_values_map_path: Option<VariablePathResolver>,
    search_values_key_value_path: Option<VariablePathResolver>,

    /// What query
    query: Option<BuildableValueTag>,
}
impl BuildableOnlineContentSearch {
    fn get_value(
        search_option: &SearchOption,
        path: &str,
        values: &dyn Reflect,
    ) -> Option<String> {
        match &search_option.values {
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
    

    fn get_search_values(
        &self, 
        engine_id: &str,
        values: &dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<Vec<OnlineContentSearchValue>> {
        let mut search_values = Vec::new();

        if let Some(self_search_values) = &self
            .search_values 
        {
            search_values = self_search_values
                .iter()
                .filter_map(|i| i.resolve(values, passed_in))
                .collect::<Vec<_>>();
        } else if let Some(map_path) = &self.search_values_map_path {

            let map_path = map_path.resolve_path(values)
                .inspect_err(|e| error!("error resolving path: {e:?}"))
                .ok()?;
            let engines = Engines::new(values);
            let engine = engines.get(engine_id)?;

            let iter = values
                .impl_iter(ReflectPath::new(&map_path))
                .inspect_err(|e| 
                    error!("Error with search values map path '{map_path}': {e:?}")
                )
                .ok()?;
            
            for i in iter {
                let Ok(id) = i
                    .item
                    .reflect_get::<String>("id")
                    .inspect_err(|e| 
                        error!("Error getting search values id: {e:?}")
                    )
                else { continue };
                let id = id.to_string();

                let Some(option) = engine
                    .search_options
                    .iter()
                    .find(|i| i.id == id)
                else { 
                    warn!("search option id not found: {id}"); 
                    continue;
                };

                if let Some(value) = Self::get_value(
                    option,
                    "value",
                    i.item,
                ) {
                    search_values.push(OnlineContentSearchValue::new(id, value));
                } else {
                    warn!("Search value not found: {id}");
                }
            }
        } else if let Some(key_value_path) = &self.search_values_key_value_path {
            let engines = Engines::new(values);
            let engine = engines.get(engine_id)?;

            let key_value_path = key_value_path.resolve_path(values)
                .inspect_err(|e| error!("error resolving path: {e:?}"))
                .ok()?;

            for i in engine.search_options.iter() {
                let id = i.id.clone();
                let path = format!("{key_value_path}.{id}");

                if let Some(value) = Self::get_value(
                    i,
                    &path,
                    values
                ) {
                    search_values.push(OnlineContentSearchValue::new(id, value));
                } else {
                    warn!("Search value not found: {path}");
                }
            }
        }

        Some(search_values)
    }

    fn resolve(
        self,
        values: &dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<OnlineContentSearch> {
        let engine_id = self
            .engine_id
            .resolve(values, passed_in)?
            .as_string();

        let search_values = self
            .get_search_values(&engine_id, values, passed_in)?;

        Some(OnlineContentSearch {
            engine_id,
            page: self.page.resolve(values, passed_in)?.as_u32().ok()?,
            search_values: search_values.into(),

            search_type: vec![OnlineContentType::from_str(
                &self.search_type.resolve(values, passed_in)?.as_string()
            ).ok()?],
                // .iter()
                // .filter_map(|i| OnlineContentType::from_str(
                //     &i.resolve(values, passed_in)?.as_string()
                // ).ok())
                // .collect(),
                
            query: self.query
                .and_then(|i| i
                    .resolve(values, passed_in)
                    .map(|i| i.as_string())
                )
        })
    }
}


/// helper struct to reduce code
struct Engines<'a> {
    engines: MaybeOwned<'a, HashMap<String, OnlineContentCapabilities>>,
}
impl<'a> Engines<'a> {
    fn new(values: &'a dyn Reflect) -> Self {
        let engines = values
            .reflect_get::<HashMap<String, OnlineContentCapabilities>>(
                "game.online_content.engines"
            )
            .expect("no engines?");

        Self {
            engines
        }
    }

    fn get(
        &self, 
        engine_id: &str,
    ) -> Option<&OnlineContentCapabilities> {
        let Some(engine) = self
            .engines
            .values()
            .find(|e| e.engine_id == engine_id)
        else { 
            warn!("engine {engine_id} not found!"); 
            return None
        };

        Some(engine)
    }
}


