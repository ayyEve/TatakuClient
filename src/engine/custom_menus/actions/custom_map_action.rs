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

    // TODO: document the difference between BeatmapAction::Next and BeatmapListAction::NextSet
    NextMap,
    NextSet,
    PrevMap,
    PrevSet,
}
impl CustomMenuMapAction {
    pub fn into_action(self, values: &mut ValueCollection) -> Option<BeatmapAction> {
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

            Self::SelectGroup(set) => {
                match set {
                    CustomEventValueType::None => return None,
                    CustomEventValueType::Value(v) => {
                        let num = v.as_u64().ok()?;
                        Some(BeatmapAction::ListAction(BeatmapListAction::SelectSet(num as usize)))
                    }
                    CustomEventValueType::Variable(var) => {
                        let num = values.get_raw(&var).ok()?.as_u64().ok()?;
                        Some(BeatmapAction::ListAction(BeatmapListAction::SelectSet(num as usize)))
                    }
                }
            }
        
            Self::SelectMap(map) => {
                match map {
                    CustomEventValueType::None => return None,
                    CustomEventValueType::Value(v) => {
                        let str = v.string_maybe()?;
                        let hash = Md5Hash::try_from(str).ok()?;
                        Some(BeatmapAction::SetFromHash(hash, SetBeatmapOptions::new().use_preview_point(true)))
                    }
                    CustomEventValueType::Variable(var) => {
                        let v = values.get_raw(&var).ok()?;
                        let str = v.string_maybe()?;
                        let hash = Md5Hash::try_from(str).ok()?;
                        Some(BeatmapAction::SetFromHash(hash, SetBeatmapOptions::new().use_preview_point(true)))
                    }
                }
            }
        }
    }

    
    pub fn resolve(&mut self, values: &ValueCollection) {
        let thing = match self {
            Self::SelectGroup(group) => group,
            Self::SelectMap(map) => map,

            _ => return,
        };

        let Some(resolved) = thing.resolve(values) else {
            error!("Couldn't resolve: {:?}", self);
            return;
        };

        *thing = CustomEventValueType::Value(resolved);
    }
}
impl<'lua> FromLua<'lua> for CustomMenuMapAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="custom_menu_debugging")] info!("Reading CustomMenuMapAction");
        match lua_value {
            Value::Table(table) => {
                #[cfg(feature="custom_menu_debugging")] info!("Is table");
                let id:String = table.get("id")?;
                match &*id {
                    "play" => Ok(Self::Play),
                    "next" => Ok(Self::Next),
                    "confirm" => Ok(Self::Confirm),
                    "random" => Ok(Self::Random(table.get::<_, Option<bool>>("use_preview")?.unwrap_or(true))),
                    
                    "next_map" => Ok(Self::NextMap),
                    "next_set" => Ok(Self::NextSet),
                    "prev_map" => Ok(Self::PrevMap),
                    "prev_set" => Ok(Self::PrevSet),

                    "select_group" => Ok(Self::SelectGroup(table.get("group_id")?)),
                    "select_map" => Ok(Self::SelectMap(table.get("map_hash")?)),


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
                #[cfg(feature="custom_menu_debugging")] info!("Is string");
                match action_str.to_str()? {
                    "play" => Ok(Self::Play),
                    "next" => Ok(Self::Next),
                    "confirm" => Ok(Self::Confirm),
                    "random" => Ok(Self::Random(true)),
                    "previous" | "prev" => Ok(Self::Previous(MapActionIfNone::ContinueCurrent)),

                    "next_map" => Ok(Self::NextMap),
                    "next_set" => Ok(Self::NextSet),
                    "prev_map" => Ok(Self::PrevMap),
                    "prev_set" => Ok(Self::PrevSet),

                    other => Err(FromLuaConversionError { from: "String", to: "CustomMenuMapAction", message: Some(format!("Unknown map action {other}")) })
                }
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: "CustomMenuMapAction", message: None })
        }
    
    }
}
