use crate::prelude::*;
use rlua::{ Value, FromLua, Error::FromLuaConversionError };

#[derive(Clone, Debug)]
pub enum CustomMenuAction {
    /// No action
    None,

    /// Set the menu
    SetMenu(String),

    /// Add a dialog
    AddDialog(String),

    /// Perform a map action
    Map(CustomMenuMapAction),

    /// Perform a song action
    Song(CustomMenuSongAction),
}
impl Into<MenuAction> for CustomMenuAction {
    fn into(self) -> MenuAction {
        match self {
            Self::None => MenuAction::None,
            Self::AddDialog(dialog) => MenuAction::Menu(MenuMenuAction::AddDialogCustom(dialog, true)),
            Self::SetMenu(menu) => MenuAction::Menu(MenuMenuAction::SetMenuCustom(menu)),

            Self::Map(action) => MenuAction::Beatmap(action.into()),
            Self::Song(action) => MenuAction::Song(action.into()),
        }
    }
}
impl<'lua> FromLua<'lua> for CustomMenuAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        let Value::Table(table) = lua_value else { return Err(FromLuaConversionError { from: lua_value.type_name(), to: "CustomMenuAction", message: Some("Not a table".to_owned()) }) };
    
        // menu actions
        if let Some(action_str) = table.get::<_, Option<String>>("menu")? {
            Ok(Self::SetMenu(action_str))
        }
        // dialog actions
        else if let Some(action_str) = table.get::<_, Option<String>>("dialog")? {
            Ok(Self::AddDialog(action_str))
        }
        // beatmap actions
        else if let Some(map_action) = table.get::<_, Option<CustomMenuMapAction>>("map")? {
            Ok(Self::Map(map_action))
        }
        // song actions
        else if let Some(song_action) = table.get::<_, Option<CustomMenuSongAction>>("song")? {
            Ok(Self::Song(song_action))
        }
        // nope
        else {
            Err(FromLuaConversionError { 
                from: "Table", 
                to: "CustomMenuAction", 
                message: Some("Could not determine action".to_owned()) 
            })
        }
    }
}




/// An action that deals with the current beatmap
#[derive(Clone, Debug)]
pub enum CustomMenuMapAction {
    /// Play the current map
    Play,

    /// Change to the next map
    Next,

    /// Change to the previous map
    Previous(MapActionIfNone),

    /// Change to a random map
    Random(bool),
}
impl Into<BeatmapMenuAction> for CustomMenuMapAction {
    fn into(self) -> BeatmapMenuAction {
        match self {
            Self::Play => BeatmapMenuAction::PlaySelected,
            Self::Next => BeatmapMenuAction::Next,
            Self::Previous(action) => BeatmapMenuAction::Previous(action),
            Self::Random(use_preview) => BeatmapMenuAction::Random(use_preview),
        }
    }
}
impl<'lua> FromLua<'lua> for CustomMenuMapAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        match lua_value {
            Value::Table(table) => {
                let id:String = table.get("id")?;
                match &*id {
                    "play" => Ok(Self::Play),
                    "next" => Ok(Self::Next),
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
                match action_str.to_str()? {
                    "play" => Ok(Self::Play),
                    "next" => Ok(Self::Next),
                    "random" => Ok(Self::Random(true)),
                    "previous" | "prev" => Ok(Self::Previous(MapActionIfNone::ContinueCurrent)),

                    other => Err(FromLuaConversionError { from: "String", to: "CustomMenuMapAction", message: Some(format!("Unknown map action {other}")) })
                }
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: "CustomMenuMapAction", message: None })
        }
    
    }
}



/// An action that deals with the Song
#[derive(Clone, Debug)]
pub enum CustomMenuSongAction {
    /// Play/resume the song
    Play,

    /// Pause the song
    Pause,

    /// Restart the song from the beginning
    Restart,

    /// Seek by the specified number of ms
    Seek(f32),

    /// Set the song's position
    SetPosition(f32),
}
impl Into<SongMenuAction> for CustomMenuSongAction {
    fn into(self) -> SongMenuAction {
        match self {
            Self::Play => SongMenuAction::Play,
            Self::Pause => SongMenuAction::Pause,
            Self::Restart => SongMenuAction::Restart,
            Self::Seek(n) => SongMenuAction::SeekBy(n),
            Self::SetPosition(p) => SongMenuAction::SetPosition(p)
        }
    }
}

impl<'lua> FromLua<'lua> for CustomMenuSongAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        match lua_value {
            Value::Table(table) => {
                let id:String = table.get("id")?;
                match &*id {
                    "seek" => Ok(Self::Seek(table.get::<_, Option<f32>>("seek")?.unwrap_or(500.0))),
                    "position" => Ok(Self::SetPosition(table.get::<_, f32>("position")?)),
                    other => Err(FromLuaConversionError { from: "String", to: "CustomMenuSongAction", message: Some(format!("Unknown song action {other}")) }),
                }
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: "CustomMenuSongAction", message: None })
        }
    
    }
}
