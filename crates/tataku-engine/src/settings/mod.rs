mod settings;
mod osu_import;
mod osu_settings;
mod taiko_settings;
mod catch_settings;
mod mania_settings;
mod display_settings;
mod settings_helpers;
mod logging_settings;
mod cursor_settings;
mod integration_settings;
mod settings_deserializer;
mod common_gameplay_settings;
mod background_game_settings;


pub use settings::*;
pub use osu_import::*;
pub use osu_settings::*;
pub use taiko_settings::*;
pub use catch_settings::*;
pub use mania_settings::*;
pub use cursor_settings::*;
pub use display_settings::*;
pub use settings_helpers::*;
pub use logging_settings::*;
pub use integration_settings::*;
pub use settings_deserializer::*;
pub use common_gameplay_settings::*;
pub use background_game_settings::*;


use crate::prelude::IcedElement;

#[derive(Copy, Clone, Debug)]
pub enum QueryType {
    Any,
    All,
}

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
            QueryType::All => self.filter.iter().all(|query_str|keywords.contains(&&**query_str)),
            QueryType::Any => self.filter.iter().any(|query_str|keywords.iter().any(|k|k.starts_with(query_str))),
        }
    }
}

#[derive(Default)]
pub struct SettingsBuilder {
    pub categories: Vec<SettingsCategory>, //Vec<(String, (Vec<IcedElement>, Vec<IcedElement>))>,
}
impl SettingsBuilder {
    pub fn add_item(
        &mut self,
        prop: impl Into<IcedElement>,
        val: impl Into<IcedElement>,
    ) {
        let sc = self.categories.last_mut().unwrap();
        sc.properties.push(prop.into());
        sc.values.push(val.into());
    }

    pub fn add_category(
        &mut self, 
        category: impl ToString,
    ) {
        self.categories.push(SettingsCategory {
            name: category.to_string(),
            ..Default::default()
        });
    }
}

#[derive(Default)]
pub struct SettingsCategory {
    pub name: String,
    pub properties: Vec<IcedElement>, 
    pub values: Vec<IcedElement>,
}
