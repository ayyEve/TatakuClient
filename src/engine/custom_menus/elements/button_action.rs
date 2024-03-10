use crate::prelude::*;
use rlua::{ Value, FromLua, Error::FromLuaConversionError };

#[derive(Clone, Debug)]
pub struct ButtonAction {
    pub action: CustomMenuAction,
    pub context: ButtonActionContext,
}
impl ButtonAction {
    pub fn into_message(&self, owner: MessageOwner) -> Option<Message> {
        // use CustomMenuAction::*;
        if let CustomMenuAction::None = &self.action { return None };

        let message = MessageType::CustomMenuAction(self.action.clone());
        Some(Message::new(owner, "", message))

        // match &self.action {
        //     CustomMenuAction::None => Option::None,
        //     SetMenu(name) => Some(owner.click(format!("set-menu-{name}"))),
        //     AddDialog(name) => Some(owner.click(format!("add-dialog-{name}"))),
            
        //     MapNext => Some(owner.click("set-map-next")),
        //     MapPrevious => Some(owner.click("set-map-prev")),
        //     SongSeek(amount) => Some(owner.float("set-song-seek", val))
        // }
    }
}
impl<'lua> FromLua<'lua> for ButtonAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        let Value::Table(table) = lua_value else { return Err(FromLuaConversionError { from: lua_value.type_name(), to: "CustomMenuAction", message: Some("Not a table".to_owned()) }) };
    
        let mut action = CustomMenuAction::None;
        
        // menu actions
        if let Some(action_str) = table.get::<_, Option<String>>("menu")? {
            action = CustomMenuAction::SetMenu(action_str);
        }

        // dialog actions
        if let Some(action_str) = table.get::<_, Option<String>>("dialog")? {
            action = CustomMenuAction::AddDialog(action_str);
        }

        // beatmap actions
        if let Ok(Some(map_action)) = table.get::<_, Option<CustomMenuMapAction>>("map") {
            action = CustomMenuAction::Map(map_action);
        }

        // song actions
        if let Ok(Some(song_action)) = table.get::<_, Option<CustomMenuSongAction>>("song") {
            action = CustomMenuAction::Song(song_action);
        }

        Ok(Self {
            action,
            context: ButtonActionContext::Empty
        })
    }
}


#[derive(Clone, Debug)]
pub enum ButtonActionContext {
    Empty,
    Array(Vec<String>),
    // Other(Box<dyn std::any::Any + Send + Sync>),
}
