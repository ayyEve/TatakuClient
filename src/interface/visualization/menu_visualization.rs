use crate::prelude::*;
use super::Visualization;

const CUTOFF:f32 = 0.1;
pub const VISUALIZATION_SIZE_FACTOR:f32 = 1.2;


// must be above this value to create a ripple
const RIPPLE_MIN:f32 = 100.0;
const RIPPLE_RESET:f32 = 120.0;

lazy_static::lazy_static! {
    static ref BAR_COLOR:Color = Color::from_hex("#27bfc2");
}


pub struct MenuVisualization {
    data: Vec<FFTData>,
    timer: Instant, // external use only

    bar_height: f32,
    rotation: f32,

    cookie: Image,
    unload_cookie: bool,
    initial_inner_radius: f32,
    current_inner_radius: f32,

    ripples: Vec<TransformGroup>,
    // current_timing_point: TimingPoint,

    window_size: WindowSizeHelper,

    other_timer: Instant, // internal use only
    last_bass: f32, // last bass value. if current is less than this and we havent created a new ripple since, we should do that now
    created_ripple: bool, // have we drawn a ripple since the last fall?
    last_ripple: f32, // bass value of the last ripple

    last_created: f32, // when was the last ripple?

    // index to use for bass
    pub index: usize,
}
impl MenuVisualization {
    pub async fn new() -> Self {
        let window_size = WindowSizeHelper::new();
        let initial_inner_radius = window_size.y / 6.0;

        let mut unload_cookie = false;
        let mut cookie = SkinManager::get_texture("menu-osu", true).await;
        if cookie.is_none() {
            unload_cookie = true;
            cookie = load_image("./resources/icon.png", false, Vector2::ONE).await;
        }

        let mut cookie = cookie.expect("no cookie image?");
        cookie.set_size(Vector2::ONE * initial_inner_radius);

        Self {
            rotation: 0.0,
            data: Vec::new(),
            timer: Instant::now(),
            other_timer: Instant::now(),
            cookie,
            unload_cookie,

            bar_height: 1.0,
            initial_inner_radius,
            current_inner_radius: initial_inner_radius,

            // ripple things
            ripples: Vec::new(),
            // current_timing_point: TimingPoint::default(),
            window_size,

            last_bass: 0.0,
            last_ripple: 0.0,

            last_created: 0.0,
            created_ripple: false,
            index: 3,
        }
    }

    fn add_ripple(&mut self) {
        let mut group = TransformGroup::new(self.window_size.0 / 2.0, 50.0).alpha(1.0).border_alpha(1.0);
        let duration = 1000.0;
        let time = self.other_timer.as_millis();

        // info!("adding ripple {time}");

        group.push(Circle::new(
            Color::WHITE.alpha(0.5),
            0.0,
            Vector2::ZERO,
            self.current_inner_radius,
            Some(Border::new(Color::WHITE, 2.0))
        ));
        group.ripple(0.0, duration, time, 2.0, true, Some(0.5));

        self.ripples.push(group);
    }

    pub fn song_changed(&mut self, _new_manager: &mut Option<IngameManager>) {
        self.ripples.clear();
        self.last_bass = 0.0;
        self.last_ripple = 0.0;
        self.created_ripple = false;
    }

    pub async fn update(&mut self, _manager: &mut Option<IngameManager>) {
        if self.window_size.update() {
            self.initial_inner_radius = self.window_size.y / 6.0;
            
            self.cookie.set_size(Vector2::ONE * self.initial_inner_radius);
        }


        // see if we should add a ripple
        if self.data.len() >= self.index {

            // current bass
            let current_bass = self.data[self.index].amplitude();
            let time = self.other_timer.as_millis();

            // we've fallen below the amplitude threshold from the previous ripple to now, where we can reset the created flag
            let fall_ok = self.last_ripple - current_bass < RIPPLE_RESET;

            // the current amplitude is high enough that we can create a ripple
            let meets_min = current_bass >= RIPPLE_MIN;

            // has it been long enough since the last ripple? (at the very least, this helps prevent ripple spam)
            let timer_okay = time - self.last_created > 150.0;
            
            // is the amplitude falling?
            let falling = current_bass <= self.last_bass;


            if fall_ok && falling {
                self.created_ripple = false;
            }

            if meets_min && timer_okay && !falling && !self.created_ripple {
                self.add_ripple();
                self.last_created = time;
                self.created_ripple = true;
                self.last_ripple = current_bass;
            }

            self.last_bass = current_bass;
        }

        
        let time = self.other_timer.as_millis();
        self.ripples.retain_mut(|ripple| {
            ripple.update(time);
            ripple.visible()
        });
    }

    pub fn on_click(&self, pos:Vector2) -> bool {
        let circle_pos = self.window_size.0 / 2.0;

        let dist = (pos.x - circle_pos.x).powi(2) + (pos.y - circle_pos.y).powi(2);
        let radius = self.current_inner_radius.powi(2);

        dist <= radius
    }

    pub async fn reload_skin(&mut self) {
        // if self.unload_cookie {
        //     GameWindow::free_texture(self.cookie.tex);
        // }

        if let Some(cookie) = SkinManager::get_texture("menu-osu", true).await {
            self.cookie = cookie;
        } else {
            self.cookie = load_image("./resources/icon.png", false, Vector2::ONE).await.unwrap();
        }
        self.cookie.set_size(Vector2::ONE * self.initial_inner_radius);
    }
}

#[async_trait]
impl Visualization for MenuVisualization {
    fn lerp_factor(&self) -> f32 {10.0} // 15
    fn data(&mut self) -> &mut Vec<FFTData> { &mut self.data }
    fn timer(&mut self) -> &mut Instant { &mut self.timer }

    async fn draw(&mut self, pos:Vector2, depth:f32, list: &mut RenderableCollection) {
        let since_last = self.timer.elapsed().as_secs_f32(); // not ms
        self.update_data().await;

        let min = self.initial_inner_radius / VISUALIZATION_SIZE_FACTOR;
        let max = self.initial_inner_radius * VISUALIZATION_SIZE_FACTOR;

        if self.data.len() < 3 { return }

        
        let val = self.data[self.index].amplitude() as f32 / 500.0;
        self.current_inner_radius = f32::lerp(min, max, val).clamp(min, max);

        let rotation_increment = 0.2;
        self.rotation += rotation_increment * since_last;

        self.cookie.depth = depth - 1.0;
        self.cookie.pos = pos;
        self.cookie.rotation = self.rotation * 2.0;
        self.cookie.set_size(Vector2::ONE * self.current_inner_radius * 2.05);
        list.push(self.cookie.clone());

        // draw ripples
        for ripple in self.ripples.iter() {
            list.push(ripple.clone());
            // ripple.draw(list)
        }

        // let mut mirror = self.data.clone();
        // mirror.reverse();
        // self.data.extend(mirror);
        // let mut graph = ayyeve_piston_ui::menu::menu_elements::Graph::new(
        //     Vector2::new(0.0, _args.window_size[1] - 500.0), 
        //     Vector2::new(500.0, 500.0),
        //     self.data.iter().map(|a|a.1).collect(),
        //     0.0, 20.0
        // );
        // list.extend(ayyeve_piston_ui::menu::menu_elements::ScrollableItem::draw(&mut graph, _args, Vector2::new(0.0, 0.0), depth));
        // list.push(Box::new(Rectangle::new(
        //     Color::WHITE,
        //     depth + 10.0,
        //     Vector2::new(0.0, _args.window_size[1] - 500.0), 
        //     Vector2::new(500.0, 500.0),
        //     None
        // )));


        let a = (2.0 * PI) / self.data.len() as f32;
        let n = (2.0 * PI * self.current_inner_radius) / self.data.len() as f32 / 2.0;

        const BAR_MULT:f32 = 1.5;

        for i in 1..self.data.len() {
            let val = self.data[i].amplitude();

            if val <= CUTOFF { continue }

            let factor = (i as f32 + 2.0).log10();
            let l = self.current_inner_radius + val as f32 * factor * self.bar_height * BAR_MULT;

            let theta = self.rotation + a * i as f32;
            let cos = theta.cos();
            let sin = theta.sin();
            let p1 = pos + Vector2::new(
                cos * self.current_inner_radius,
                sin * self.current_inner_radius
            );

            let p2 = pos + Vector2::new(
                cos * l,
                sin * l
            );

            list.push(Line::new(
                p1,
                p2,
                n,
                depth + 10.0,
                // COLORS[i % COLORS.len()]
                if i == self.index { Color::RED } else { *BAR_COLOR }
            ));
        }
    }

    fn reset(&mut self) {
        self.data.clear();
        self.ripples.clear();
        // self.timer = Instant::now();
    }
}
// free up the image from the texture atlas
impl Drop for MenuVisualization {
    fn drop(&mut self) {
        if self.unload_cookie {
            GameWindow::free_texture(self.cookie.tex);
        }
    }
}