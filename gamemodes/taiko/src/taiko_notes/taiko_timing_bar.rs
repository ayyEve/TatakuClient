use crate::prelude::*;

/// timing bar color
pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0);
/// how wide is a timing bar
const BAR_WIDTH:f32 = 4.0;

// timing bar struct
//TODO: might be able to reduce this to a (time, speed) and just calc pos on draw
#[derive(Clone)]
pub struct TimingBar {
    pub pos: Vector2,
    pub size: Vector2,
    
    pub time: f32,
    pub speed: f32,
    pub playfield: Arc<TaikoPlayfield>,
}
impl TimingBar {
    pub fn new(time:f32, speed:f32, playfield: Arc<TaikoPlayfield>) -> TimingBar {
        let size = Vector2::new(BAR_WIDTH, playfield.height);

        TimingBar {
            time, 
            speed,
            pos: Vector2::new(0.0, playfield.hit_position.y - size.y/2.0),
            playfield,
            size
        }
    }

    pub fn update(&mut self, time:f32) {
        self.pos.x = self.playfield.hit_position.x + self.x_at(time) - BAR_WIDTH / 2.0;
    }

    fn x_at(&self, time: f32) -> f32 {
        ((self.time - time) / SV_OVERRIDE) * self.speed * self.playfield.size.x
    }
    pub fn draw(&mut self, list: &mut RenderableCollection){
        if self.pos.x + BAR_WIDTH < self.playfield.pos.x || self.pos.x - BAR_WIDTH > self.playfield.pos.x + self.playfield.size.x { return }

        list.push(Rectangle::new(
            self.pos,
            self.size,
            BAR_COLOR,
            None
        ));
    }

    pub fn playfield_changed(&mut self, new: Arc<TaikoPlayfield>) {
        self.playfield = new;
        self.pos.y = self.playfield.hit_position.y - self.size.y/2.0;
        self.size.y = self.playfield.height;
    }

    pub fn set_settings(&mut self, _settings: Arc<TaikoSettings>) {
        // self.size = Vector2::new(BAR_WIDTH, self.settings.get_playfield(0.0, false).size.y);
        // self.pos = Vector2::new(0.0, self.settings.hit_position.y - self.size.y/2.0);
    }

}
