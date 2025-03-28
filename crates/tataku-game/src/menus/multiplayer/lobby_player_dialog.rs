use crate::prelude::*;
use crate::prelude::ui::*;
// const BUTTON_SIZE:Vector2 = Vector2::new(300.0, 50.0);

pub struct LobbyPlayerDialog {
    user_id: u32,
    slot_id: u8,
    is_self: bool,
    we_are_host: bool,


    node: Box<dyn Widget>,
    node_id: NodeId,
}
impl LobbyPlayerDialog {
    pub fn new(
        user_id: u32, 
        slot_id: u8, 
        is_self: bool, 
        we_are_host: bool
    ) -> Self {
        Self {
            user_id,
            slot_id,

            is_self,
            we_are_host,

            node: EmptyWidget::new_boxed(),
            node_id: EMPTY_NODE
        }
    }
}

#[async_trait]
impl Widget for LobbyPlayerDialog {
    fn name(&self) -> Cow<'static, str> { "lobby_player_dialog".into() }
    fn node_id(&self) -> NodeId { self.node_id }
    
    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        let owner = shell.owner;
        self.node = col!(
            // make host
            (self.we_are_host && !self.is_self)
                .then(|| Button::new(TextWidget::new("Transfer Host").boxed()).on_press(Message::new(owner, "make_host", MessageValue::Click)).boxed())
                .unwrap_or_else(|| EmptyWidget::new_boxed()),
            // kick
            (self.we_are_host && !self.is_self)
                .then(|| Button::new(TextWidget::new("Kick").boxed()).on_press(Message::new(owner, "kick", MessageValue::Click)).boxed())
                .unwrap_or_else(|| EmptyWidget::new_boxed()),
            // close
            Button::new(TextWidget::new("Close").boxed()).on_press(Message::new(owner, "close", MessageValue::Click)).boxed();
        );


        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            Style::DEFAULT, 
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
    }
    
    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>, 
        actions: &mut ActionQueue
    ) {
        self.node.update(shell, actions);
    }
    
    fn draw(&self, shell: &mut DrawShell<'_>) {
        self.node.draw(shell);
    }

    
    async fn handle_message(
        &mut self, 
        message: &Message, 
        _values: &mut dyn Reflect,
        actions: &mut ActionQueue,
    ) {
        let Some(tag) = message.tag.as_string() else { return }; 

        match &**tag {
            "close" => actions.push(UiAction::new(self.node_id, DialogAction::Close)),
            "make_host" => {
                tokio::spawn(OnlineManager::lobby_change_host(self.user_id));
                actions.push(UiAction::new(self.node_id, DialogAction::Close))
            }
            "kick" => {
                actions.push(LobbyAction::SlotAction(LobbySlotAction::Kick(self.slot_id)));
                actions.push(UiAction::new(self.node_id, DialogAction::Close))
            }

            _ => {}
        }
    }
}
