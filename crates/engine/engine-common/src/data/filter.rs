// TODO:! rename all this

use crate::common::*;
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
            filter: filter.into_iter().map(|s| s.to_lowercase()).filter(|s|!s.is_empty()).collect(),
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