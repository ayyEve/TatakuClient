use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct CustomMenu {
    pub id: String,
    pub element: ElementDef,
}
impl CustomMenu {
    pub async fn build(&self) -> BuiltCustomMenu {
        BuiltCustomMenu {
            id: self.id.clone(),
            element: self.element.build().await,
            queued_actions: ActionQueue::new(),
        }
    }
}
impl<'lua> rlua::FromLua<'lua> for CustomMenu {
    fn from_lua(lua_value: rlua::prelude::LuaValue<'lua>, _lua: rlua::prelude::LuaContext<'lua>) -> rlua::prelude::LuaResult<Self> {
        let rlua::Value::Table(table) = lua_value else { return Err(rlua::Error::ToLuaConversionError { from: lua_value.type_name(), to: "CustomMenu", message: Some("Not a table".to_owned()) }) };
        
        Ok(Self {
            id: table.get("id")?,
            element: table.get("element")?,
        })
    }
}


pub struct BuiltCustomMenu {
    pub id: String,
    pub element: BuiltElementDef,
    pub queued_actions: ActionQueue,
}

#[async_trait]
impl AsyncMenu for BuiltCustomMenu {
    fn get_name(&self) -> &'static str { "custom_menu" }
    fn get_custom_name(&self) -> Option<&String> { Some(&self.id) }

    fn view(&self, values: &mut ShuntingYardValues) -> IcedElement {
        self.element.view(MessageOwner::new_menu(self), values)
            // .explain(Color::TEAL)
            .into_element()
    }

    async fn update(&mut self) -> Vec<MenuAction> {
        self.element.update().await;
        self.queued_actions.take()
    }

    async fn handle_message(&mut self, message: Message, values: &mut ShuntingYardValues) {
        match message.message_type {
            MessageType::Text(incoming) => {
                let Some(variable) = message.tag.as_string() else { return };
                values.set(variable, incoming);
            }
            // MessageType::CustomMenuAction(AddDialog(dialog)) => {}
            // MessageType::CustomMenuAction(SetMenu(menu)) => self.queued_actions.push(MenuMenuAction::SetMenuCustom(menu)),

            MessageType::CustomMenuAction(action) => self.queued_actions.push(action),

            _ => {}
        }
    }
}



pub struct CustomMenuEvent {
    pub event_type: CustomMenuEventType,
    pub action: CustomMenuAction
}

pub enum CustomMenuEventType {
    SongEnd,
    
}
