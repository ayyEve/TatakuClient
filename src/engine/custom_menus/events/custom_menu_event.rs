use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct CustomMenuEvent {
    pub event_type: TatakuEventType,
    pub action: Option<ButtonAction>,
    pub actions: Option<Vec<ButtonAction>>,
}
impl CustomMenuEvent {
    pub fn get_actions(&self) -> Vec<ButtonAction> {
        let mut actions = self.actions.clone().unwrap_or_default();
        if let Some(action) = self.action.clone() { actions.push(action) }
        actions
    }
}
impl<'lua> rlua::FromLua<'lua> for CustomMenuEvent {
    fn from_lua(lua_value: rlua::prelude::LuaValue<'lua>, _lua: rlua::prelude::LuaContext<'lua>) -> rlua::prelude::LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomMenuEvent");
        
        let rlua::Value::Table(table) = lua_value else { return Err(rlua::Error::ToLuaConversionError { from: lua_value.type_name(), to: "CustomMenuEvent", message: Some("Not a table".to_owned()) }) };
        Ok(Self {
            event_type: table.get("event")?,
            action: table.get("action")?,
            actions: table.get("actions")?,
        })
    }
}