use crate::prelude::*;

const SIZE:Vector2 = Vector2::new(180.0, 20.0);
const TEXT_PADDING:Vector2 = Vector2::new(0.0, 2.0);

/// fps display helper, cleans up some of the code in game
pub struct FpsDisplay {
    name: String,
    pos: Vector2,
    count: u32,
    timer: Instant,
    last: f32,

    frametimes: Vec<f32>,
    frametime_last: f32,
    frametime_timer: Instant
}

impl FpsDisplay {
    /// name is what to display in text, count is which fps counter is this (only affects position)
    pub fn new(name:&str, count:u8) -> Self {

        let window_size = Settings::window_size();
        Self {
            count: 0,
            last: 0.0,
            timer: Instant::now(),
            name: name.to_owned(),
            pos: Vector2::new(window_size.x - SIZE.x, window_size.y - SIZE.y * (count+1) as f64),

            frametime_last: 0.0,
            frametimes: Vec::new(),
            frametime_timer: Instant::now(),
        }
    }

    pub fn increment(&mut self) {
        self.count += 1;
        
        self.frametimes.push(self.frametime_timer.elapsed().as_secs_f32() * 1000.0);
        self.frametime_timer = Instant::now();
    }
    pub fn draw(&mut self, list:&mut Vec<Box<dyn Renderable>>) {
        let font = get_font("main");

        let fps_elapsed = self.timer.elapsed().as_micros() as f64 / 1000.0;
        if fps_elapsed >= 100.0 {
            self.last = (self.count as f64 / fps_elapsed * 1000.0) as f32;
            self.timer = Instant::now();
            self.count = 0;

            // frame times
            self.frametime_last = 0.0;
            for i in std::mem::take(&mut self.frametimes) {
                self.frametime_last = self.frametime_last.max(i)
            }
        }

        list.push(Box::new(Text::new(
            Color::BLACK,
            -99_999_999.99, // should be on top of everything
            self.pos + TEXT_PADDING,
            12,
            format!("{:.2}{} ({:.2}ms)", self.last, self.name, self.frametime_last),
            font.clone()
        )));

        list.push(visibility_bg(self.pos, SIZE, -99_999_999.98));
    }
}