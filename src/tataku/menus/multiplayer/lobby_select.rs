use crate::prelude::*;

pub struct LobbySelect {
    scrollable: ScrollableArea,
    lobbies: HashMap<u32, LobbyInfo>,
    needs_init: bool,

    multiplayer_data: MultiplayerDataHelper,
}
impl LobbySelect {
    pub async fn new() -> Self {
        let multiplayer_data = MultiplayerDataHelper::new();
        let window_size = WindowSizeHelper::new();

        let scrollable = ScrollableArea::new(
            Vector2::ZERO, 
            window_size.0, 
            ListMode::Grid(GridSettings::new(Vector2::new(5.0, 5.0), HorizontalAlign::Center)
        ));
        Self {
            lobbies: multiplayer_data.lobbies.clone(),
            scrollable,
            multiplayer_data,
            needs_init: true
        }
    }
}

#[async_trait]
impl AsyncMenu<Game> for LobbySelect {
    async fn on_change(&mut self, into:bool) {
        if into {
            // tell the server we want to receive updates
            tokio::spawn(OnlineManager::add_lobby_listener());
        } else {
            tokio::spawn(OnlineManager::remove_lobby_listener());
        }
    }

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.scrollable.set_size(window_size.0);
        self.needs_init = true;
    }
    async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        if let Some(tag) = self.scrollable.on_click_tagged(pos, button, mods) {
            match &*tag {
                "create_lobby" => game.add_dialog(Box::new(DraggableDialog::new(DraggablePosition::CenterMiddle, Box::new(CreateLobbyDialog::new()))), false),
                "back" => game.queue_state_change(GameState::InMenu(Box::new(MainMenu::new().await))),
            
                tag => {
                    let id = tag.parse::<u32>().unwrap();
                    if self.lobbies.get(&id).unwrap().has_password {
                        game.add_dialog(Box::new(DraggableDialog::new(DraggablePosition::CenterMiddle, Box::new(JoinLobbyDialog::new(id)))), false);
                    } else {
                        tokio::spawn(OnlineManager::join_lobby(id, String::new()));
                    }
                }
            }
        }
    }
    async fn on_click_release(&mut self, pos:Vector2, button:MouseButton, _g:&mut Game) {
        self.scrollable.on_click_release(pos, button);
    }

    async fn on_scroll(&mut self, delta:f32, _g:&mut Game) {
        self.scrollable.on_scroll(delta);
    }
    async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
        self.scrollable.on_mouse_move(pos);
    }
    async fn on_key_press(&mut self, key:Key, game:&mut Game, mods:KeyModifiers) {
        if key == Key::Escape {
            game.queue_state_change(GameState::InMenu(Box::new(MainMenu::new().await)));
            return;
        }

        self.scrollable.on_key_press(key, mods);
    }
    async fn on_key_release(&mut self, key:Key, _g:&mut Game) {
        self.scrollable.on_key_release(key);
    }
    async fn on_text(&mut self, text:String) {
        self.scrollable.on_text(text);
    }

    async fn update(&mut self, game: &mut Game) {
        if self.multiplayer_data.update() || self.needs_init {
            if self.multiplayer_data.lobbies != self.lobbies || self.needs_init {
                self.needs_init = false;

                self.lobbies = self.multiplayer_data.lobbies.clone();
                let mut lobbies = self.lobbies.clone().into_values().collect::<Vec<_>>();
                lobbies.sort_by(|a, b|a.id.cmp(&b.id));

                let window_size = WindowSizeHelper::new();
                
                let mut lobby_scrollable = ScrollableArea::new(
                    Vector2::ZERO, 
                    window_size.0 - Vector2::new(10.0, 100.0), 
                    ListMode::Grid(GridSettings::new(Vector2::new(5.0, 5.0), HorizontalAlign::Left)
                )); 

                for i in lobbies {
                    lobby_scrollable.add_item(Box::new(LobbyDisplay::new(i.clone())));
                }

                self.scrollable.clear();
                self.scrollable.add_item(Box::new(lobby_scrollable));
                self.scrollable.add_item(Box::new(MenuButton::new(Vector2::ZERO, Vector2::new(100.0, 50.0), "Create Lobby", Font::Main).with_tag("create_lobby")));
                self.scrollable.add_item(Box::new(MenuButton::new(Vector2::ZERO, Vector2::new(100.0, 50.0), "Back", Font::Main).with_tag("back")));
            }
        }

        if (self.multiplayer_data.lobby_creation_pending || self.multiplayer_data.lobby_join_pending) && CurrentLobbyInfo::get().is_some() {
            // joined lobby
            game.queue_state_change(GameState::InMenu(Box::new(LobbyMenu::new().await)));
        }

        // lost connection
        if !OnlineManager::get().await.logged_in {
            game.queue_state_change(GameState::InMenu(Box::new(MainMenu::new().await)));
        }
    }
    async fn draw(&mut self, list: &mut RenderableCollection) {
        list.push(visibility_bg(Vector2::ZERO, self.scrollable.size()));
        self.scrollable.draw(Vector2::ZERO, list);
    }
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
