use crate::prelude::*;


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TatakuEventType {
    SongEnd,
    SongPause,
    SongStart,


    // ingame events
    
}
impl<'lua> rlua::FromLua<'lua> for TatakuEventType {
    fn from_lua(lua_value: rlua::prelude::LuaValue<'lua>, _lua: rlua::prelude::LuaContext<'lua>) -> rlua::prelude::LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomMenuEventType");
        
        let rlua::Value::String(str) = lua_value else { return Err(rlua::Error::ToLuaConversionError { from: lua_value.type_name(), to: "CustomMenuEventType", message: Some("Not a string".to_owned()) }) };
        let str = str.to_str()?.to_lowercase();
        
        match &*str {
            "song_start" => Ok(Self::SongStart),
            "song_end" => Ok(Self::SongEnd),

            other => Err(rlua::Error::ToLuaConversionError { from: "String", to: "CustomMenuEventType", message: Some(format!("unknown event: '{other}'")) })
        }
    }
}