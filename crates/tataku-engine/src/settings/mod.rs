mod helpers;
mod settings;
mod cursor_settings;
mod display_settings;
mod logging_settings;
mod integration_settings;
mod common_gameplay_settings;
mod background_game_settings;

pub use helpers::*;
pub use settings::*;
pub use cursor_settings::*;
pub use display_settings::*;
pub use logging_settings::*;
pub use integration_settings::*;
pub use common_gameplay_settings::*;
pub use background_game_settings::*;


use crate::prelude::*;

#[derive(Reflect)]
#[derive(Copy, Clone, Debug)]
pub enum QueryType {
    Any,
    All,
}

#[derive(Clone)]
#[derive(Reflect)]
pub struct ItemFilter {
    pub filter: Vec<String>,
    pub filter_type: QueryType,
}
impl ItemFilter {
    pub fn new(filter: Vec<String>, filter_type: QueryType) -> Self {
        Self {
            filter: filter.into_iter().map(|s|s.to_lowercase()).filter(|s|!s.is_empty()).collect(),
            filter_type,
        }
    }

    /// check item against the filter to see if it should be included (true) or filtered (false)
    pub fn check(&self, item: impl AsRef<str>) -> bool {
        if self.filter.is_empty() { return true }

        let item = item.as_ref().to_lowercase();

        let keywords:Vec<&str> = item.split(" ").collect();
        match self.filter_type {
            QueryType::All => self.filter.iter()
                .all(|query_str| keywords.contains(&&**query_str)),
            QueryType::Any => self.filter.iter()
                .any(|query_str| 
                    keywords.iter().any(|k| k.starts_with(query_str))
                ),
        }
    }
}


#[cfg(feature="graphics")]
use crate::prelude::Widget;
#[cfg(feature="graphics")]
#[derive(Default)]
pub struct SettingsCategory {
    pub name: String,
    pub properties: Vec<Box<dyn Widget>>, 
    pub values: Vec<Box<dyn Widget>>,
    pub names: Vec<String>,
}
