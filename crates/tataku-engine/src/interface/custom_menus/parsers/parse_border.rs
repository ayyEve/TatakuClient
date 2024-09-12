use super::prelude::*;

impl<'lua> FromLua<'lua> for Border {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading Border");
        let Value::Table(table) = lua_value else { return Err(FromLuaConversionError { from: "Not Table", to: "Border", message: Some("Not a table".to_owned()) }) }; 
        
        Ok(Border {
            color: table.get::<_, super::parse_color::LuaColor>("color")?.0,
            radius: table.get("radius")?,
        })
    }
}
