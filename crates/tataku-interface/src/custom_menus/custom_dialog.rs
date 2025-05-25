use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(Clone, Debug)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct CustomDialog {
    #[serde(rename = "@id")] pub id: String,
    #[serde(rename = "@title")] pub title: String,
    #[serde(rename = "@allow_multiple", default)] pub allow_multiple: bool,
    #[serde(rename = "@draggable", default)] pub draggable: bool, 
    #[serde(rename = "@resizable", default)] pub resizable: bool,

    #[serde(default)] pub style: Option<String>,
    #[serde(default)] pub events: BuildableEventsTag,
    #[serde(default)] pub inputs: BuildableInputsTag,

    pub element: ElementTag,
}
impl CustomDialog {
    pub fn build(
        &self, 
        values: &mut dyn Reflect,
        variables: BuildableInputArguments,
    ) -> Result<BuiltCustomDialog, Vec<BuildableInputError>> {
        self.inputs.init(variables, values)?;

        let mut events: HashMap<TatakuEventType, Vec<BuildableAction>> = HashMap::new();
        for (event, event_type) in self.events.events
            .clone()
            .into_iter()
            .filter_map(|i| 
                i.get_event()
                .copied()
                .map(|e| (i, e))
        ) {
            events.entry(event_type)
                .or_default()
                .extend(event.get_actions());
        }

        Ok(BuiltCustomDialog {
            id: self.id.clone(),
            title: self.title.clone(),
            element: self.element.build(), 
            styles: self.style.clone().unwrap_or_default(),
            events,
            draggable: self.draggable,
            resizable: self.resizable,

            node_id: EMPTY_NODE,
        })
    }

    pub fn options(&self) -> DialogCreateOptions {
        DialogCreateOptions {
            draggable: self.draggable,
            resizable: self.resizable,
            allow_multiple: self.allow_multiple,
            title: Cow::Owned(self.title.clone()),
        }
    }
}

pub struct BuiltCustomDialog {
    pub id: String,
    pub title: String,
    pub element: Box<dyn Widget>,
    pub events: HashMap<TatakuEventType, Vec<BuildableAction>>,

    pub styles: String,

    pub draggable: bool,
    pub resizable: bool,

    node_id: NodeId,
}
impl Widget for BuiltCustomDialog {
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
        shell: &mut InputShell,
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
            shell.handled = true;
            if let Some(action) = action.into_action(
                self.node_id, 
                shell.values, 
                &passed_in
            ) {
                shell.actions.push(action);
            }
            return
        }

        let tag = message.tag.clone();
        match &message.value {
            MessageValue::Value(TatakuValue::Reflect(value)) => {
                let Some(variable) = tag.as_string() else { return };
                shell.handled = true;

                let Some(value) = value.duplicate() else {
                    error!("error duplicating message value");
                    return
                };

                if let Err(e) = shell.values.reflect_insert(
                    variable, 
                    value
                ) {
                    error!("error inserting into values: {e:?}");
                }
            }
            MessageValue::Text(incoming) => {
                let Some(variable) = tag.as_string() else { return };
                shell.handled = true;
                if let Err(e) = shell.values.reflect_insert(
                    variable, 
                    Box::new(incoming.clone())
                ) {
                    error!("error inserting into values: {e:?}");
                }
            }
            
            MessageValue::Multi(messages) => {
                for m in messages {
                    self.handle_message(m, shell);
                }
            }

            _other => warn!("unhandled message: {message:?}"),
        }
    }

    fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<&TatakuValue>, 
        shell: &mut MessageShell,
    ) {
        let Some(events) = self.events.get(&event) else { return };

        for i in events.iter() {
            let Some(message) = i.resolve(
                MessageOwner::Menu, 
                shell.values, 
                event_value
            ) else { continue };

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
    const TEST: &str = include_str!("../../../tataku-resources/dialogs/mods.xml");

    let _dialog = quick_xml::de::from_str::<CustomDialog>(TEST)
        .map_err(|e| format!("{e}"))
        .unwrap();
}
