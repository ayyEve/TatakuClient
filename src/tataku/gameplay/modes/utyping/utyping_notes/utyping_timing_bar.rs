use crate::prelude::*;

/// how wide is a timing bar
const BAR_WIDTH:f64 = 4.0;

/// timing bar color
pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0);

// timing bar struct
//TODO: might be able to reduce this to a (time, speed) and just calc pos on draw
#[derive(Clone)]
pub struct TimingBar {
    time: f32,
    speed: f32,
    pos: Vector2,
    settings: Arc<TaikoSettings>,
    size: Vector2
}
impl TimingBar {
    pub fn new(time:f32, speed:f32, settings: Arc<TaikoSettings>) -> TimingBar {
        let size = Vector2::new(BAR_WIDTH, settings.get_playfield(0.0, false).size.y);

        TimingBar {
            time, 
            speed,
            pos: Vector2::new(0.0, settings.hit_position.y - size.y/2.0),
            settings,
            size
        }
    }

    pub fn update(&mut self, time:f32) {
        self.pos.x = self.settings.hit_position.x + ((self.time - time) * self.speed) as f64 - BAR_WIDTH / 2.0;
    }

    pub fn draw(&mut self, args:RenderArgs, list: &mut RenderableCollection){
        if self.pos.x + BAR_WIDTH < 0.0 || self.pos.x - BAR_WIDTH > args.window_size[0] {return}

        const DEPTH:f64 = 1001.5;

        list.push(Rectangle::new(
            BAR_COLOR,
            DEPTH,
            self.pos,
            self.size,
            None
        ));
    }
}

