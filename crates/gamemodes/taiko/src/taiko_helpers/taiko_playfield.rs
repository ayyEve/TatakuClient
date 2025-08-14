use crate::prelude::*;

pub struct TaikoPlayfield {
    pub bounds: Bounds,
    pub height: f32,

    pub hit_position: Vector2,

    // pub full_window: bool,
}

impl TaikoPlayfield {
    pub fn get_rectangle(&self, kiai: bool) -> Rectangle {
        Rectangle::new_bounds(
            self.get_playfield_bounds(),
            Color::new(0.1, 0.1, 0.1, 1.0),
        )
        .border_maybe(kiai.then_some(Border::new(Color::YELLOW, 2.0)))
    }

    pub fn get_playfield_bounds(&self) -> Bounds {
        let width = self.bounds.size.x;
        let height = self.height;

        Bounds::new(
            Vector2::new(self.pos.x, self.hit_position.y - height / 2.0),
            Vector2::new(width, height)
        )
    }
}
impl Deref for TaikoPlayfield {
    type Target = Bounds;
    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}
