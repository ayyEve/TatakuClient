use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(Clone, Debug)]
#[derive(Deserialize)]
pub struct CustomMenu {
    #[serde(rename = "@id")] pub id: String,
    pub element: ElementTag,
    
    #[serde(default)] pub style: Option<String>,
    #[serde(default)] pub events: BuildableEventsTag, 
    #[serde(default)] pub inputs: BuildableInputsTag,
}
impl CustomMenu {
    pub fn build(
        &self, 
        values: &mut dyn Reflect,
        variables: BuildableInputArguments,
    ) -> Result<BuiltCustomMenu, Vec<BuildableInputError>> {
        self.inputs.init(variables, values)?;

        let mut events: HashMap<TatakuEventType, Vec<BuildableAction>> = HashMap::new();
        for event in self.events.events.clone().into_iter().filter(|i| i.get_event().is_some()) {
            events.entry(*event.get_event().unwrap())
                .or_default()
                .extend(event.get_actions());
        }
        
        Ok(BuiltCustomMenu {
            id: self.id.clone(),
            element: self.element.build(), 
            styles: self.style.clone().unwrap_or_default(),
            events,
            node_id: EMPTY_NODE,
        })
    }
}


#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct BuildableEventsTag {
    #[serde(rename = "$value")] pub events: Vec<BuildableEvent>,
}


pub struct BuiltCustomMenu {
    pub id: String,
    pub element: Box<dyn Widget>,
    pub events: HashMap<TatakuEventType, Vec<BuildableAction>>,

    pub styles: String,

    node_id: NodeId,
}
impl Widget for BuiltCustomMenu {
    fn name(&self) -> Cow<'static, str> { format!("custom-{}", self.id).into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn get_style_str(&self) -> String { self.styles.clone() }

    fn update_styles(
        &mut self, 
        shell: &mut StyleShell,
        _display_override: Option<ui::Display>
    ) {
        self.element.update_styles(shell, None);
    }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
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
        shell: &mut InputShell
    ) {
        self.element.input(event, shell);
    }

    fn draw(&self, shell: &mut DrawShell) {
        self.element.draw(shell);
    }
    fn draw_overlay(&self, shell: &mut DrawShell) {
        self.element.draw_overlay(shell);
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        self.element.update(shell);
    }

    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell,
    ) {
        self.element.handle_message(message, shell);
        if shell.handled { return }

        let cast = message.value
            .try_downcast_ref::<(BuildableAction, Option<TatakuValue>)>()
            .cloned();

        if let Some((action, passed_in)) = cast {
            if let Some(action) = action.into_action(
                self.node_id, 
                shell.values, 
                &passed_in
            ) {
                shell.actions.push(action);
            }
            
            shell.handled = true;
            return
        }

        let tag = message.tag.clone();
        match message.value.clone() {
            MessageValue::Value(TatakuValue::Reflect(value)) => {
                let Some(variable) = tag.as_string() else { return };
                shell.handled = true;

                if let Err(e) = shell.values.reflect_insert(variable, value) {
                    error!("error inserting into values: {e:?}");
                }
            }
            MessageValue::Text(incoming) => {
                let Some(variable) = tag.as_string() else { return };
                shell.handled = true;
                if let Err(e) = shell.values.reflect_insert(variable, Box::new(incoming)) {
                    error!("error inserting into values: {e:?}");
                }
            }
            
            MessageValue::Multi(messages) => {
                for i in messages {
                    self.handle_message(&i, shell);
                }
            }

            other => warn!("unhandled message: {other:?}"),
        }
    }

    fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        shell: &mut MessageShell,
    ) {
        let Some(events) = self.events.get(&event) else { return };

        for i in events.iter() {
            let Some(message) = i.resolve(MessageOwner::Menu, shell.values, event_value.clone()) else { continue };

            let cast = message.value
                .try_downcast_ref::<(BuildableAction, Option<TatakuValue>)>()
                .cloned();
            
            if let Some((action, passed_in)) = cast {
                let Some(a) = action.into_action(
                    self.node_id, 
                    shell.values, 
                    &passed_in
                ) else { continue };
                shell.actions.push(a);
            } else {
                shell.actions.push(GameAction::HandleMessage(message));
            }
        }
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell) {
        self.element.reload_skin(shell);
    }
}





#[test]
fn test() {
    let reader = std::io::Cursor::new(tataku_resources::menus::BEATMAP_SELECT);
    let _menu = quick_xml::de::from_reader::<_, CustomMenu>(reader)
        .map_err(|e| format!("{e}"))
        .unwrap();
}
