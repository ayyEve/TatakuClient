use crate::prelude::*;
use lua::*;

#[derive(Clone, Debug)]
pub struct CustomMenu {
    pub id: String,
    pub element: ElementDef,
    pub events: Vec<LuaEvent>,
}
impl FromLua for CustomMenu {
    fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("=======================");
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomMenu");
        #[cfg(feature="debug_custom_menus")] info!("=======================");

        let LuaValue::Table(table) = lua_value else { 
            return Err(FromLuaConversionError { 
                from: lua_value.type_name(), 
                to: "CustomMenu".to_owned(), 
                message: Some("Not a table".to_owned()) 
            }) 
        };

        let id = table.get("id")?;
        #[cfg(feature="debug_custom_menus")] info!("Got id '{id}'");

        Ok(Self {
            id,
            element: table.get("element")?,
            events: table.get::<Option<Vec<_>>>("events")?.unwrap_or_default(),
        })
    }
}
