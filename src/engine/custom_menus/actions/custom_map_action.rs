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
}
impl CustomMenuMapAction {
    pub fn into_action(self, _values: &mut ValueCollection) -> BeatmapAction {
        match self {
            Self::Play => BeatmapAction::PlaySelected,
            Self::Next => BeatmapAction::Next,
            Self::Previous(action) => BeatmapAction::Previous(action),
            Self::Random(use_preview) => BeatmapAction::Random(use_preview),
            Self::Confirm => BeatmapAction::ConfirmSelected,
        }
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

                    other => Err(FromLuaConversionError { from: "String", to: "CustomMenuMapAction", message: Some(format!("Unknown map action {other}")) })
                }
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: "CustomMenuMapAction", message: None })
        }
    
    }
}
