#![allow(unused)]
use crate::prelude::*;
use super::Visualization;

const CUTOFF:f32 = 0.1;
pub const VISUALIZATION_SIZE_FACTOR:f64 = 1.2;


pub struct MenuVisualization {
    data: Vec<FFTData>,
    timer: Instant,

    bar_height: f64,
    rotation: f64,

    cookie: Image,
    initial_inner_radius: f64,
    current_inner_radius: f64,

    ripples: Vec<TransformGroup>,
    last_ripple_at: f32,
    current_timing_point: TimingPoint,

    window_size: WindowSizeHelper,

}
impl MenuVisualization {
    pub async fn new() -> Self {
        let window_size = WindowSizeHelper::new();
        let initial_inner_radius = window_size.y / 6.0;

        Self {
            rotation: 0.0,
            data: Vec::new(),
            timer: Instant::now(),
            cookie: load_image("./resources/icon.png", false, Vector2::ONE).await.unwrap(), //Image::from_path("./resources/icon.png", Vector2::ZERO, 0.0, Vector2::ONE * initial_inner_radius).await.unwrap(),

            bar_height: 1.0,
            initial_inner_radius,
            current_inner_radius: initial_inner_radius,

            // ripple things
            ripples: Vec::new(),
            last_ripple_at: 0.0,
            current_timing_point: TimingPoint::default(),
            window_size
        }
    }

    fn check_ripple(&mut self, time: f32) {
        let next_ripple = self.last_ripple_at + self.current_timing_point.beat_length;

        if self.last_ripple_at == 0.0 || time >= next_ripple {
            self.last_ripple_at = time;
            let mut group = TransformGroup::new(self.window_size.0 / 2.0, 10.0).alpha(1.0).border_alpha(1.0);
            let duration = 1000.0;

            group.push(Circle::new(
                Color::WHITE.alpha(0.5),
                0.0,
                Vector2::ONE,
                self.initial_inner_radius / VISUALIZATION_SIZE_FACTOR,
                Some(Border::new(Color::WHITE, 2.0))
            ));
            group.ripple(0.0, duration, time as f64, 2.0, true, Some(0.5));

            self.ripples.push(group);
        }

        let time = time as f64;
        self.ripples.retain_mut(|ripple| {
            ripple.update(time);
            ripple.visible()
        });
    }

    pub fn song_changed(&mut self, new_manager: &mut Option<IngameManager>) {
        self.last_ripple_at = 0.0;
        self.ripples.clear();
        if let Some(new_manager) = new_manager {
            let time = new_manager.time();
            self.current_timing_point = new_manager.timing_point_at(time, false).clone();
        }
    }

    pub async fn update(&mut self, manager: &mut Option<IngameManager>) {
        self.window_size.update();

        // update ripples
        if let Some(manager) = manager {
            let time = manager.time();
            let current_timing_point = manager.timing_point_at(time, false);

            // timing point changed
            if current_timing_point.time != self.current_timing_point.time {
                self.last_ripple_at = 0.0;
                self.current_timing_point = current_timing_point.clone();
            }
            
            self.check_ripple(time);
        } else if let Some(song) = AudioManager::get_song().await {
            self.check_ripple(song.get_position());
        }
    }

    pub fn on_click(&self, pos:Vector2) -> bool {
        let circle_pos = self.window_size.0 / 2.0;

        let dist = (pos.x - circle_pos.x).powi(2) + (pos.y - circle_pos.y).powi(2);
        let radius = self.current_inner_radius.powi(2);

        dist <= radius
    }
}

#[async_trait]
impl Visualization for MenuVisualization {
    fn lerp_factor(&self) -> f32 {10.0} // 15
    fn data(&mut self) -> &mut Vec<FFTData> {&mut self.data}
    fn timer(&mut self) -> &mut Instant {&mut self.timer}

    async fn draw(&mut self, _args:piston::RenderArgs, pos:Vector2, depth:f64, list: &mut RenderableCollection) {
        let since_last = self.timer.elapsed().as_secs_f64();
        self.update_data().await;

        let min = self.initial_inner_radius / VISUALIZATION_SIZE_FACTOR;
        let max = self.initial_inner_radius * VISUALIZATION_SIZE_FACTOR;

        if self.data.len() < 3 {return}

        let val = self.data[3].amplitude() as f64 / 500.0;
        self.current_inner_radius = f64::lerp(min, max, val).clamp(min, max);

        let rotation_increment = 0.2;
        self.rotation += rotation_increment * since_last;

        // draw cookie
        // let s = self.cookie.initial_scale;
        // let s2 = s * val;
        // self.cookie.current_scale = Vector2::new(
        //     s2.x.clamp(s.x / 1.1, s.x * 1.1), 
        //     s2.y.clamp(s.y / 1.1, s.y * 1.1)
        // );

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


        let a = (2.0 * PI) / self.data.len() as f64;
        let n = (2.0 * PI * self.current_inner_radius) / self.data.len() as f64 / 2.0;

        const BAR_MULT:f64 = 1.5;

        for i in 0..self.data.len() {
            let val = self.data[i].amplitude();


            if val <= CUTOFF { continue }

            let factor = (i as f64 + 2.0).log10();
            let l = self.current_inner_radius + val as f64 * factor * self.bar_height * BAR_MULT;

            let theta = self.rotation + a * i as f64;
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
                depth,
                // COLORS[i % COLORS.len()]
                Color::from_hex("#27bfc2")
            ));
        }
    }

    fn reset(&mut self) {
        self.data = Vec::new();
        self.timer = Instant::now();
    }
}
