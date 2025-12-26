pub mod cursor;
pub mod display;
pub mod logging;
pub mod buildable;
pub mod settings;
pub mod osu_import;
pub mod connection;
pub mod integration;
pub mod common_gameplay;
pub mod background_game;

pub use self::buildable::*;
pub use settings::Settings;

use crate::*;
use common::reflect::*;

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

#[derive(Default)]
#[cfg(feature="graphics")]
pub struct SettingsCategory {
    pub name: String,
    pub properties: Vec<Box<dyn ui::widget::Widget<actions::Action>>>, 
    pub values: Vec<Box<dyn ui::widget::Widget<actions::Action>>>,
    pub names: Vec<String>,
}
