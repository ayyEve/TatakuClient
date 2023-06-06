use crate::prelude::*;
use super::super::prelude::*;

// timing bar consts
pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0); // timing bar color
const BAR_HEIGHT:f32 = 4.0; // how tall is a timing bar
const BAR_DEPTH:f32 = -90.0;


// timing bar struct
//TODO: might be able to reduce this to a (time, speed) and just calc pos on draw
#[derive(Clone)]
pub struct TimingBar {
    time: f32,
    speed: f32,
    pos: Vector2,
    size: Vector2,

    playfield: Arc<ManiaPlayfield>,

    relative_y: f32,
    position_function: Arc<Vec<PositionPoint>>,
    position_function_index: usize,
}
impl TimingBar {
    pub fn new(time:f32, width:f32, x:f32, playfield: Arc<ManiaPlayfield>) -> TimingBar {
        TimingBar {
            time, 
            size: Vector2::new(width, BAR_HEIGHT),
            speed: 1.0,
            pos: Vector2::with_x(x),
            relative_y: 0.0,

            position_function: Arc::new(Vec::new()),
            position_function_index: 0,

            playfield
        }
    }

    pub fn set_sv(&mut self, sv:f32) {
        self.speed = sv;
    }

    fn y_at(&mut self, time: f32) -> f32 {
        let speed = self.speed * if self.playfield.upside_down {-1.0} else {1.0};

        self.playfield.hit_y() - (self.relative_y - ManiaGame::pos_at(&self.position_function, time, &mut self.position_function_index)) * speed
    }

    
    pub fn set_position_function(&mut self, p: Arc<Vec<PositionPoint>>) {
        self.position_function = p;

        self.relative_y = ManiaGame::pos_at(&self.position_function, self.time, &mut 0);
    }

    pub fn update(&mut self, time:f32) {
        self.pos.y = self.y_at(time);
        
        // (self.playfield.hit_y() + self.playfield.note_size().y-self.size.y) - ((self.time - time) * self.speed) as f64;
        // self.pos = HIT_POSITION + Vector2::new(( - BAR_WIDTH / 2.0, -PLAYFIELD_RADIUS);
    }

    pub fn draw(&mut self, list: &mut RenderableCollection) {
        if self.pos.y < 0.0 || self.pos.y > self.playfield.window_size.y { return }

        list.push(Rectangle::new(
            BAR_COLOR,
            BAR_DEPTH,
            self.pos + Vector2::with_y(self.playfield.note_size().y),
            self.size,
            None
        ));

    }

    pub fn reset(&mut self) {
        self.position_function_index = 0;
    }
    
    pub fn playfield_changed(&mut self, playfield: Arc<ManiaPlayfield>) {
        self.playfield = playfield;
    }
}
