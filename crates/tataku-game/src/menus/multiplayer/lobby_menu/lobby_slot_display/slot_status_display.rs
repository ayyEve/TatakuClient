use crate::prelude::*;

#[derive(ScrollableGettersSetters)]
pub struct LobbySlotStatus {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    tag: String,
    ui_scale: Vector2,

    // slot_id: u8,
    status_receiver: AsyncReceiver<(LobbySlot, bool)>,
    status: LobbySlot,
    is_host: bool,
}
impl LobbySlotStatus {
    pub fn new(size: f32, slot_id: u8, status_receiver: AsyncReceiver<(LobbySlot, bool)>) -> Self {
        let size = Vector2::ONE * size;

        Self {
            pos: Vector2::ZERO,
            size,
            hover: false,
            tag: "slot_status_".to_owned() + &slot_id.to_string(),
            ui_scale: Vector2::ONE,

            // slot_id,
            status: LobbySlot::Empty,
            is_host: false,
            status_receiver
        }
    }
}

impl ScrollableItem for LobbySlotStatus {
    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.ui_scale = scale;
    }

    fn update(&mut self) {
        if let Ok((status, is_host)) = self.status_receiver.try_recv() {
            self.status = status;
            self.is_host = is_host;
        }
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        let pos = self.pos + pos_offset;
        let rect = Rectangle::new(pos, self.size, Color::TRANSPARENT_WHITE, Some(Border::new(if self.hover {Color::RED} else {Color::BLACK}, 2.0))).shape(Shape::Round(5.0));
        // list.push(rect);

        let color = Color::WHITE.alpha(if self.hover {1.0} else {0.8});

        let mut icon = match self.status {
            LobbySlot::Empty | LobbySlot::Filled { .. } => FontAwesome::UnlockAlt,
            LobbySlot::Locked | LobbySlot::Unknown => FontAwesome::Lock,
        };
        if self.is_host {
            icon = FontAwesome::Crown;
        }
        list.push(Text::new(pos, self.size.y * 0.8, icon, color, Font::FontAwesome).centered(&rect));
    }
}