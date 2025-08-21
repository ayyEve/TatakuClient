use crate::prelude::*;

#[derive(Default)]
pub struct UTypingPlayfield {
    pub bounds: Bounds,
    pub height: f32,

    pub hit_position: Vector2,
}

impl UTypingPlayfield {
    #[cfg(feature="graphics")] 
    pub fn get_rectangle(&self, kiai: bool) -> Rectangle {
        let width = self.bounds.size.x;
        let height = self.height;

        Rectangle::new(
            Vector2::new(self.pos.x, self.hit_position.y - height / 2.0),
            Vector2::new(width, height),
            Color::new(0.1, 0.1, 0.1, 1.0),
            
        ).border_maybe(
            kiai.then_some(Border::new(Color::YELLOW, 2.0))
        )
    }
}
impl Deref for UTypingPlayfield {
    type Target = Bounds;
    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}