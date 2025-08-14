use crate::prelude::*;

#[derive(Clone, Debug)]
#[derive(Deserialize)]
pub struct CustomMenu {
    #[serde(rename = "@id")] pub id: ArcStr,
    pub element: ElementTag,
    
    #[serde(default)] pub style: Option<ArcStr>,
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
        for event in self
            .events
            .events
            .clone()
            .into_iter()
            .filter(|i| i.get_event().is_some())
        {
            let mut event2 = event.get_event().unwrap().clone();
            event2.build();

            let Some(e) = event2.resolve(values) 
            else { continue };

            events.entry(e)
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
    pub id: ArcStr,
    pub styles: ArcStr,
    pub element: Box<dyn Widget<TatakuAction>>,
    pub events: HashMap<TatakuEventType, Vec<BuildableAction>>,

    node_id: NodeId,
}
impl Widget<TatakuAction> for BuiltCustomMenu {
    fn name(&self) -> CowStr { self.id.to_string().into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn get_style_str(&self) -> ArcStr { self.styles.clone() }

    fn children(&self) -> WidgetChildren<TatakuAction> {
        WidgetChildren::Single(&self.element)
    }
    fn children_mut(&mut self) -> WidgetChildrenMut<TatakuAction> {
        WidgetChildrenMut::Single(&mut self.element)
    }

    fn layout(&mut self, shell: &mut LayoutShell<TatakuAction>) -> taffy::TaffyResult<NodeId> {
        let child = self.element.layout(shell)?;
        self.node_id = shell.tree.new_with_children(&[child])?;
        Ok(self.node_id)
    }

    fn init_style(&mut self, shell: &mut LayoutShell<TatakuAction>) {
        shell.tree.update_style(
            self.node_id, 
            |style| *style = style.clone()
                .merge_parent(CssStyle::menu_layout())
        );
        self.element.init_style(shell);
    }

    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell<TatakuAction>,
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
                passed_in.as_ref()
            ) {
                shell.actions.push(action);
            }
            
            shell.handled = true;
            return
        }

        let tag = message.tag.clone();
        match message.value.clone() {
            MessageValue::Value(TatakuValue::Reflect(value)) => {
                shell.handled = true;

                if let Err(e) = shell
                    .values
                    .reflect_insert(&*tag, value)
                {
                    error!("error inserting into values: {e:?}");
                }
            }
            MessageValue::Text(incoming) => {
                shell.handled = true;
                if let Err(e) = shell
                    .values
                    .reflect_insert(&*tag, Box::new(incoming)) 
                {
                    error!("error inserting into values: {e:?}");
                }
            }

            other => warn!("unhandled message: {other:?}"),
        }
    }

    fn handle_event(
        &mut self, 
        event: &TatakuEventType, 
        event_value: Option<&TatakuValue>, 
        shell: &mut MessageShell<TatakuAction>,
    ) {
        let Some(events) = self.events.get(event) 
        else { return };

        for i in events.iter() {
            let Some(action) = i.clone().into_action(
                self.node_id(), 
                shell.values, 
                event_value,
            ) else { continue };
            shell.actions.push(action);
        }
    }
}


#[test]
fn test() {
    let reader = std::io::Cursor::new(tataku_resources::menus::BEATMAP_SELECT);
    let _menu = quick_xml::de::from_reader::<_, CustomMenu>(reader)
        .map_err(|e| format!("{e}"))
        .unwrap();
}
