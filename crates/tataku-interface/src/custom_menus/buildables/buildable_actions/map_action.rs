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
        action: MapActionIfNone
    },

    /// Change to a random map
    Random {
        #[serde(rename="@use_preview", default)] 
        use_preview: bool,
    },

    /// Select a specific set by group id
    SelectGroup { 
        #[serde(rename="$value", alias="$text")] 
        value: BuildableValue
    },

    /// Select a specific map by hash
    SelectMap { 
        #[serde(rename="$value", alias="$text")] 
        value: BuildableValue
    },

    /// Set the current playmode
    SetPlaymode { 
        #[serde(rename="$value", alias="$text")] 
        value: BuildableValue
    },

    RefreshList,

    // TODO: document the difference between BeatmapAction::Next and BeatmapListAction::NextSet
    NextMap,
    NextSet,
    PreviousMap,
    PreviousSet,
}
impl BuildableMapAction {
    pub fn into_action(self, values: &mut dyn Reflect, passed_in: Option<TatakuValue>) -> Option<BeatmapAction> {
        match self {
            Self::Play => Some(BeatmapAction::PlaySelected),
            Self::Next => Some(BeatmapAction::Next),
            Self::Previous { action } => Some(BeatmapAction::Previous(action)),
            Self::Random{ use_preview } => Some(BeatmapAction::Random(use_preview)),
            Self::Confirm => Some(BeatmapAction::ConfirmSelected),

            Self::NextMap => Some(BeatmapAction::ListAction(BeatmapListAction::NextMap)),
            Self::NextSet => Some(BeatmapAction::ListAction(BeatmapListAction::NextSet)),
            Self::PreviousMap => Some(BeatmapAction::ListAction(BeatmapListAction::PrevMap)),
            Self::PreviousSet => Some(BeatmapAction::ListAction(BeatmapListAction::PrevSet)),
            Self::RefreshList => Some(BeatmapAction::ListAction(BeatmapListAction::Refresh)),


            Self::SetPlaymode { value: BuildableValue::None } => None,
            Self::SetPlaymode { value: BuildableValue::Value(v) } => Some(BeatmapAction::SetPlaymode(v.string_maybe()?.clone())),
            Self::SetPlaymode { value: BuildableValue::Variable(var) } => {
                let val = values.reflect_get::<String>(&var).ok()?;
                Some(BeatmapAction::SetPlaymode((*val).clone()))
            },
            Self::SetPlaymode { value: BuildableValue::PassedIn } => {
                let val = passed_in?.string_maybe()?.clone();
                Some(BeatmapAction::SetPlaymode(val))
            }

            Self::SelectGroup { value} => {
                let num = value.resolve(values, passed_in)?.as_u32().ok()?;
                Some(BeatmapAction::ListAction(BeatmapListAction::SelectSet(num as usize)))
            }

            Self::SelectMap { value } => {
                let hash = value.resolve(values, passed_in)?.string_maybe()?.try_into().ok()?;
                Some(BeatmapAction::SetFromHash(hash, SetBeatmapOptions::new().use_preview_point(true)))
            }
        }
    }

    pub fn build(&mut self, values: &dyn Reflect) {
        let thing = match self {
            Self::SelectGroup { value } => value,
            Self::SelectMap { value } => value,

            _ => return,
        };

        thing.resolve_pre(values);
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
        Action { action: BuildableMapAction::Previous { action: Default::default()} }
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