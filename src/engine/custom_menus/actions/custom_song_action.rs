use crate::prelude::*;
use rlua::{ Value, FromLua, Error::FromLuaConversionError };


/// An action that deals with the Song
#[derive(Clone, Debug)]
pub enum CustomMenuSongAction {
    /// Play/resume the song
    Play,

    /// Pause the song
    Pause,

    /// Toggle the song (pause if playing, play if paused)
    Toggle,

    /// Restart the song from the beginning
    Restart,

    /// Seek by the specified number of ms
    Seek(f32),

    /// Set the song's position
    SetPosition(f32),
}
impl CustomMenuSongAction {
    pub fn into_action(self, _values: &mut ValueCollection) -> SongAction {
        match self {
            Self::Play => SongAction::Play,
            Self::Pause => SongAction::Pause,
            Self::Toggle => SongAction::Toggle,
            Self::Restart => SongAction::Restart,
            Self::Seek(n) => SongAction::SeekBy(n),
            Self::SetPosition(p) => SongAction::SetPosition(p)
        }
    }
}
impl<'lua> FromLua<'lua> for CustomMenuSongAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="custom_menu_debugging")] info!("Reading CustomMenuSongAction");
        match lua_value {
            Value::Table(table) => {
                #[cfg(feature="custom_menu_debugging")] info!("Is table");
                if let Some(seek) = table.get("seek")? {
                    Ok(Self::Seek(seek))
                } else if let Some(pos) = table.get("position")?{
                    Ok(Self::SetPosition(pos))
                } else {
                    Err(FromLuaConversionError { 
                        from: "table", 
                        to: "CustomMenuSongAction", 
                        message: Some(format!("couldn't determine song action")) 
                    })
                }
            }

            Value::String(str) => {
                #[cfg(feature="custom_menu_debugging")] info!("Is String");
                match str.to_str()? {
                    "play" => Ok(Self::Play),
                    "pause" => Ok(Self::Pause),
                    "toggle" => Ok(Self::Toggle),
                    "restart" => Ok(Self::Restart),

                    other => Err(FromLuaConversionError { 
                        from: "String", 
                        to: "CustomMenuSongAction", 
                        message: Some(format!("Unknown action: {other}")) 
                    }),
                }
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: "CustomMenuSongAction", message: None })
        }
    
    }
}
