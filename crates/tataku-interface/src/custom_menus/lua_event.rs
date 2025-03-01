use crate::prelude::*;
use lua::*;

#[derive(Clone, Debug)]
pub struct LuaEvent {
    pub event_type: TatakuEventType,
    pub actions: Vec<LuaAction>,
}
impl LuaEvent {
    pub fn get_actions(&self) -> Vec<LuaAction> {
        self.actions.clone()
    }
}
impl FromLua for LuaEvent {
    fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomMenuEvent");
        
        let LuaValue::Table(table) = lua_value else { 
            return Err(FromLuaConversionError { 
                from: lua_value.type_name(), 
                to: "CustomMenuEvent".to_owned(), 
                message: Some("Not a table".to_owned()) 
            }) 
        };
        
        let action: Option<LuaAction> = table.get("action")?;
        let actions: Option<Vec<LuaAction>> = table.get("actions")?;

        let mut actions = actions.unwrap_or_default();
        if let Some(a) = action { actions.push(a) }

        Ok(Self {
            event_type: table.get("event")?,
            actions
        })
    }
}