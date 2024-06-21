use rlua::{ Value, prelude::LuaResult, Error::FromLuaConversionError };

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TatakuEventType {
    /// Song has ended
    SongEnd,

    /// Song was paused
    SongPause,

    /// Song has started
    SongStart,

    /// Menu was entered
    MenuEnter,
    

    /// A key press
    KeyPress(CustomMenuKeyEvent),
    KeyRelease(CustomMenuKeyEvent),
}
impl<'lua> rlua::FromLua<'lua> for TatakuEventType {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading TatakuEventType");

        match lua_value {
            Value::String(str) => {
                let str = str.to_str()?.to_lowercase();
                
                match &*str {
                    "song_start" => Ok(Self::SongStart),
                    "song_pause" => Ok(Self::SongPause),
                    "song_end" => Ok(Self::SongEnd),
                    "menu_enter" => Ok(Self::MenuEnter),

                    other => Err(FromLuaConversionError { from: "String", to: "TatakuEventType", message: Some(format!("unknown event: '{other}'")) })
                }
            }

            Value::Table(table) => {
                if let Ok(e) = table.get::<_, CustomMenuKeyEvent>("key_press") {
                    Ok(Self::KeyPress(e))
                } else if let Ok(e) = table.get::<_, CustomMenuKeyEvent>("key_release") {
                    Ok(Self::KeyRelease(e))
                } else{ 
                    Err(FromLuaConversionError { from: "Table", to: "TatakuEventType", message: Some("Unknown event type".to_owned()) })
                }
            }


            _ => Err(FromLuaConversionError { from: lua_value.type_name(), to: "TatakuEventType", message: Some("Not a table or string".to_owned()) })
        }
        
    }
}



#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CustomMenuKeyEvent {
    /// what key?
    pub key: crate::prelude::Key,

    /// must control be pressed
    pub control: bool,

    /// must alt be pressed
    pub alt: bool,

    /// must shift be pressed
    pub shift: bool,
}
impl<'lua> rlua::FromLua<'lua> for CustomMenuKeyEvent {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomMenuKeyEvent");
        let Value::Table(table) = lua_value else { return Err(FromLuaConversionError { from: lua_value.type_name(), to: "CustomMenuKeyEvent", message: None }) }; 
        
        #[cfg(feature="debug_custom_menus")] info!("Reading key");
        let key = table.get("key")?;
        let key = serde_json::from_value(serde_json::Value::String(key))
            .map_err(|e| FromLuaConversionError { from: "String", to: "Key", message: Some(e.to_string()) })?;

        let mut out = Self {
            key,
            control: false,
            alt: false,
            shift: false,
        };

        #[cfg(feature="debug_custom_menus")] info!("Reading mods");
        if let Some(incoming_mods) = table.get::<_, Option<Vec<String>>>("mods")? {
            for m in incoming_mods { 
                match &*m {
                    "ctrl" | "control" => out.control = true,
                    "alt" => out.alt = true,
                    "shift" => out.shift = true,
                    _ => {}
                }
            }
        }

        Ok(out)
    }
}