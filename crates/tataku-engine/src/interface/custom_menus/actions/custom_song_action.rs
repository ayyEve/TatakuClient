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
    Seek(CustomEventValueType),

    /// Set the song's position
    SetPosition(CustomEventValueType),

    /// Set the song's speed
    SetRate(CustomEventValueType),
}
impl CustomMenuSongAction {
    pub fn into_action(self, values: &mut dyn Reflect) -> Option<SongAction> {
        match self {
            Self::Play => Some(SongAction::Play),
            Self::Pause => Some(SongAction::Pause),
            Self::Toggle => Some(SongAction::Toggle),
            Self::Restart => Some(SongAction::Restart),
            Self::Seek(n) => n.resolve(values, None).and_then(|n| n.as_f32().ok()).map(SongAction::SeekBy),
            Self::SetPosition(n) => n.resolve(values, None).and_then(|n| n.as_f32().ok()).map(SongAction::SetPosition),
            Self::SetRate(n) => n.resolve(values, None).and_then(|n| n.as_f32().ok()).map(SongAction::SetRate),
        }
    }


    pub fn build(&mut self, values: &dyn Reflect) {
        let thing = match self {
            Self::Seek(n) => n,
            Self::SetPosition(n) => n,
            Self::SetRate(n) => n,
            _ => return,
        };

        thing.resolve_pre(values);
        // let Some(resolved) = thing.resolve(values, None) else {
        //     error!("Couldn't resolve: {:?}", self);
        //     return;
        // };

        // *thing = CustomEventValueType::Value(resolved);
    }
}
impl<'lua> FromLua<'lua> for CustomMenuSongAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomMenuSongAction");
        match lua_value {
            Value::Table(table) => {
                #[cfg(feature="debug_custom_menus")] info!("Is table");
                if let Some(seek) = table.get("seek")? {
                    Ok(Self::Seek(seek))
                } else if let Some(pos) = table.get("position")? {
                    Ok(Self::SetPosition(pos))
                } else if let Some(rate) = table.get("rate")? {
                    Ok(Self::SetRate(rate))
                } else {
                    Err(FromLuaConversionError { 
                        from: "table", 
                        to: "CustomMenuSongAction", 
                        message: Some("couldn't determine song action".to_string()) 
                    })
                }
            }

            Value::String(str) => {
                #[cfg(feature="debug_custom_menus")] info!("Is String");
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
