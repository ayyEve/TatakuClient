use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct CustomMenu {
    pub id: String,
    pub element: ElementDef,
    pub components: Vec<ComponentDef>,
    pub events: Vec<CustomMenuEvent>,
}
impl CustomMenu {
    pub async fn build(&self) -> BuiltCustomMenu {
        let mut components = Vec::new();
        for i in self.components.iter() {
            components.push(i.build().await);
        }

        let mut events:HashMap<TatakuEventType, Vec<ButtonAction>> = HashMap::new();
        for event in self.events.clone() {
            let list = events.entry(event.event_type).or_default();
            for mut action in event.get_actions() {
                action.build();
                list.push(action);
            }
        }

        BuiltCustomMenu {
            id: self.id.clone(),
            element: *self.element.build().await,
            actions: ActionQueue::new(),
            components,
            events,
        }
    }
}
impl<'lua> rlua::FromLua<'lua> for CustomMenu {
    fn from_lua(lua_value: rlua::prelude::LuaValue<'lua>, _lua: rlua::prelude::LuaContext<'lua>) -> rlua::prelude::LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("=======================");
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomMenu");
        #[cfg(feature="debug_custom_menus")] info!("=======================");
        
        let rlua::Value::Table(table) = lua_value else { return Err(rlua::Error::ToLuaConversionError { from: lua_value.type_name(), to: "CustomMenu", message: Some("Not a table".to_owned()) }) };
        let id = table.get("id")?;
        #[cfg(feature="debug_custom_menus")] info!("Got id '{id}'");

        Ok(Self {
            id,
            element: table.get("element")?,
            components: table.get::<_, Option<Vec<_>>>("components")?.unwrap_or_default(),
            events: table.get::<_, Option<Vec<_>>>("events")?.unwrap_or_default(),
        })
    }
}


pub struct BuiltCustomMenu {
    pub id: String,
    pub element: BuiltElementDef,
    pub actions: ActionQueue,
    pub components: Vec<Box<dyn Widgetable>>,
    pub events: HashMap<TatakuEventType, Vec<ButtonAction>>,
}

#[async_trait]
impl AsyncMenu for BuiltCustomMenu {
    fn get_name(&self) -> &'static str { "custom_menu" }
    fn get_custom_name(&self) -> Option<&String> { Some(&self.id) }

    fn view(&self, values: &mut ValueCollection) -> IcedElement {
        let view = self.element.view(MessageOwner::new_menu(self), values);

        if let Some(debug_color) = self.element.element.debug_color {
            view
                .explain(debug_color)
                .into_element()
        } else {
            view
        }
    }

    async fn update(&mut self, values: &mut ValueCollection) -> Vec<TatakuAction> {
        self.element.update(values, &mut self.actions).await;
        for i in self.components.iter_mut() {
            i.update(values, &mut self.actions).await;
        }
        self.actions.take()
    }

    async fn handle_message(&mut self, message: Message, values: &mut ValueCollection) {

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
                values.update_or_insert(&variable, TatakuVariableWriteSource::Menu, incoming, || TatakuVariable::new_any(TatakuValue::None));

                // values.set(variable, incoming);
            }
            // MessageType::CustomMenuAction(AddDialog(dialog)) => {}
            // MessageType::CustomMenuAction(SetMenu(menu)) => self.queued_actions.push(MenuMenuAction::SetMenuCustom(menu)),

            MessageType::CustomMenuAction(action, passed_in) => {
                if let Some(action) = action.into_action(values, passed_in) {
                    self.actions.push(action)
                }
            }
            MessageType::Multi(actions) => {
                for i in actions {
                    self.handle_message(i, values).await;
                }
            }

            other => warn!("unhandled message: {other:?}"),
        }
    }

    async fn handle_event(&mut self, event: TatakuEventType, event_value: Option<TatakuValue>, values: &mut ValueCollection) {
        let Some(events) = self.events.get(&event) else { return };
        let owner = MessageOwner::new_menu(self);

        for i in events.iter() {
            let Some(message) = i.resolve(owner, values, event_value.clone()) else { continue };
            match message.message_type {
                MessageType::CustomMenuAction(action, passed_in) => {
                    let Some(a) = action.into_action(values, passed_in) else { continue };
                    self.actions.push(a);
                }

                _ => self.actions.push(TatakuAction::Game(GameAction::HandleMessage(message))),
            }
        }

    }
}
