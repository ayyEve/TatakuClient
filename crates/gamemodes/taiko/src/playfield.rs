use crate::prelude::*;

#[cfg(feature="graphics")]
use engine::graphics;

use tataku::{
    Color,
    Bounds,
    Border,
    Vector2,
};


#[derive(Default)]
pub struct Playfield {
    pub bounds: Bounds,
    pub hit_position: Vector2,
}

impl Playfield {
    pub fn from_settings(
        settings: &Settings,
        mut bounds: Bounds,
    ) -> Playfield {
        let max_note_radius = settings.note_radius * settings.big_note_multiplier;
        let height = max_note_radius * 2.0 + settings.playfield_height_padding;

        let x_offset = settings.playfield_x_pos * bounds.size.x;
        let y_offset = settings.playfield_y_pos * bounds.size.y;

        bounds.pos += Vector2::with_y(y_offset);
        bounds.size.y = height;

        let hit_position = bounds.pos
            + Vector2::new(x_offset + max_note_radius, height / 2.0);

        Playfield {
            bounds,
            hit_position,
        }
    }

    #[cfg(feature = "graphics")]
    pub fn rectangle(&self, kiai: bool) -> impl TatakuRenderable + 'static {
        graphics::Rectangle::new(
            self.bounds.size,
            Color::new(0.1, 0.1, 0.1, 1.0),
        )
        .border_maybe(kiai.then_some(Border::new(Color::YELLOW, 2.0)))
        .with_transform(tataku::Matrix::identity()
            .trans(self.bounds.pos)
        )
    }

    /// Length from the hit position to the end.
    pub fn track_length(&self) -> f32 {
        self.bounds.size.x - self.hit_position.x
    }

    pub fn note_pos(&self, delta_time: f32, speed: f32) -> f32 {
        delta_time * speed
            * self.track_length() / DEFAULT_TRACK_LENGTH
    }
}
impl Deref for Playfield {
    type Target = Bounds;
    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}
