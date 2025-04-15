use crate::prelude::*;
use crate::prelude::ui::*;

const LOBBY_NAME_PATH: &str = "new_lobby.name";
const LOBBY_PASSWORD_PATH: &str = "new_lobby.password";
const LOBBY_PRIVATE_PATH: &str = "new_lobby.private";

pub struct CreateLobbyDialog {
    node: Box<dyn Widget>,
    node_id: NodeId
}
impl CreateLobbyDialog {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            node: EmptyWidget::new_boxed(),
            node_id: EMPTY_NODE
        }
    }

    fn view(&self, owner: MessageOwner) -> Box<dyn Widget> {
        col!(
            TextWidget::new("Create Lobby: ").boxed(),
            TextWidget::new(" ").boxed(),
            
            TextInput::new("Lobby Name", BuildableTextInner::Variable(LOBBY_NAME_PATH.to_string()).as_buildable()).on_input(move |t: &str| Message::new(owner, "lobby_name", MessageValue::Text(t.to_string()))).boxed(),
            TextInput::new("Lobby Password", BuildableTextInner::Variable(LOBBY_PASSWORD_PATH.to_string()).as_buildable()).on_input(move |t: &str| Message::new(owner, "lobby_password", MessageValue::Text(t.to_string()))).boxed(),
            Checkbox::new("Private", BuildableCondition::Unbuilt(LOBBY_PRIVATE_PATH.to_owned())).on_toggle(move |v| Message::new(owner, "lobby_private", MessageValue::Toggle(v))).boxed(),
            
            row!(
                Button::new(TextWidget::new("Done").boxed()).on_press(Message::new(owner, "done", MessageValue::Click)).boxed(),
                Button::new(TextWidget::new("Close").boxed()).on_press(Message::new(owner, "close", MessageValue::Click)).boxed()
                ;
                width = FILL
            );
        )
    }
}

#[async_trait]
impl Widget for CreateLobbyDialog {
    fn name(&self) -> Cow<'static, str> { "create_lobby_dialog".into() }
    fn node_id(&self) -> NodeId { self.node_id }
    
    fn update_styles(&mut self, tree: &mut Tree, resolver: &mut CssResolver, display_override: Option<ui::Display>) {
        self.node.update_styles(tree, resolver, display_override);
    }
    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId>  {
        // insert/clear our values
        shell.values.reflect_insert(LOBBY_NAME_PATH, String::new()).unwrap();
        shell.values.reflect_insert(LOBBY_PASSWORD_PATH, String::new()).unwrap();
        shell.values.reflect_insert(LOBBY_PRIVATE_PATH, true).unwrap();

        self.node = self.view(shell.owner);
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
        actions: &mut ActionQueue,
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
        actions: &mut ActionQueue,
    ) {
        let Some(tag) = message.tag.as_string() else { return }; 

        match &**tag {
            "lobby_name" => {
                let Some(text) = message.value.as_text_ref() else { return };
                if let Err(e) = values.reflect_insert(LOBBY_NAME_PATH, text.to_owned()) {
                    error!("{e:?}");
                }
            }
            "lobby_password" => {
                let Some(text) = message.value.as_text_ref() else { return };
                if let Err(e) = values.reflect_insert(LOBBY_PASSWORD_PATH, text.to_owned()) {
                    error!("{e:?}");
                }
            }
            "lobby_private" => {
                let Some(val) = message.value.as_toggle_ref() else { return };
                if let Err(e) = values.reflect_insert(LOBBY_PRIVATE_PATH, *val) {
                    error!("{e:?}");
                }
            }

            "done" => {
                let name = get(values, LOBBY_NAME_PATH);
                let password = get(values, LOBBY_PASSWORD_PATH);
                let private = get(values, LOBBY_PRIVATE_PATH);
                let players = 16;

                
                actions.push(MultiplayerAction::CreateLobby { name, password, private, players });
                actions.push(UiAction::new(self.node_id, DialogAction::Close));
            }

            "close" => actions.push(UiAction::new(self.node_id, DialogAction::Close)),
            
            _ => {}
        }

    }
}


fn get<T: Reflect + Clone + Default + 'static>(values: &mut dyn Reflect, path: &str) -> T {
    values.reflect_get::<T>(path)
        .inspect_err(|e| warn!("{e:?}"))
        .map(|a| (*a).clone())
        .unwrap_or_default()
}
