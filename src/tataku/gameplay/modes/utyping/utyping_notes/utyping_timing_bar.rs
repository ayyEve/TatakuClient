use crate::prelude::*;
use super::super::prelude::*;

/// how wide is a timing bar
const BAR_WIDTH:f32 = 4.0;

/// timing bar color
pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0);

// timing bar struct
//TODO: might be able to reduce this to a (time, speed) and just calc pos on draw
#[derive(Clone)]
pub struct UTypingTimingBar {
    pos: Vector2,
    size: Vector2,
    
    time: f32,
    speed: f32,
    playfield: Arc<UTypingPlayfield>,
}
impl UTypingTimingBar {
    pub fn new(time:f32, speed:f32, playfield: Arc<UTypingPlayfield>) -> Self {
        let size = Vector2::new(BAR_WIDTH, playfield.height);

        Self {
            time, 
            speed,
            pos: Vector2::new(0.0, playfield.hit_position.y - size.y/2.0),
            playfield,
            size
        }
    }

    pub fn update_playfield(&mut self, playfield: Arc<UTypingPlayfield>) {
        self.playfield = playfield;
        self.pos.y = self.playfield.hit_position.y - self.size.y/2.0
    }

    pub fn update(&mut self, _time:f32) {}

    pub fn draw(&mut self, time: f32, list: &mut RenderableCollection) {
        self.pos.x = self.playfield.hit_position.x + ((self.time - time) * self.speed) - BAR_WIDTH / 2.0;
        if self.pos.x + BAR_WIDTH < 0.0 || self.pos.x - BAR_WIDTH > 1000000.0 { return }

        list.push(Rectangle::new(
            self.pos,
            self.size,
            BAR_COLOR,
            None
        ));
    }
}

