use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct CustomMenuEvent {
    pub event_type: TatakuEventType,
    pub actions: Vec<ButtonAction>,
}
impl CustomMenuEvent {
    pub fn get_actions(&self) -> Vec<ButtonAction> {
        self.actions.clone()
    }
}
impl<'lua> rlua::FromLua<'lua> for CustomMenuEvent {
    fn from_lua(lua_value: rlua::prelude::LuaValue<'lua>, _lua: rlua::Context<'lua>) -> rlua::prelude::LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomMenuEvent");
        
        let rlua::Value::Table(table) = lua_value else { return Err(rlua::Error::ToLuaConversionError { from: lua_value.type_name(), to: "CustomMenuEvent", message: Some("Not a table".to_owned()) }) };
        
        let action: Option<ButtonAction> = table.get("action")?;
        let actions: Option<Vec<ButtonAction>> = table.get("actions")?;

        let mut actions = actions.unwrap_or_default();
        action.map(|a| actions.push(a));

        Ok(Self {
            event_type: table.get("event")?,
            actions
        })
    }
}