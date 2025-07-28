use crate::prelude::*;

/// An action that deals with the current beatmap
#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildableMapAction {
    /// Play the current map
    Play,

    /// Confirm the current map
    Confirm,

    /// Change to the next map
    Next,

    /// Change to the previous map
    Previous {
        #[serde(rename="$value", alias="$text", default)] 
        action: MapActionIfNone,
    },

    /// Change to a random map
    Random {
        #[serde(rename="@use_preview", default)] 
        use_preview: bool,
    },

    /// Select a specific set by group id
    SelectGroup { 
        #[serde(rename="$value", alias="$text")] 
        value: BuildableValue,
    },

    /// Select a specific map by hash
    SelectMap { 
        #[serde(rename="$value", alias="$text")] 
        value: BuildableValue,
    },

    /// Set the current playmode
    SetPlaymode { 
        #[serde(rename="$value", alias="$text")] 
        value: BuildableValue,
    },

    /// Refresh the beatmap list
    RefreshMaps,

    /// Delete the current map
    DeleteCurrent,

    /// Delete the provided map hash
    Delete {
        #[serde(rename="$value", alias="$text")] 
        value: BuildableValue,
    },


    // TODO: document the difference between BeatmapAction::Next and BeatmapListAction::NextSet
    NextMap,
    NextSet,
    PreviousMap,
    PreviousSet,
}
impl BuildableMapAction {
    pub fn into_action(
        self, 
        values: &mut dyn Reflect, 
        passed_in: Option<&TatakuValue>
    ) -> Option<BeatmapAction> {
        match self {
            Self::Play => Some(BeatmapAction::PlaySelected),
            Self::Confirm => Some(BeatmapAction::ConfirmSelected),

            Self::Next => Some(BeatmapAction::Next),
            Self::Previous { action } 
                => Some(BeatmapAction::Previous(action)),

            Self::Random { use_preview } 
                => Some(BeatmapAction::Random(use_preview)),

            Self::DeleteCurrent 
                => Some(BeatmapAction::DeleteCurrent(PostDelete::Next)),

            Self::Delete { value } => {
                let value = value.resolve(values, passed_in)?;
                let hash = Md5Hash::try_from(value.as_string()).ok()?;
                Some(BeatmapAction::Delete(hash))
            }

            Self::NextMap => Some(BeatmapListAction::NextMap.into()),
            Self::NextSet => Some(BeatmapListAction::NextSet.into()),
            Self::PreviousMap => Some(BeatmapListAction::PrevMap.into()),
            Self::PreviousSet => Some(BeatmapListAction::PrevSet.into()),
            Self::RefreshMaps => Some(BeatmapListAction::Refresh.into()),

            Self::SetPlaymode { value } => {
                let value = value.resolve(values, passed_in)?;
                Some(BeatmapAction::SetPlaymode(value.as_string()))
            }


            Self::SelectGroup { value} => {
                let num = value.resolve(values, passed_in)?.as_u32().ok()?;
                Some(BeatmapAction::ListAction(
                    BeatmapListAction::SelectSet(num as usize)
                ))
            }

            Self::SelectMap { value } => {
                let hash = value
                    .resolve(values, passed_in)?
                    .string_maybe()?
                    .try_into()
                    .ok()?;
                
                Some(BeatmapAction::SetFromHash(
                    hash, 
                    SetBeatmapOptions::new().use_preview_point(true)
                ))
            }
        }
    }

    pub fn build(&mut self, values: &dyn Reflect) {
        match self {
            Self::SelectGroup { value } 
                => value.resolve_pre(values),
                
            Self::SelectMap { value } 
                => value.resolve_pre(values),

            Self::SetPlaymode { value } 
                => value.resolve_pre(values),

            Self::Delete { value } 
                => value.resolve_pre(values),

            _ => {}
        };
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
        Action { action: BuildableMapAction::Previous { action: MapActionIfNone::default()} }
    );
    assert_eq!(
        from_str::<Action>(r#"<action> <previous> <setNone/> </previous> </action>"#).unwrap(),
        Action { action: BuildableMapAction::Previous{ action: MapActionIfNone::SetNone} }
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
