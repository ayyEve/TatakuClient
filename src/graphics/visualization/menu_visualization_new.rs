use crate::prelude::*;
use super::{FFTEntry, Visualization};

const CUTOFF:f32 = 0.1;
const VISUALIZATION_SIZE_FACTOR:f64 = 1.2;


// must be above this value to create a ripple
const RIPPLE_MIN:f32 = 100.0;

const RIPPLE_RESET:f32 = 120.0;

lazy_static::lazy_static! {
    static ref BAR_COLOR:Color = Color::from_hex("#27bfc2");
}


pub struct MenuVisualizationNew {
    data: Vec<FFTEntry>,
    timer: Instant, // external use only

    bar_height: f64,
    rotation: f64,

    cookie: Image,
    initial_inner_radius: f64,
    current_inner_radius: f64,

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
impl MenuVisualizationNew {
    pub async fn new() -> Self {
        let window_size = WindowSizeHelper::new().await;
        let initial_inner_radius = window_size.y / 6.0;

        Self {
            rotation: 0.0,
            data: Vec::new(),
            timer: Instant::now(),
            other_timer: Instant::now(),
            //TODO!: skins
            cookie: Image::from_path("./resources/icon.png", Vector2::zero(), 0.0, Vector2::one() * initial_inner_radius).await.unwrap(),

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
        let mut group = TransformGroup::new();
        let duration = 1000.0;
        let time = self.other_timer.as_millis();

        // info!("adding ripple {time}");

        group.items.push(DrawItem::Circle(Circle::new(
            Color::WHITE.alpha(0.5),
            50.0,
            self.window_size.0 / 2.0,
            self.current_inner_radius,
            Some(Border::new(Color::WHITE, 2.0))
        )));
        group.ripple(0.0, duration, time as f64, 2.0, true, Some(0.5));

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
            
            self.cookie.set_size(Vector2::one() * self.initial_inner_radius);
        }


        // see if we should add a ripple
        if self.data.len() >= self.index {

            // current bass
            let current_bass = self.data[self.index];
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

        
        let time = self.other_timer.as_millis64();
        self.ripples.retain_mut(|ripple| {
            ripple.update(time);
            ripple.items[0].visible()
        });
    }

    pub fn on_click(&self, pos:Vector2) -> bool {
        let circle_pos = self.window_size.0 / 2.0;

        let dist = (pos.x - circle_pos.x).powi(2) + (pos.y - circle_pos.y).powi(2);
        let radius = self.current_inner_radius.powi(2);

        dist <= radius
    }
}

#[async_trait]
impl Visualization for MenuVisualizationNew {
    fn lerp_factor(&self) -> f32 {10.0} // 15
    fn data(&mut self) -> &mut Vec<FFTEntry> {&mut self.data}
    fn timer(&mut self) -> &mut Instant {&mut self.timer}

    async fn draw(&mut self, _args:piston::RenderArgs, pos:Vector2, depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        let since_last = self.timer.elapsed().as_secs_f64(); // not ms
        self.update_data().await;

        let min = self.initial_inner_radius / VISUALIZATION_SIZE_FACTOR;
        let max = self.initial_inner_radius * VISUALIZATION_SIZE_FACTOR;

        if self.data.len() < 3 { return }

        
        #[cfg(feature="bass_audio")]
        let val = self.data[self.index] as f64 / 500.0;
        
        #[cfg(feature="neb_audio")]
        let val = self.data[self.index].1 as f64 / 500.0;
        self.current_inner_radius = f64::lerp(min, max, val).clamp(min, max);

        let rotation_increment = 0.2;
        self.rotation += rotation_increment * since_last;

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

        for i in 1..self.data.len() {
            #[cfg(feature="bass_audio")]
            let val = self.data[i];
            #[cfg(feature="neb_audio")]
            let val = self.data[i].1;


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

            list.push(Box::new(Line::new(
                p1,
                p2,
                n,
                depth + 10.0,
                // COLORS[i % COLORS.len()]
                if i == self.index { Color::RED } else { *BAR_COLOR }
            )));
        }
    }

    fn reset(&mut self) {
        self.data.clear();
        self.ripples.clear();
        // self.timer = Instant::now();
    }
}
