use crate::prelude::*;

#[derive(ScrollableGettersSetters)]
pub struct LobbySlotDisplay {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    tag: String,
    ui_scale: Vector2,
    base_size: Vector2,

    items: ScrollableArea,
}
impl LobbySlotDisplay {
    pub fn new(width: f32, slot: u8, state_receiver: AsyncReceiver<(LobbySlot, bool)>, player_receiver: AsyncReceiver<Option<LobbyPlayerInfo>>) -> Self {
        let size = Vector2::new(width, 50.0);
        
        let mut items = ScrollableArea::new(Vector2::ZERO, size, ListMode::Grid(GridSettings::new(Vector2::new(5.0, 0.0), HorizontalAlign::Left)));
        items.add_item(Box::new(LobbySlotStatus::new(size.y * 0.8, slot, state_receiver)));
        items.add_item(Box::new(LobbySlotUser::new(Vector2::new(size.x - (size.y * 0.8 + 5.0 * 3.0), size.y * 0.8), slot, player_receiver)));

        Self {
            pos: Vector2::ZERO,
            size,
            base_size: size,
            hover: false,
            tag: String::new(),
            ui_scale: Vector2::ONE,

            items
        }
    }
}

impl ScrollableItem for LobbySlotDisplay {
    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.ui_scale = scale;
        self.size = self.base_size * scale;
    }

    fn on_click_tagged(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers) -> Option<String> {
        self.items.on_click_tagged(pos, button, mods)
    }

    fn on_mouse_move(&mut self, p:Vector2) {
        self.check_hover(p);
        self.items.on_mouse_move(p);
    }

    fn update(&mut self) {
        self.items.update();
        if self.pos != self.items.get_pos() {
            self.items.set_pos(self.pos);
        }
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        // background and border
        list.push(Rectangle::new(self.pos + pos_offset, self.size, Color::GRAY.alpha(0.8), Some(Border::new(if self.hover {Color::RED} else {Color::BLACK}, 2.0))).shape(Shape::Round(4.0)));

        self.items.draw(pos_offset, list);
    }
}



pub struct LobbySlotComponent {
    pub status: LobbySlot,
    pub is_host: bool,
    
    pub player: Option<LobbyPlayerInfo>,
    
    /// cached text
    text: String,
}
impl LobbySlotComponent {
    pub fn new(
        status: LobbySlot,
        is_host: bool, 
        player: Option<LobbyPlayerInfo>,
    ) -> LobbySlotComponent {
        LobbySlotComponent {
            status,
            is_host,
            player,
            text: String::new(),
        }.do_text()
    }
    fn do_text(mut self) -> Self {
        self.update_text(); self
    }

    pub fn update_text(&mut self) {
        let Some(player) = &self.player else {
            self.text = "Empty".to_owned();
            return;
        };

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
    }
    
    fn get_icon(&self) -> FontAwesome {
        if self.is_host { return FontAwesome::Crown }

        match self.status {
            LobbySlot::Empty | LobbySlot::Filled { .. } => FontAwesome::UnlockAlt,
            LobbySlot::Locked | LobbySlot::Unknown => FontAwesome::Lock,
        }
    }
    pub fn view(&self) -> IcedElement {
        use iced_elements::*;
        const FONT_SIZE:f32 = 40.0;

        row!(
            // status box thing
            Text::new(self.get_icon().to_string())
                .vertical_alignment(Vertical::Center)
                .font(Font::FontAwesome.to_iced())
                .size(FONT_SIZE)
                .height(Fill),

            // slot text
            Text::new(self.text.clone())
                .vertical_alignment(Vertical::Center)
                .size(FONT_SIZE)
                .width(Fill)
                .height(Fill)
            ;
            width = Fill,
            align_items = Alignment::Center
        )
    } 
}