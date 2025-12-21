use crate::prelude::*;
use tataku::TatakuValue;
use ui::{
    tree::NodeId,
    widget::*,
    message::*,
    style::CssStyle,
};

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
    pub fn build(&self) -> BuiltCustomDialog {
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

        BuiltCustomDialog {
            id: self.id.clone(),
            title: self.title.clone(),
            element: self.element.build(),
            styles: self.style.clone().unwrap_or_default(),
            events,
            draggable: self.draggable,
            resizable: self.resizable,

            node_id: ui::EMPTY_NODE,
        }
    }

    pub fn options(&self) -> actions::menu::DialogCreateOptions {
        actions::menu::DialogCreateOptions {
            draggable: self.draggable,
            resizable: self.resizable,
            allow_multiple: self.allow_multiple,
            title: Cow::Owned(self.title.to_string()),
            location: actions::menu::DialogLocation::Auto,
            background: true,
        }
    }
}

pub struct BuiltCustomDialog {
    pub id: ArcStr,
    pub title: ArcStr,
    pub element: widgets::WidgetBase,
    pub events: HashMap<input::TatakuEvent, Vec<BuildableAction>>,

    pub styles: ArcStr,

    pub draggable: bool,
    pub resizable: bool,

    node_id: NodeId
}
impl Widget<actions::Action> for BuiltCustomDialog {
    fn name(&self) -> CowStr { self.id.to_string().into() }
    fn node_id(&self) -> &NodeId { &self.node_id }
    fn get_style_str(&self) -> ArcStr { self.styles.clone() }

    fn children(&self) -> WidgetChildren<'_, actions::Action> {
        WidgetChildren::Single(&self.element)
    }
    fn children_mut(&mut self) -> WidgetChildrenMut<'_, actions::Action> {
        WidgetChildrenMut::Single(&mut self.element)
    }

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId> {
        let child = self.element.layout(shell)?;
        self.node_id = shell.tree.new_with_children(&[child])?;
        Ok(self.node_id)
    }
    fn init_style(&mut self, shell: &mut LayoutShell<actions::Action>) {
        shell.tree.update_style(
            &self.node_id,
            |style| *style = CssStyle::menu_layout()
        );
        self.element.init_style(shell);
    }

    fn handle_message(
        &mut self,
        message: &Message,
        shell: &mut MessageShell<actions::Action>,
    ) {
        self.element.handle_message(message, shell);
        if shell.handled { return }

        let cast = message.value
            .downcast_ref::<(BuildableAction, Option<TatakuValue>)>()
            .cloned();

        if let Some((mut action, passed_in)) = cast {
            action.build();

            shell.handled = true;
            if let Some(action) = action.resolve(
                &self.node_id,
                shell.values,
                passed_in.as_ref()
            ) {
                shell.actions.push(action);
            }
            return
        }

        warn!("unhandled message: {message:?}");
    }

    fn handle_event(
        &mut self,
        event: &input::TatakuEvent,
        event_value: Option<&TatakuValue>,
        shell: &mut MessageShell<actions::Action>,
    ) {
        let Some(events) = self.events.get(event)
        else { return };

        for mut i in events.iter().cloned() {
            i.build();
            let Some(action) = i.resolve(
                &self.node_id,
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
