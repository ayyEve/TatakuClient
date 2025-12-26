use crate::prelude::*;
use tataku::TatakuValue;
use common::{
    Md5Hash,
    reflect::Reflect,
};

/// An action that deals with the current beatmap
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableMapAction {
    /// Play the current map
    Play,

    /// Confirm the current map
    Confirm,

    /// Change to the next map
    Next,

    /// Change to the previous map
    Previous {
        #[serde(rename="$value", default)]
        action: actions::beatmap::MapActionIfNone,
    },

    /// Change to a random map
    Random {
        #[serde(rename="@use_preview", default)] 
        use_preview: bool,
    },

    /// Select a specific set by group id
    SelectGroup { 
        #[serde(rename="$value")]
        value: BuildableValue,
    },

    /// Select a specific map by hash
    SelectMap { 
        #[serde(rename="$value")]
        value: BuildableValue,
    },

    /// Set the current playmode
    SetPlaymode { 
        #[serde(rename="$value")]
        value: BuildableValue,
    },

    /// Refresh the beatmap list
    RefreshMaps,

    /// Delete the current map
    DeleteCurrent,

    /// Delete the provided map hash
    Delete {
        #[serde(rename="$value")]
        value: BuildableValue,
    },

    /// Change to the previous map
    Collection {
        #[serde(rename="$value")]
        action: super::collection_action::BuildableCollectionAction,
    },

    // TODO: document the difference between BeatmapAction::Next and BeatmapListAction::NextSet
    NextMap,
    NextSet,
    PreviousMap,
    PreviousSet,
}

#[cfg(feature="graphics")]
impl BuildableMapAction {
    pub fn resolve(
        &self, 
        values: &mut dyn Reflect, 
        passed_in: Option<&TatakuValue>
    ) -> Option<actions::beatmap::BeatmapAction> {
        match self {
            Self::Play => Some(actions::beatmap::BeatmapAction::PlaySelected),
            Self::Confirm => Some(actions::beatmap::BeatmapAction::ConfirmSelected),

            Self::Next => Some(actions::beatmap::BeatmapAction::Next),
            Self::Previous { action } 
                => Some(actions::beatmap::BeatmapAction::Previous(*action)),

            Self::Random { use_preview } 
                => Some(actions::beatmap::BeatmapAction::Random(*use_preview)),

            Self::DeleteCurrent => Some(actions::beatmap::BeatmapAction::DeleteCurrent(
                actions::beatmap::PostDelete::Next
            )),

            Self::Delete { value } => {
                let value = value.resolve(values, passed_in)?;
                let hash = Md5Hash::try_from(value.as_string()).ok()?;
                Some(actions::beatmap::BeatmapAction::Delete(hash))
            }

            Self::NextMap => Some(actions::beatmap::BeatmapListAction::NextMap.into()),
            Self::NextSet => Some(actions::beatmap::BeatmapListAction::NextSet.into()),
            Self::PreviousMap => Some(actions::beatmap::BeatmapListAction::PrevMap.into()),
            Self::PreviousSet => Some(actions::beatmap::BeatmapListAction::PrevSet.into()),
            Self::RefreshMaps => Some(actions::beatmap::BeatmapListAction::Refresh.into()),

            Self::SetPlaymode { value } => {
                let value = value.resolve(values, passed_in)?;
                Some(actions::beatmap::BeatmapAction::SetPlaymode(value.as_string()))
            }


            Self::SelectGroup { value } => {
                let num = value.resolve(values, passed_in)?.as_u32()?;
                Some(actions::beatmap::BeatmapAction::ListAction(
                    actions::beatmap::BeatmapListAction::SelectSet(num as usize)
                ))
            }

            Self::SelectMap { value } => {
                let hash = value
                    .resolve(values, passed_in)?
                    .string_maybe()?
                    .try_into()
                    .ok()?;
                
                Some(actions::beatmap::BeatmapAction::Set(
                    hash, 
                    actions::beatmap::SetBeatmapOptions::default().use_preview_point(true)
                ))
            }

            Self::Collection { 
                action 
            } => action.resolve(values, passed_in),
        }
    }

    pub fn build(&mut self) {
        match self {
            Self::SelectGroup { value } 
                => value.build(),
                
            Self::SelectMap { value } 
                => value.build(),

            Self::SetPlaymode { value } 
                => value.build(),

            Self::Delete { value } 
                => value.build(),

            Self::Collection { action } 
                => action.build(),

            _ => {}
        }
    }
}



#[test]
fn test() {
    use quick_xml::de::from_str;
    #[derive(Deserialize, PartialEq, Debug)]
    struct Action { #[serde(rename="$value")] action: BuildableMapAction }

    assert_eq!(
        from_str::<Action>(r#"<action> <play/> </action>"#).unwrap(),
        Action { action: BuildableMapAction::Play }
    );

    assert_eq!(
        from_str::<Action>(r#"<action> <confirm/> </action>"#).unwrap(),
        Action { action: BuildableMapAction::Confirm }
    );

    assert_eq!(
        from_str::<Action>(r#"<action> <next/> </action>"#).unwrap(),
        Action { action: BuildableMapAction::Next }
    );

    assert_eq!(
        from_str::<Action>(r#"<action> <previous/> </action>"#).unwrap(),
        Action { action: BuildableMapAction::Previous { action: actions::beatmap::MapActionIfNone::default()} }
    );
    assert_eq!(
        from_str::<Action>(r#"<action> <previous> <setNone/> </previous> </action>"#).unwrap(),
        Action { action: BuildableMapAction::Previous{ action: actions::beatmap::MapActionIfNone::SetNone} }
    );

    assert_eq!(
        from_str::<Action>(r#"<action>  <random/> </action>"#).unwrap(),
        Action { action: BuildableMapAction::Random { use_preview: false } }
    );
    assert_eq!(
        from_str::<Action>(r#"<action> <random use_preview="true"/> </action>"#).unwrap(),
        Action { action: BuildableMapAction::Random { use_preview: true } }
    );

    // TODO: finish adding tests lol

}
