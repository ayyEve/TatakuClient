use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct CustomDialog {
    #[serde(rename = "@id")] pub id: ArcStr,
    #[serde(rename = "@title")] title: ArcStr,
    #[serde(rename = "@allow_multiple", default)] allow_multiple: bool,
    #[serde(rename = "@draggable", default)] draggable: bool,
    #[serde(rename = "@resizable", default)] resizable: bool,

    #[serde(default)] style: Option<ArcStr>,
    #[serde(alias="event", default)] events: Wrapped<Vec<BuildableEvent>>,

    #[serde(rename = "$value")]
    element: Element,
}
impl CustomDialog {
    pub fn build(
        &self,
        values: &mut dyn Reflect,
    ) -> BuiltCustomDialog {
        let events  = self.events.inner.iter()
            .filter_map(|buildable| {
                let event = BuildableEvent::resolve(
                    &buildable.event,
                    values
                );

                let actions = buildable.actions.iter().cloned()
                    .map(|mut action| {
                        action.build(values);
                        action
                    })
                    .collect();

                event.map(|event| (event, actions))
            })
            .collect();

        BuiltCustomDialog {
            id: self.id.clone(),
            title: self.title.clone(),
            element: self.element.build(),
            styles: self.style.clone().unwrap_or_default(),
            events,
            draggable: self.draggable,
            resizable: self.resizable,

            node_id: EMPTY_NODE,
        }
    }

    pub fn options(&self) -> DialogCreateOptions {
        DialogCreateOptions {
            draggable: self.draggable,
            resizable: self.resizable,
            allow_multiple: self.allow_multiple,
            title: Cow::Owned(self.title.to_string()),
            location: DialogLocation::Auto,
            background: true,
        }
    }
}

pub struct BuiltCustomDialog {
    pub id: ArcStr,
    pub title: ArcStr,
    pub element: Box<dyn Widget<TatakuAction>>,
    pub events: HashMap<TatakuEvent, Vec<BuildableAction>>,

    pub styles: ArcStr,

    pub draggable: bool,
    pub resizable: bool,

    node_id: NodeId,
}
impl Widget<TatakuAction> for BuiltCustomDialog {
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
            |style| *style = CssStyle::menu_layout()
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

        if let Some((mut action, passed_in)) = cast {
            action.build(shell.values);

            shell.handled = true;
            if let Some(action) = action.into_action(
                self.node_id,
                shell.values,
                passed_in.as_ref()
            ) {
                shell.actions.push(action);
            }
            return
        }

        let tag = message.tag.clone();
        match &message.value {
            MessageValue::Value(TatakuValue::Reflect(value)) => {
                shell.handled = true;

                let Some(value) = value.duplicate()
                else {
                    error!("Error duplicating message value");
                    return
                };

                if let Err(e) = shell.values.reflect_insert(
                    &*tag,
                    value
                ) {
                    error!("Error inserting into values: {e:?}");
                }
            }
            MessageValue::Text(incoming) => {
                shell.handled = true;
                if let Err(e) = shell.values.reflect_insert(
                    &*tag,
                    Box::new(incoming.clone())
                ) {
                    error!("Error inserting into values: {e:?}");
                }
            }

            _other => warn!("Unhandled message: {message:?}"),
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

        for mut i in events.iter().cloned() {
            i.build(shell.values);
            let Some(action) = i.into_action(
                self.node_id,
                shell.values,
                event_value
            )
            else { continue };

            shell.actions.push(action);
        }
    }
}







// #[test]
// fn test() {
//     for (_, i) in tataku_resources::dialogs::ALL {
//         let _dialog = quick_xml::de::from_reader::<_, CustomDialog>(std::io::Cursor::new(i.to_vec()))
//             .map_err(|e| format!("{e}"))
//             .unwrap();
//     }
// }
