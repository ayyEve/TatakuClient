use crate::prelude::*;
use crate::prelude::ui::*;

const PASSWORD_PATH: &str = "join_lobby.password";

pub struct JoinLobbyDialog {
    lobby_id: u32,

    node: Box<dyn Widget>,
    node_id: NodeId,
}
impl JoinLobbyDialog {
    pub fn new(lobby_id: u32) -> Self {
        let node = col!(
            TextWidget::new("Enter Password:").boxed(),
            TextInput::new("Password:", CustomElementText::Variable(PASSWORD_PATH.to_string())).on_input(move |t: &str| Message::new_dialog("password", MessageValue::Text(t.to_string()))).boxed(),

            row!(
                Button::new(TextWidget::new("Join").boxed()).on_press(Message::new_dialog("done", MessageValue::Click)).boxed(),
                Button::new(TextWidget::new("Cancel").boxed()).on_press(Message::new_dialog("close", MessageValue::Click)).boxed();
                width = FILL
            );
        );

        Self {
            lobby_id,

            node,
            node_id: EMPTY_NODE
        }
    }
}

#[async_trait]
impl Widget for JoinLobbyDialog {
    fn name(&self) -> Cow<'static, str> { "join_lobby_dialog".into() }
    fn node_id(&self) -> NodeId { self.node_id }
    
    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId>  {
        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            Style::default(), 
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
        values: &mut dyn Reflect,
        actions: &mut ActionQueue
    ) {
        let Some(tag) = message.tag.as_string() else { return }; 

        match &**tag {
            "password" => {
                let Some(text) = message.value.as_text_ref() else { return };
                if let Err(e) = values.reflect_insert(PASSWORD_PATH, text.to_owned()) {
                    error!("{e:?}");
                }
            }

            "done" => {
                let lobby_id = self.lobby_id;
                let password = values
                    .reflect_get::<String>(PASSWORD_PATH)
                    .inspect_err(|e| warn!("{e:?}"))
                    .map(|a| (*a).clone())
                    .unwrap_or_default();
                // self.password.clone(); //get_value::<String>("password");
                // tokio::spawn(async move { OnlineManager::join_lobby(id, password).await; });
                actions.push(MultiplayerAction::JoinLobby {
                    lobby_id, 
                    password,
                });
                actions.push(UiAction::new(self.node_id, DialogAction::Close));
            }

            "close" => actions.push(UiAction::new(self.node_id, DialogAction::Close)),

            _ => {}
        }
    }
}

