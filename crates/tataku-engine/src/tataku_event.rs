use crate::prelude::*;
use lua::*;

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
    
    // Menu was left
    MenuLeft,

    /// A new beatmap has been added
    MapAdded,

    /// A key press
    KeyPress(CustomMenuKeyEvent),

    /// A key release
    KeyRelease(CustomMenuKeyEvent),

    /// A controller button was pressed
    ControllerPress(CustomMenuControllerEvent),

    /// A controller button was released
    ControllerRelease(CustomMenuControllerEvent),
}
impl FromLua for TatakuEventType {
    fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading TatakuEventType");

        match lua_value {
            LuaValue::String(str) => {
                let str = str.to_str()?.to_lowercase();

                match &*str {
                    "song_start" => Ok(Self::SongStart),
                    "song_pause" => Ok(Self::SongPause),
                    "song_end" => Ok(Self::SongEnd),
                    "menu_enter" => Ok(Self::MenuEnter),
                    "map_added" | "beatmap_added" => Ok(Self::MapAdded),

                    other => Err(FromLuaConversionError { 
                        from: "String", 
                        to: "TatakuEventType".to_owned(), 
                        message: Some(format!("unknown event: '{other}'")) 
                    })
                }
            }

            LuaValue::Table(table) => {
                // key
                if let Ok(e) = table.get::<CustomMenuKeyEvent>("key_press") {
                    Ok(Self::KeyPress(e))
                } else if let Ok(e) = table.get::<CustomMenuKeyEvent>("key_release") {
                    Ok(Self::KeyRelease(e))
                }
                // controller
                else if let Ok(e) = table.get::<CustomMenuControllerEvent>("controller_press") {
                    Ok(Self::ControllerPress(e))
                } else if let Ok(e) = table.get::<CustomMenuControllerEvent>("controller_release") {
                    Ok(Self::ControllerRelease(e))
                }

                else {
                    Err(FromLuaConversionError { 
                        from: "Table", 
                        to: "TatakuEventType".to_owned(), 
                        message: Some("Unknown event type".to_owned()) 
                    })
                }
            }


            _ => Err(FromLuaConversionError { 
                from: lua_value.type_name(), 
                to: "TatakuEventType".to_owned(), 
                message: Some("Not a table or string".to_owned()) 
            })
        }

    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CustomMenuKeyEvent {
    /// What key?
    pub key: crate::prelude::Key,

    /// Must control be pressed?
    pub control: bool,

    /// Must alt be pressed?
    pub alt: bool,

    /// Must shift be pressed?
    pub shift: bool,
}
impl FromLua for CustomMenuKeyEvent {
    fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] crate::info!("Reading CustomMenuKeyEvent");
        let LuaValue::Table(table) = lua_value else { 
            return Err(FromLuaConversionError { 
                from: lua_value.type_name(), 
                to: "CustomMenuKeyEvent".to_owned(), 
                message: None
            }) 
        };

        #[cfg(feature="debug_custom_menus")] crate::info!("Reading key");
        let key = table.get("key")?;
        let key = serde_json::from_value(serde_json::Value::String(key))
            .map_err(|e| FromLuaConversionError { 
                from: "String", 
                to: "Key".to_owned(), 
                message: Some(e.to_string()) 
            })?;

        let mut out = Self {
            key,
            control: false,
            alt: false,
            shift: false,
        };

        #[cfg(feature="debug_custom_menus")] crate::info!("Reading mods");
        if let Some(incoming_mods) = table.get::<Option<Vec<String>>>("mods")? {
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



#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CustomMenuControllerEvent {
    pub button: ControllerButton,
}
impl FromLua for CustomMenuControllerEvent {
    fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        const SELF: &str = "CustomMenuControllerEvent";
        #[cfg(feature="debug_custom_menus")] crate::info!("Reading {SELF}");

        match lua_value {
            LuaValue::String(s) => {
                let s = s.to_str()?;
                let button = ControllerButton::from_string(&s);
                if button == ControllerButton::Unknown {
                    return Err(FromLuaConversionError {
                        from: "String",
                        to: "ControllerButton".to_owned(),
                        message: Some(format!("Invalid value {s}"))
                    });
                }

                Ok(Self {
                    button,
                })
            }

            other => Err(FromLuaConversionError { 
                from: other.type_name(), 
                to: SELF.to_owned(), 
                message: Some("invalid type".to_string()) 
            }),
        }

    }
}

