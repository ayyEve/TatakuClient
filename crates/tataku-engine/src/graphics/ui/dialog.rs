use crate::prelude::*;
use crate::prelude::ui::*;

// TODO: implement draggables and resizables
pub struct DialogWidget {
    title: Cow<'static, str>,
    num: usize,
    should_close: bool,
    style: Style,

    draggable: bool,
    resizable: bool,

    node_id: NodeId,
    node: Box<dyn Widget>,
}
impl DialogWidget {
    pub fn new(
        title: impl Into<Cow<'static, str>>,
        draggable: bool,
        resizable: bool,
        node: Box<dyn Widget>
    ) -> Self {
        let node = if draggable { 
            Self::draggable_view(node) 
        } else { 
            node 
        };

        Self {
            title: title.into(),
            num: 0,
            should_close: false,
            style: Style::default(),
            draggable,
            resizable,
            node_id: NodeId::default(),
            node,
        }
    }


    fn draggable_view(node: Box<dyn Widget>) -> Box<dyn Widget> {
        // TODO:
        node
    }
}

#[async_trait]
impl Widget for DialogWidget {
    fn name(&self) -> Cow<'static, str> { format!("{}{}", self.title, if self.draggable { " (Draggable)" } else { "" } ).into() }
    fn node_id(&self) -> NodeId { self.node_id }
    
    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId>  {
        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            self.style.clone(), 
            &[ child ]
        )?;

        Ok(self.node_id)
    }
    

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<'_>,
    ) {
        self.node.input(event, shell);

        let to_update = shell.messages
            .iter_mut()
            .filter(|m| m.owner == MessageOwner::DialogUnset);

        for m in to_update {
            println!("updating {m:?}");
            m.owner = MessageOwner::Dialog(self.num);
        }
    }

    fn draw(
        &self, 
        shell: &mut DrawShell<'_>,
    ) {
        let Some(bounds) = shell.tree.absolute_bounds(self) else { return };

        shell.list.push(Rectangle::new_bounds(
            bounds, 
            Color::BLACK.alpha(0.9), 
            None
        ));

        self.node.draw(shell);
    }

    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>, 
        actions: &mut ActionQueue,
    ) {
        self.node.update(shell, actions);
    }
    
    async fn handle_message(
        &mut self, 
        message: &Message, 
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue,
    ) {
        match message.owner {
            MessageOwner::Menu => return,
            MessageOwner::DialogUnset => panic!("got unset dialog message owner"),
            MessageOwner::Dialog(num) => if num != self.num { return }
            MessageOwner::Any => {}
        }

        if let Some(str) = message.tag.as_string() {
            match &**str {
                "set_num" => if let MessageValue::Number(n) = message.value {
                    self.num = n;
                    return
                }
                "force_close" => {
                    self.should_close = true;
                    // dont return in case the child has special logic to do on close
                }
                _ => {}
            }
        }

        self.node.handle_message(
            message, 
            values, 
            actions
        ).await;
    }

    async fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        values: &mut dyn Reflect,
    ) {
        self.node.handle_event(event, event_value, values).await;
    }

    async fn reload_skin(
        &mut self, 
        skin_manager: &mut dyn SkinProvider,
    ) {
        self.node.reload_skin(skin_manager).await;
    }
}
