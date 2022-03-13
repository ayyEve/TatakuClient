use crate::prelude::*;
use super::{FFTEntry, Visualization};

const CUTOFF:f32 = 0.1;
pub const VISUALIZATION_SIZE_FACTOR:f64 = 1.2;

pub fn visualization_initial_radius() -> f64 {
    Settings::window_size().y / 6.0
}


pub struct MenuVisualization {
    data: Vec<FFTEntry>,
    timer: Instant,

    bar_height: f64,
    rotation: f64,

    cookie: Image,
    initial_inner_radius: f64,
    current_inner_radius: f64,

    ripples: Vec<TransformGroup>,
    last_ripple_at: f32,
    current_timing_point: TimingPoint,
}
impl MenuVisualization {
    pub fn new() -> Self {
        let initial_inner_radius  = visualization_initial_radius();
        Self {
            rotation: 0.0,
            data: Vec::new(),
            timer: Instant::now(),
            //TODO!: skins
            cookie: Image::from_path("./resources/icon.png", Vector2::zero(), 0.0, Vector2::one() * initial_inner_radius).unwrap(),

            bar_height: 1.0,
            initial_inner_radius,
            current_inner_radius: initial_inner_radius,

            // ripple things
            ripples: Vec::new(),
            last_ripple_at: 0.0,
            current_timing_point: TimingPoint::default()
        }
    }

    fn check_ripple(&mut self, time: f32) {
        let next_ripple = self.last_ripple_at + self.current_timing_point.beat_length;

        if self.last_ripple_at == 0.0 || time >= next_ripple {
            self.last_ripple_at = time;
            let mut group = TransformGroup::new();
            let duration = 1000.0;

            group.items.push(DrawItem::Circle(Circle::new(
                Color::WHITE.alpha(0.5),
                10.0,
                Settings::window_size() / 2.0,
                self.initial_inner_radius / VISUALIZATION_SIZE_FACTOR,
                Some(Border::new(Color::WHITE, 2.0))
            )));
            group.ripple(0.0, duration, time as f64, 2.0, true, Some(0.5));

            self.ripples.push(group);
        }

        let time = time as f64;
        self.ripples.retain_mut(|ripple| {
            ripple.update(time);
            ripple.items[0].visible()
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

    pub fn update(&mut self, manager: &mut Option<IngameManager>) {
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
        } else if let Some(song) = Audio::get_song() {
            #[cfg(feature="bass_audio")]
            if let Ok(pos) = song.get_position() {
                self.check_ripple(pos as f32)
            }
            
            #[cfg(feature="neb_audio")] 
            self.check_ripple(song.current_time())
        }
    }

    pub fn on_click(&self, pos:Vector2) -> bool {
        let circle_pos = Settings::window_size() / 2.0;

        let dist = (pos.x - circle_pos.x).powi(2) + (pos.y - circle_pos.y).powi(2);
        let radius = self.current_inner_radius.powi(2);

        dist <= radius
    }
}

impl Visualization for MenuVisualization {
    fn lerp_factor(&self) -> f32 {10.0} // 15
    fn data(&mut self) -> &mut Vec<FFTEntry> {&mut self.data}
    fn timer(&mut self) -> &mut Instant {&mut self.timer}

    fn draw(&mut self, _args:piston::RenderArgs, pos:Vector2, depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        let since_last = self.timer.elapsed().as_secs_f64();
        self.update_data();

        let min = self.initial_inner_radius / VISUALIZATION_SIZE_FACTOR;
        let max = self.initial_inner_radius * VISUALIZATION_SIZE_FACTOR;

        if self.data.len() < 3 {return}

        
        #[cfg(feature="bass_audio")]
        let val = self.data[3] as f64 / 500.0;
        
        #[cfg(feature="neb_audio")]
        let val = self.data[3].1 as f64 / 500.0;
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
        self.cookie.current_pos = pos;
        self.cookie.current_rotation = self.rotation * 2.0;
        self.cookie.set_size(Vector2::one() * self.current_inner_radius * 2.05);
        list.push(Box::new(self.cookie.clone()));

        // draw ripples
        for ripple in self.ripples.iter_mut() {
            ripple.draw(list)
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
            #[cfg(feature="bass_audio")]
            let val = self.data[i];
            #[cfg(feature="neb_audio")]
            let val = self.data[i].1;


            if val <= CUTOFF {continue}

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

            list.push(Box::new(Line::new(
                p1,
                p2,
                n,
                depth,
                // COLORS[i % COLORS.len()]
                Color::from_hex("#27bfc2")
            )));
        }
    }

    fn reset(&mut self) {
        self.data = Vec::new();
        self.timer = Instant::now();
    }
}
