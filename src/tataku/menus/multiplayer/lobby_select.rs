use crate::prelude::*;

pub struct LobbySelect {
    actions: Vec<MenuAction>,
    // scrollable: ScrollableArea,
    lobbies: HashMap<u32, LobbyInfo>,
    needs_init: bool,

    multiplayer_data: MultiplayerDataHelper,
}
impl LobbySelect {
    pub async fn new() -> Self {
        let multiplayer_data = MultiplayerDataHelper::new();
        Self {
            actions: Vec::new(),
            lobbies: multiplayer_data.lobbies.clone(),
            // scrollable,
            multiplayer_data,
            needs_init: true
        }
    }
}

#[async_trait]
impl AsyncMenu for LobbySelect {
    async fn on_change(&mut self, into:bool) {
        if into {
            // tell the server we want to receive updates
            tokio::spawn(OnlineManager::add_lobby_listener());
        } else {
            tokio::spawn(OnlineManager::remove_lobby_listener());
        }
    }

    async fn update(&mut self) -> Vec<MenuAction> {
        if self.multiplayer_data.update() || self.needs_init {
            if self.multiplayer_data.lobbies != self.lobbies || self.needs_init {
                self.needs_init = false;

                self.lobbies = self.multiplayer_data.lobbies.clone();
                let mut lobbies = self.lobbies.clone().into_values().collect::<Vec<_>>();
                lobbies.sort_by(|a, b|a.id.cmp(&b.id));
            }
        }

        if (self.multiplayer_data.lobby_creation_pending || self.multiplayer_data.lobby_join_pending) && CurrentLobbyInfo::get().is_some() {
            // joined lobby
            self.actions.push(MenuAction::SetMenu(Box::new(LobbyMenu::new().await)));
        }

        // lost connection
        if !OnlineManager::get().await.logged_in {
            self.actions.push(MenuAction::SetMenu(Box::new(MainMenu::new().await)));
        }

        self.actions.take()
    }
    
    
    fn view(&self) -> IcedElement {
        use iced_elements::*;
        let cols = 5;
        let rows = 5;

        col!(

            // lobby grid
            col!(
                self.lobbies.clone().into_values().collect::<Vec<_>>().chunks(cols as usize).map(|i|row!(
                    i.iter().map(|lobby|col!(
                        Text::new(lobby.name.clone());
                        width = FillPortion(cols),
                        height = FillPortion(rows)
                    ))
                    .chain((0..(i.len() - cols as usize)).into_iter().map(|_|Space::new(FillPortion(cols), FillPortion(rows)).into_element()))
                    .collect(),
                    width = Fill,
                    spacing = 5.0
                )).collect(),

                width = Fill
            ),

            // buttons
            row!(
                Button::new(Text::new("Create Lobby")).on_press(Message::new_menu(self, "create_lobby", MessageType::Click)),
                Button::new(Text::new("Back")).on_press(Message::new_menu(self, "back", MessageType::Click));
            )
            ;
        )
    }
    
    async fn handle_message(&mut self, message: Message) {
        let Some(tag) = message.tag.as_string() else { return };

        match &*tag {
            "create_lobby" => self.actions.push(MenuAction::AddDialog(Box::new(CreateLobbyDialog::new()), false)),
            "back" => self.actions.push(MenuAction::SetMenu(Box::new(MainMenu::new().await))),

            _ => if let Some(id) = message.message_type.as_number() {
                let id = id  as u32;
                if self.lobbies.get(&id).unwrap().has_password {
                    MenuAction::AddDialog(Box::new(DraggableDialog::new(DraggablePosition::CenterMiddle, Box::new(JoinLobbyDialog::new(id)))), false);
                } else {
                    tokio::spawn(OnlineManager::join_lobby(id, String::new()));
                }
            }
        }
    }
    
    // async fn on_key_press(&mut self, key:Key, game:&mut Game, mods:KeyModifiers) {
    //     if key == Key::Escape {
    //         game.queue_state_change(GameState::InMenu(Box::new(MainMenu::new().await)));
    //         return;
    //     }

    //     self.scrollable.on_key_press(key, mods);
    // }
}



const LOBBY_DISPLAY_SIZE:Vector2 = Vector2::new(200.0, 50.0);
#[derive(ScrollableGettersSetters)]
pub struct LobbyDisplay {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    tag: String,
    ui_scale: Vector2,

    info: LobbyInfo,
}
impl LobbyDisplay {
    pub fn new(info: LobbyInfo) -> Self {
        Self {
            pos: Vector2::ZERO,
            size: LOBBY_DISPLAY_SIZE,
            hover: false,
            tag: info.id.to_string(),
            ui_scale: Vector2::ONE,
            info
        }
    }
}
impl ScrollableItem for LobbyDisplay {
    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.ui_scale = scale;
        self.size = LOBBY_DISPLAY_SIZE * scale;
    }
    fn update(&mut self) {

    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        // background and border
        let bg = Rectangle::new(self.pos + pos_offset, self.size, Color::GRAY.alpha(0.8), Some(Border::new(if self.hover {Color::RED} else {Color::BLACK}, 2.0))).shape(Shape::Round(4.0));
        list.push(bg);

        // lobby title
        list.push(Text::new(self.pos + pos_offset + Vector2::ONE * 5.0, 32.0, self.info.name.clone(), Color::BLACK, Font::Main));
    }
}
