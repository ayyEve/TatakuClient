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

    frametime_last: f32,
    /// what frametime to actually draw
    frametime_last_draw: f32,
    frametime_timer: Instant,

    window_size: WindowSizeHelper,
    pos_count: u8,
}
impl FpsDisplay {
    /// name is what to display in text, count is which fps counter is this (only affects position)
    pub async fn new(name:&str, pos_count:u8) -> Self {
        let window_size = WindowSizeHelper::new().await;

        Self {
            count: 0,
            last: 0.0,
            timer: Instant::now(),
            name: name.to_owned(),
            pos: Vector2::new(window_size.x - SIZE.x, window_size.y - SIZE.y * (pos_count+1) as f64),

            frametime_last: 0.0,
            frametime_last_draw: 0.0,
            frametime_timer: Instant::now(),
            window_size,
            pos_count
        }
    }

    pub fn update(&mut self) {
        if self.window_size.update() {
            self.pos = self.window_size.0 - Vector2::new(SIZE.x, SIZE.y * (self.pos_count+1) as f64)
        }

        let fps_elapsed = self.timer.elapsed().as_micros() as f64 / 1000.0;
        if fps_elapsed >= 100.0 {
            self.last = (self.count as f64 / fps_elapsed * 1000.0) as f32;
            self.timer = Instant::now();
            self.count = 0;
            
            // frame times
            self.frametime_last_draw = self.frametime_last;
            self.frametime_last = 0.0;
            // info!("{:.2}{} ({:.2}ms)", self.last, self.name, self.frametime_last);
        }
    }

    pub fn increment(&mut self) {
        self.count += 1;
        
        self.frametime_last = self.frametime_last.max(self.frametime_timer.elapsed().as_secs_f32() * 1000.0);
        self.frametime_timer = Instant::now();
    }
    pub fn draw(&self, list:&mut Vec<Box<dyn Renderable>>) {
        let font = get_font();

        list.push(Box::new(Text::new(
            Color::BLACK,
            -99_999_999.99, // should be on top of everything
            self.pos + TEXT_PADDING,
            12,
            format!("{:.2}{} ({:.2}ms)", self.last, self.name, self.frametime_last_draw),
            font.clone()
        )));


        list.push(visibility_bg(self.pos, SIZE, -99_999_999.98));
    }
}


/// fps display helper, cleans up some of the code in game
pub struct AsyncFpsDisplay {
    name: String,
    pos: Vector2,

    count: Arc<AtomicU32>,
    
    timer: Instant,
    last: f32,

    frametime_last: Arc<AtomicU32>,
    frametime_last_draw: f32,
    
    window_size: WindowSizeHelper,
    pos_count: u8,
}
impl AsyncFpsDisplay {
    /// name is what to display in text, count is which fps counter is this (only affects position)
    pub async fn new(name:&str, pos_count:u8, count: Arc<AtomicU32>, frametime_last: Arc<AtomicU32>) -> Self {
        let window_size = WindowSizeHelper::new().await;

        Self {
            count,
            frametime_last,

            last: 0.0,
            timer: Instant::now(),
            name: name.to_owned(),
            pos: Vector2::new(window_size.x - SIZE.x, window_size.y - SIZE.y * (pos_count+1) as f64),

            frametime_last_draw: 0.0,
            window_size,
            pos_count
        }
    }

    pub fn update(&mut self) {
        if self.window_size.update() {
            self.pos = self.window_size.0 - Vector2::new(SIZE.x, SIZE.y * (self.pos_count+1) as f64)
        }
        
        let now = Instant::now();
        let fps_elapsed = now.duration_since(self.timer).as_micros() as f64 / 1000.0;

        
        if fps_elapsed >= 100.0 {
            // reset timer
            self.timer = now;

            // update frametime and last updates/s
            self.frametime_last_draw = self.frametime_last.swap(0, SeqCst) as f32 / 100.0; // restore 2 decimal places
            self.last = (self.count.swap(0, SeqCst) as f64 / fps_elapsed * 1000.0) as f32;
        }
    }

    pub fn draw(&self, list:&mut Vec<Box<dyn Renderable>>) {
        let font = get_font();

        list.push(Box::new(Text::new(
            Color::BLACK,
            -99_999_999.99, // should be on top of everything
            self.pos + TEXT_PADDING,
            12,
            format!("{:.2}{} ({:.2}ms)", self.last, self.name, self.frametime_last_draw),
            font.clone()
        )));

        list.push(visibility_bg(self.pos, SIZE, -99_999_999.98));
    }
}
