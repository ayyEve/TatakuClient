use crate::prelude::*;
use std::str::FromStr;
use common::reflect::*;
use tataku::TatakuValue;
use engine::{
    VariablePathResolver,
    online_content::*,
};

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableOnlineContentAction {
    Search(BuildableOnlineContentSearch),

    NextPage,
    PreviousPage,
    
    SetPage {
        #[serde(rename="$value")]
        page: BuildableValue,
    },

    #[serde(alias="download")]
    StartDownload {
        #[serde(rename="$value")]
        index: BuildableValue,
    },

    AudioPreview {
        #[serde(rename="$value")]
        index: BuildableValue,
    },
}

#[cfg(feature="graphics")]
impl BuildableOnlineContentAction {
    fn index(
        index: &BuildableValue,

        values: &dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<usize> {
        index.resolve(values, passed_in)
            .and_then(|i| i.as_u64())
            .map(|i| i as usize)
    }

    pub fn resolve(
        &self,
        values: &dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<actions::online_content::OnlineContentAction> {
        match self {
            Self::NextPage => Some(actions::online_content::OnlineContentAction::NextPage),
            Self::PreviousPage => Some(actions::online_content::OnlineContentAction::PreviousPage),

            Self::SetPage { page } => Some(actions::online_content::OnlineContentAction::SetPage(Self::index(
                page,
                values, 
                passed_in
            )?)),

            Self::AudioPreview { index } => Some(actions::online_content::OnlineContentAction::AudioPreview(Self::index(
                index,
                values, 
                passed_in
            )?)),
            
            Self::StartDownload { index } => Some(actions::online_content::OnlineContentAction::Download(Self::index(
                index,
                values, 
                passed_in
            )?)),
            
            Self::Search(search) => Some(actions::online_content::OnlineContentAction::Search(
                Box::new(search.resolve(values, passed_in)?)
            )),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub struct BuildableOnlineContentSearch {
    /// What "engine" to use to search
    engine_id: Wrapped<BuildableValue>,

    /// What type of search to perform
    search_type: Wrapped<BuildableValue>,
    
    /// What page of results are we on?
    page: Wrapped<BuildableValue>,

    /// What search-specific settings were provided
    #[serde(alias="values", default)]
    search_values: Option<HashMap<String, Wrapped<BuildableValue>>>,
    #[serde(rename="@valuesMapPath", default)]
    search_values_map_path: Option<VariablePathResolver>,
    #[serde(rename="@valuesKeyValuePath", default)]
    search_values_key_value_path: Option<VariablePathResolver>,

    /// What query
    #[serde(default)]
    query: Option<Wrapped<BuildableValue>>,
}

#[cfg(feature="graphics")]
impl BuildableOnlineContentSearch {
    fn get_search_values(
        &self, 
        engine_id: &str,
        values: &dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<HashMap<String, String>> {
        let mut search_values = HashMap::new();

        if let Some(buildable) = &self.search_values {
            search_values = buildable
                .iter()
                .filter_map(|(key, value)| value.inner
                    .resolve(values, passed_in)
                    .map(|value| (key.clone(), value.as_string()))
                )
                .collect();
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
                let Some(ReflectItemIndex::Value(id)) = i.index else { break; };
                let Some(id) = id.downcast_ref::<String>().cloned() else { continue; };

                let Some(option) = engine.search_options.get(&id) else {
                    warn!("search option id not found: {id}");
                    continue;
                };

                if let Some(value) = option.values.get_value("value", i.item) {
                    search_values.insert(id, value);
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

            for (id, option) in engine.search_options.iter() {
                let id = id.clone();
                let path = format!("{key_value_path}.{id}");

                if let Some(value) = option.values.get_value(&path, values) {
                    search_values.insert(id, value);
                } else {
                    warn!("Search value not found: {path}");
                }
            }
        }

        if search_values.is_empty() {
            None
        } else {
            Some(search_values)
        }
    }

    fn resolve(
        &self,
        values: &dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<OnlineContentSearch> {
        let engine_id = self
            .engine_id.inner
            .resolve(values, passed_in)?
            .as_string();

        let search_values = self
            .get_search_values(&engine_id, values, passed_in)?;

        Some(OnlineContentSearch {
            engine_id,
            page: self.page.inner.resolve(values, passed_in)?.as_u32()?,
            search_values,

            search_type: vec![OnlineContentType::from_str(
                &self.search_type.inner.resolve(values, passed_in)?.as_string()
            ).ok()?],

            query: self.query.as_ref().and_then(|q| q.inner
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
