use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(Clone, Debug)]
#[derive(Deserialize)]
pub struct CustomMenu {
    #[serde(rename = "@id")] pub id: String,
    pub element: ElementTag,
    
    #[serde(default)] pub style: Option<String>,
    #[serde(default)] pub events: BuildableEventsTag, 
}

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct BuildableEventsTag {
    #[serde(alias = "$value", alias = "$text")] pub events: Vec<BuildableEvent>,
}


pub struct BuiltCustomMenu {
    pub id: String,
    pub element: Box<dyn Widget>,
    pub actions: ActionQueue,
    pub events: HashMap<TatakuEventType, Vec<BuildableAction>>,

    pub styles: String,

    node_id: NodeId,
}
impl BuiltCustomMenu {
    pub fn build(menu: &CustomMenu) -> Self {
        let mut events: HashMap<TatakuEventType, Vec<BuildableAction>> = HashMap::new();
        for event in menu.events.events.clone().into_iter().filter(|i| i.get_event().is_some()) {
            events.entry(*event.get_event().unwrap())
                .or_default()
                .extend(event.get_actions());
        }
        let mut shell = ElementBuildShell {
            owner: MessageOwner::Menu,
            _empty: std::marker::PhantomData
        };

        Self {
            id: menu.id.clone(),
            element: menu.element.build(&mut shell), 
            styles: menu.style.clone().unwrap_or_default(),
            actions: ActionQueue::new(),
            events,
            node_id: EMPTY_NODE,
        }
    }
}
#[async_trait]
impl Widget for BuiltCustomMenu {
    fn name(&self) -> Cow<'static, str> { format!("custom-{}", self.id).into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn get_style_str(&self) -> String { self.styles.clone() }

    fn update_styles(&mut self, tree: &mut Tree, resolver: &mut CssResolver, _display_override: Option<ui::Display>) {
        self.element.update_styles(tree, resolver, None);
    }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        let child = self.element.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            menu_layout(), 
            &[child]
        )?;

        Ok(self.node_id)
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<'_>
    ) {
        self.element.input(event, shell);
    }

    fn draw(&self, shell: &mut DrawShell<'_>) {
        self.element.draw(shell);
    }

    fn update(&mut self, shell: &mut UpdateShell<'_>, actions: &mut ActionQueue) {
        actions.extend(self.actions.take());
        self.element.update(shell, actions);
    }

    async fn handle_message(
        &mut self, 
        message: &Message, 
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue,
    ) {
        actions.extend(self.actions.take());

        self.element.handle_message(message, values, &mut self.actions).await;
        if !self.actions.is_empty() { return actions.extend(self.actions.take())}

        let cast = message.value
            .try_downcast_ref::<(BuildableAction, Option<TatakuValue>)>()
            .cloned();
        if let Some((action, passed_in)) = cast {
            if let Some(action) = action.into_action(values, passed_in) {
                self.actions.push(action)
            }
            
            return
        }

        let tag = message.tag.clone();
        match message.value.clone() {
            MessageValue::Value(TatakuValue::Reflect(value)) => {
                let Some(variable) = tag.as_string() else { return };
                // values.update_or_insert(&variable, TatakuVariableWriteSource::Menu, incoming, || TatakuVariable::new_any(TatakuValue::None));

                if let Err(e) = values.reflect_insert(variable, value) {
                    error!("error inserting into values: {e:?}");
                }
            }
            MessageValue::Text(incoming) => {
                let Some(variable) = tag.as_string() else { return };
                if let Err(e) = values.reflect_insert(variable, Box::new(incoming)) {
                    error!("error inserting into values: {e:?}");
                }
            }
            
            MessageValue::Multi(messages) => {
                for i in messages {
                    self.handle_message(&i, values, actions).await;
                }
            }

            other => warn!("unhandled message: {other:?}"),
        }
    }

    async fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        values: &mut dyn Reflect,
    ) {
        let Some(events) = self.events.get(&event) else { return };

        for i in events.iter() {
            let Some(message) = i.resolve(MessageOwner::Menu, values, event_value.clone()) else { continue };

            let cast = message.value
                .try_downcast_ref::<(BuildableAction, Option<TatakuValue>)>()
                .cloned();
            if let Some((action, passed_in)) = cast {
                let Some(a) = action.into_action(values, passed_in) else { continue };
                self.actions.push(a);
            } else {
                self.actions.push(GameAction::HandleMessage(message))
            }
        }
    }

    async fn reload_skin(&mut self, shell: &mut UpdateShell) {
        self.element.reload_skin(shell).await;
    }
}





#[test]
fn test() {
    const TEST_MENU: &str = include_str!("../../../tataku-resources/menus/beatmap_select_menu.xml");

    let _menu = quick_xml::de::from_str::<CustomMenu>(TEST_MENU)
        .map_err(|e| format!("{e}"))
        .unwrap();
}
