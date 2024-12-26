use crate::prelude::*;
use rlua::{ Value, FromLua, Error::FromLuaConversionError };


/// An action that deals with the current beatmap
#[derive(Clone, Debug)]
pub enum CustomMenuMapAction {
    /// Play the current map
    Play,

    /// Confirm the current map
    Confirm,

    /// Change to the next map
    Next,

    /// Change to the previous map
    Previous(MapActionIfNone),

    /// Change to a random map
    Random(bool),

    /// Select a specific set by group id
    SelectGroup(CustomEventValueType),

    /// Select a specific map by hash
    SelectMap(CustomEventValueType),

    /// Set the current playmode
    SetPlaymode(CustomEventValueType),

    RefreshList,

    // TODO: document the difference between BeatmapAction::Next and BeatmapListAction::NextSet
    NextMap,
    NextSet,
    PrevMap,
    PrevSet,
}
impl CustomMenuMapAction {
    pub fn into_action(self, values: &mut dyn Reflect, passed_in: Option<TatakuValue>) -> Option<BeatmapAction> {
        match self {
            Self::Play => Some(BeatmapAction::PlaySelected),
            Self::Next => Some(BeatmapAction::Next),
            Self::Previous(action) => Some(BeatmapAction::Previous(action)),
            Self::Random(use_preview) => Some(BeatmapAction::Random(use_preview)),
            Self::Confirm => Some(BeatmapAction::ConfirmSelected),

            Self::NextMap => Some(BeatmapAction::ListAction(BeatmapListAction::NextMap)),
            Self::NextSet => Some(BeatmapAction::ListAction(BeatmapListAction::NextSet)),
            Self::PrevMap => Some(BeatmapAction::ListAction(BeatmapListAction::PrevMap)),
            Self::PrevSet => Some(BeatmapAction::ListAction(BeatmapListAction::PrevSet)),
            Self::RefreshList => Some(BeatmapAction::ListAction(BeatmapListAction::Refresh)),


            Self::SetPlaymode(CustomEventValueType::None) => None,
            Self::SetPlaymode(CustomEventValueType::Value(v)) => Some(BeatmapAction::SetPlaymode(v.string_maybe()?.clone())),
            Self::SetPlaymode(CustomEventValueType::Variable(var)) => {
                let val = values.reflect_get::<String>(&var).ok()?;
                Some(BeatmapAction::SetPlaymode((*val).clone()))
            },
            Self::SetPlaymode(CustomEventValueType::PassedIn) => {
                let val = passed_in?.string_maybe()?.clone();
                Some(BeatmapAction::SetPlaymode(val))
            }

            Self::SelectGroup(set) => {
                // let num = match set {
                //     CustomEventValueType::None => return None,
                //     CustomEventValueType::Value(v) => v.as_u64().ok()?,
                //     CustomEventValueType::Variable(var) => values.get_raw(&var).ok()?.as_u64().ok()?,
                //     CustomEventValueType::PassedIn => passed_in?.as_u64().ok()?,
                // };
                let num = set.resolve(values, passed_in)?.as_u32().ok()?;
                Some(BeatmapAction::ListAction(BeatmapListAction::SelectSet(num as usize)))
            }

            Self::SelectMap(map) => {
                // let mut hash = None;

                let hash = map.resolve(values, passed_in)?.string_maybe()?.try_into().ok()?;
                // if let Some(str) = map.resolve(values, passed_in)?.string_maybe()? {
                //     hash = Md5Hash::try_from(str).ok();
                // }

                // if hash.is_none() {
                //     hash = map.resolve(values, passed_in)?.downcast_ref::<Md5Hash>().copied();
                // }
                // let hash = hash?;

                // let hash = match map {
                //     CustomEventValueType::None => return None,
                //     CustomEventValueType::Value(v) => Md5Hash::try_from(v.string_maybe()?).ok()?,
                //     CustomEventValueType::PassedIn => Md5Hash::try_from(passed_in?.string_maybe()?).ok()?,

                //     CustomEventValueType::Variable(var) => {
                //         let v = values.get_raw(&var).ok()?;
                //         let str = v.string_maybe()?;
                //         Md5Hash::try_from(str).ok()?
                //     }
                // };

                Some(BeatmapAction::SetFromHash(hash, SetBeatmapOptions::new().use_preview_point(true)))
            }
        }
    }


    pub fn build(&mut self, values: &dyn Reflect) {
        let thing = match self {
            Self::SelectGroup(group) => group,
            Self::SelectMap(map) => map,

            _ => return,
        };

        thing.resolve_pre(values);
        // let Some(resolved) = thing.resolve_pre(values) else {
        //     error!("Couldn't resolve: {:?}", self);
        //     return;
        // };

        // *thing = CustomEventValueType::Value(resolved);
    }
}
impl<'lua> FromLua<'lua> for CustomMenuMapAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomMenuMapAction");
        match lua_value {
            Value::Table(table) => {
                #[cfg(feature="debug_custom_menus")] info!("Is table");
                let id:String = table.get("id")?;
                match &*id {
                    "play" => Ok(Self::Play),
                    "next" => Ok(Self::Next),
                    "confirm" => Ok(Self::Confirm),
                    "random" => Ok(Self::Random(table.get::<_, Option<bool>>("use_preview")?.unwrap_or(true))),
                    "refres" | "refresh_list" => Ok(Self::RefreshList),

                    "next_map" => Ok(Self::NextMap),
                    "next_set" => Ok(Self::NextSet),
                    "previous_map" | "prev_map" => Ok(Self::PrevMap),
                    "previous_set" | "prev_set" => Ok(Self::PrevSet),

                    "select_group" => Ok(Self::SelectGroup(table.get("group_id")?)),
                    "select_map" => Ok(Self::SelectMap(table.get("map_hash")?)),
                    "set_playmode" => Ok(Self::SetPlaymode(table.get("playmode")?)),


                    "previous" | "prev" => {
                        let use_preview:Option<bool> = table.get("use_preview")?;
                        let if_none:Option<String> = table.get("if_none")?;
                        let if_none = match if_none.as_deref() {
                            None => MapActionIfNone::ContinueCurrent,
                            Some("continue_current") => MapActionIfNone::ContinueCurrent,
                            Some("random") => MapActionIfNone::Random(use_preview.unwrap_or(true)),

                            Some(other) => return Err(FromLuaConversionError { from: "String", to: "MapActionIfNone", message: Some(format!("Unknown MapActionIfNone value {other}")) })
                        };

                        Ok(Self::Previous(if_none))
                    }

                    other => Err(FromLuaConversionError { from: "String", to: "CustomMenuMapAction", message: Some(format!("Unknown map action {other}")) })
                }
            }
            Value::String(action_str) => {
                #[cfg(feature="debug_custom_menus")] info!("Is string");
                match action_str.to_str()? {
                    "play" => Ok(Self::Play),
                    "next" => Ok(Self::Next),
                    "confirm" => Ok(Self::Confirm),
                    "random" => Ok(Self::Random(true)),
                    "previous" | "prev" => Ok(Self::Previous(MapActionIfNone::ContinueCurrent)),
                    "refres" | "refresh_list" => Ok(Self::RefreshList),

                    "next_map" => Ok(Self::NextMap),
                    "next_set" => Ok(Self::NextSet),
                    "previous_map" | "prev_map" => Ok(Self::PrevMap),
                    "previous_set" | "prev_set" => Ok(Self::PrevSet),

                    other => Err(FromLuaConversionError { from: "String", to: "CustomMenuMapAction", message: Some(format!("Unknown map action {other}")) })
                }
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: "CustomMenuMapAction", message: None })
        }

    }
}
