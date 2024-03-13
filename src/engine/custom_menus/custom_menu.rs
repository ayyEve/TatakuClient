use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct CustomMenu {
    pub id: String,
    pub element: ElementDef,
    pub components: Vec<ComponentDef>
}
impl CustomMenu {
    pub async fn build(&self) -> BuiltCustomMenu {
        let mut components = Vec::new();
        for i in self.components.iter() {
            components.push(i.build().await);
        }

        BuiltCustomMenu {
            id: self.id.clone(),
            element: *self.element.build().await,
            actions: ActionQueue::new(),
            components,
        }
    }
}
impl<'lua> rlua::FromLua<'lua> for CustomMenu {
    fn from_lua(lua_value: rlua::prelude::LuaValue<'lua>, _lua: rlua::prelude::LuaContext<'lua>) -> rlua::prelude::LuaResult<Self> {
        #[cfg(feature="custom_menu_debugging")] info!("Reading CustomMenu");
        let rlua::Value::Table(table) = lua_value else { return Err(rlua::Error::ToLuaConversionError { from: lua_value.type_name(), to: "CustomMenu", message: Some("Not a table".to_owned()) }) };
        
        Ok(Self {
            id: table.get("id")?,
            element: table.get("element")?,
            components: table.get::<_, Option<Vec<_>>>("components")?.unwrap_or_default(),
        })
    }
}


pub struct BuiltCustomMenu {
    pub id: String,
    pub element: BuiltElementDef,
    pub actions: ActionQueue,
    pub components: Vec<Box<dyn Widgetable>>,
}

#[async_trait]
impl AsyncMenu for BuiltCustomMenu {
    fn get_name(&self) -> &'static str { "custom_menu" }
    fn get_custom_name(&self) -> Option<&String> { Some(&self.id) }

    fn view(&self, values: &mut ShuntingYardValues) -> IcedElement {
        let view = self.element.view(MessageOwner::new_menu(self), values);

        if let Some(debug_color) = self.element.element.debug_color {
            view
                .explain(debug_color)
                .into_element()
        } else {
            view
        }
    }

    async fn update(&mut self, values: &mut ShuntingYardValues) -> Vec<MenuAction> {
        self.element.update(values, &mut self.actions).await;
        for i in self.components.iter_mut() {
            i.update(values, &mut self.actions).await;
        }
        self.actions.take()
    }

    async fn handle_message(&mut self, message: Message, values: &mut ShuntingYardValues) {

        for i in self.components.iter_mut() {
            let actions = i.handle_message(&message, values).await;
            if !actions.is_empty() {
                self.actions.extend(actions);
                return;
            }
        }
        
        match message.message_type {
            MessageType::Text(incoming) => {
                let Some(variable) = message.tag.as_string() else { return };
                values.set(variable, incoming);
            }
            // MessageType::CustomMenuAction(AddDialog(dialog)) => {}
            // MessageType::CustomMenuAction(SetMenu(menu)) => self.queued_actions.push(MenuMenuAction::SetMenuCustom(menu)),

            MessageType::CustomMenuAction(action) => self.actions.push(action.into_action(values)),

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
