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
        match table.get::<_, Option<Value>>("map")? {
            Some(Value::Table(table)) => {
                let id:String = table.get("id")?;
                match &*id {
                    "play" => action = CustomMenuAction::Map(CustomMenuMapAction::Play),
                    "next" => action = CustomMenuAction::Map(CustomMenuMapAction::Next),
                    "random" => {
                        let use_preview:Option<bool> = table.get("use_preview")?;
                        action = CustomMenuAction::Map(CustomMenuMapAction::Random(use_preview.unwrap_or(true)));
                    }
                    "previous" | "prev" => {
                        let use_preview:Option<bool> = table.get("use_preview")?;
                        let if_none:Option<String> = table.get("if_none")?;
                        let if_none = match if_none.as_deref() {
                            None => MapActionIfNone::ContinueCurrent,
                            Some("continue_current") => MapActionIfNone::ContinueCurrent,
                            Some("random") => MapActionIfNone::Random(use_preview.unwrap_or(true)),

                            Some(other) => return Err(FromLuaConversionError { from: "String", to: "CustomMenuAction", message: Some(format!("Unknown previous map 'if_none' action {other}")) })
                        };

                        action = CustomMenuAction::Map(CustomMenuMapAction::Previous(if_none));
                    }

                    other => return Err(FromLuaConversionError { from: "String", to: "CustomMenuAction", message: Some(format!("Unknown map action {other}")) })
                }
            }

            Some(Value::String(action_str)) => {
                match action_str.to_str()? {
                    "play" => action = CustomMenuAction::Map(CustomMenuMapAction::Play),
                    "next" => action = CustomMenuAction::Map(CustomMenuMapAction::Next),
                    "random" => action = CustomMenuAction::Map(CustomMenuMapAction::Random(true)),

                    other => return Err(FromLuaConversionError { from: "String", to: "CustomMenuAction", message: Some(format!("Unknown map action {other}")) })
                }
            }

            Some(other) => return Err(FromLuaConversionError { from: other.type_name(), to: "CustomMenuAction", message: Some(format!("Invalid map action type")) }),
            
            None => {}
        }

        // song actions
        match table.get::<_, Option<Value>>("song")? {
            Some(Value::Table(table)) => {
                let id:String = table.get("id")?;
                match &*id {
                    "seek" => {
                        let seek:Option<f32> = table.get("seek")?;
                        action = CustomMenuAction::Song(CustomMenuSongAction::Seek(seek.unwrap_or(500.0)));
                    }
                    "position" => {
                        let pos:f32 = table.get("position")?;
                        action = CustomMenuAction::Song(CustomMenuSongAction::SetPosition(pos));
                    }

                    _ => {}
                }
            }
            Some(Value::String(action_str)) => {
                match action_str.to_str()? {
                    "play" => action = CustomMenuAction::Map(CustomMenuMapAction::Play),
                    "next" => action = CustomMenuAction::Map(CustomMenuMapAction::Next),
                    "random" => action = CustomMenuAction::Map(CustomMenuMapAction::Random(true)),

                    other => return Err(FromLuaConversionError { from: "String", to: "CustomMenuAction", message: Some(format!("Unknown song action {other}")) })
                }
            }

            Some(other) => return Err(FromLuaConversionError { from: other.type_name(), to: "CustomMenuAction", message: Some(format!("Invalid song action type")) }),
            None => {}
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

