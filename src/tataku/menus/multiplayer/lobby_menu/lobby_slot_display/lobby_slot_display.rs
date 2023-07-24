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
