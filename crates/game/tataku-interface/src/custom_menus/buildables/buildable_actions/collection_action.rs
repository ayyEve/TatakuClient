use crate::prelude::*;
use common::Md5Hash;
use tataku::TatakuValue;
use common::reflect::Reflect;

/// An action that deals with the current beatmap
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableCollectionAction {
    Create {
        #[serde(rename="$value")]
        collection: BuildableValue,
    },

    /// Add the map to the collection
    Add {
        collection: Wrapped<BuildableValue>,
        map: Wrapped<BuildableValue>,
    },
    /// Remove the map from the collection
    Remove {
        collection: Wrapped<BuildableValue>,
        map: Wrapped<BuildableValue>,
    },
}

#[cfg(feature="graphics")]
impl BuildableCollectionAction {
    pub fn resolve(
        &self, 
        values: &mut dyn Reflect, 
        passed_in: Option<&TatakuValue>,
    ) -> Option<actions::beatmap::BeatmapAction> {
        match self {
            Self::Create { 
                collection,
            } => {
                let collection = collection.resolve(values, passed_in)?;
                Some(actions::beatmap::CollectionAction::Create {
                    collection: collection.as_string()
                }.into())
            }
            Self::Add { 
                collection,
                map,
            } => {
                let map = map.inner.resolve(values, passed_in)?;
                let hash = Md5Hash::try_from(map.as_string()).ok()?;
                let collection = collection.inner.resolve(values, passed_in)?;
                Some(actions::beatmap::CollectionAction::Add {
                    map: hash,
                    collection: collection.as_string()
                }.into())
            }
            Self::Remove { 
                collection,
                map,
            } => {
                let map = map.inner.resolve(values, passed_in)?;
                let hash = Md5Hash::try_from(map.as_string()).ok()?;
                let collection = collection.inner.resolve(values, passed_in)?;
                Some(actions::beatmap::CollectionAction::Remove {
                    map: hash,
                    collection: collection.as_string()
                }.into())
            }
        }
    }

    pub fn build(&mut self) {
        match self {
            Self::Create { 
                collection 
            } => collection.build(),

            Self::Add { map, collection } 
            | Self::Remove { map, collection }
            => {
                map.inner.build();
                collection.inner.build();
            },
        }
    }
}


