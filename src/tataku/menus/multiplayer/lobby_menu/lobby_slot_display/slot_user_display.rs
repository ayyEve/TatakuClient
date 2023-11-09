use crate::prelude::*;

#[derive(ScrollableGettersSetters)]
pub struct LobbySlotUser {
    pos: Vector2,
    size: Vector2,
    style: Style,
    node: Node,

    hover: bool,
    tag: String,
    ui_scale: Vector2,
    
    player: Option<LobbyPlayerInfo>,
    player_receiver: AsyncReceiver<Option<LobbyPlayerInfo>>,

    /// cached text
    text: String,
}
impl LobbySlotUser {
    pub fn new(slot_id: u8, player_receiver: AsyncReceiver<Option<LobbyPlayerInfo>>, layout_manager: &LayoutManager) -> Self {
        // size: Vector2::new(size.x - (size.y * 0.8 + 5.0 * 3.0), size.y * 0.8)
        let style = Style {
            // size: Size {

            // },
            ..Default::default()
        };
        
        let node = layout_manager.create_node(&style);
        let (pos, size) = LayoutManager::get_pos_size(&style);


        Self {
            pos,
            size,
            style,
            node,

            hover: false,
            tag: "slot_".to_owned() + &slot_id.to_string(),
            ui_scale: Vector2::ONE,

            // slot_id,
            player_receiver,
            player: None,
            text: "Empty".to_owned(),
        }
    }
}

impl ScrollableItem for LobbySlotUser {
    fn get_style(&self) -> Style { self.style.clone() }
    fn apply_layout(&mut self, layout: &LayoutManager, parent_pos: Vector2) {
        let layout = layout.get_layout(self.node);
        self.pos = layout.location.into();
        self.pos += parent_pos;
        self.size = layout.size.into();
    }

    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.ui_scale = scale;
    }

    fn update(&mut self) {
        if let Ok(player) = self.player_receiver.try_recv() {
            self.player = player;

            if let Some(player) = &self.player {
                let mut mods = ModManager::new();
                mods.mods = player.user.mods.clone();
                mods.speed = GameSpeed::from_u16(player.user.speed);

                let username = &player.username;
                let mods = mods.mods_list_string(&CurrentPlaymodeHelper::new().0);
                let status = match player.user.state {
                    LobbyUserState::NoMap => "No Map",
                    LobbyUserState::InGame => "Playing",
                    LobbyUserState::Ready => "Ready",
                    LobbyUserState::NotReady => "Not Ready",
                    LobbyUserState::Unknown => "???",
                };
                
                self.text = format!("{username} ({status}) // {mods}");
            } else {
                self.text = "Empty".to_owned()
            }
        }
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        let pos = self.pos + pos_offset;
        let bounds = Bounds::new(pos, self.size);

        let mut text = Text::new(pos, self.size.y * 0.8, &self.text, Color::BLACK, Font::Main).centered(&bounds);
        text.pos.x = pos.x;

        list.push(text);
    }
}

#[derive(Clone, Debug)]
pub struct LobbyPlayerInfo {
    pub user: LobbyUser,
    pub username: String,
}
impl LobbyPlayerInfo {
    pub fn new(user: LobbyUser, username: String) -> Self {
        Self {
            user, 
            username,
        }
    }
}
