use crate::prelude::*;
use engine::graphics;
use tataku::{
    Color,
    Vector2,
};

/// timing bar color
pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0);
/// how wide is a timing bar
const BAR_WIDTH:f32 = 4.0;

// timing bar struct
//TODO: might be able to reduce this to a (time, speed) and just calc pos on draw
#[derive(Clone)]
pub struct TimingBar {
    pub time: f32,
    pub speed: f32,
}
impl TimingBar {
    pub fn new(time: f32, speed: f32) -> Self {
        Self {
            time,
            speed,
        }
    }

    #[cfg(feature="graphics")]
    pub fn draw(&mut self, shell: &mut DrawShell) {
        let x = shell.playfield.note_pos(self.time - shell.time, self.speed);

        let x = shell.playfield.hit_position.x + x;

        if x + BAR_WIDTH < shell.playfield.pos.x
            || x - BAR_WIDTH > shell.playfield.pos.x + shell.playfield.size.x
        {
            return;
        }

        shell.list.push(graphics::Rectangle::new(
            Vector2::new(BAR_WIDTH, shell.playfield.size.y),
            BAR_COLOR,
        ).with_transform(tataku::Matrix::identity()
            .trans(Vector2::new(x - BAR_WIDTH / 2.0, shell.playfield.pos.y))
        ));
    }
}
