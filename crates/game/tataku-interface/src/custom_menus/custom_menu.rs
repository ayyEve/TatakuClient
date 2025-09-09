use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug)]
pub struct CustomMenu {
    #[serde(rename = "@id")] pub id: ArcStr,
    #[serde(rename="$value")] pub element: Element,

    #[serde(default)] pub style: Option<ArcStr>,
    #[serde(alias="event", default)] pub events: Wrapped<Vec<BuildableEvent>>,
}
impl CustomMenu {
    pub fn build(&self) -> BuiltCustomMenu {
        let events  = self.events.inner.iter()
            .map(|buildable| {
                let event = BuildableEvent::resolve(&buildable.event);

                let actions = buildable.actions.iter().cloned()
                    .map(|mut action| {
                        action.build();
                        action
                    })
                    .collect();

                (event, actions)
            })
            .collect();

        BuiltCustomMenu {
            id: self.id.clone(),
            element: self.element.build(),
            styles: self.style.clone().unwrap_or_default(),
            events,
            node_id: EMPTY_NODE,
        }
    }
}

pub struct BuiltCustomMenu {
    pub id: ArcStr,
    pub styles: ArcStr,
    pub element: Box<dyn Widget<TatakuAction>>,
    pub events: HashMap<TatakuEvent, Vec<BuildableAction>>,

    node_id: NodeId,
}
impl Widget<TatakuAction> for BuiltCustomMenu {
    fn name(&self) -> CowStr { self.id.to_string().into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn get_style_str(&self) -> ArcStr { self.styles.clone() }

    fn children(&self) -> WidgetChildren<'_, TatakuAction> {
        WidgetChildren::Single(&self.element)
    }
    fn children_mut(&mut self) -> WidgetChildrenMut<'_, TatakuAction> {
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
            if let Some(action) = action.resolve(
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
        event: &TatakuEvent,
        event_value: Option<&TatakuValue>,
        shell: &mut MessageShell<TatakuAction>,
    ) {
        let Some(events) = self.events.get(event)
        else { return };

        for i in events.iter() {
            let Some(action) = i.clone().resolve(
                self.node_id(),
                shell.values,
                event_value,
            ) else { continue };
            shell.actions.push(action);
        }
    }
}


// #[test]
// fn test() {
//     let reader = std::io::Cursor::new(tataku_resources::menus::BEATMAP_SELECT);
//     let _menu = quick_xml::de::from_reader::<_, CustomMenu>(reader)
//         .map_err(|e| format!("{e}"))
//         .unwrap();
// }
