use super::prelude::*;

impl<'lua> FromLua<'lua> for Border {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::prelude::LuaContext<'lua>) -> rlua::Result<Self> {
        let Value::Table(table) = lua_value else { return Err(FromLuaConversionError { from: "Not Table", to: "Border", message: Some("Not a table".to_owned()) }) }; 
        
        Ok(Border {
            color: table.get("color")?,
            radius: table.get("radius")?,
        })
    }
}
