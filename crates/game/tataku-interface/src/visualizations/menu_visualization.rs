use std::f32::consts::PI;
use crate::prelude::*;
use tataku_engine_common::math::Interpolation;
use tataku::{
    Vector2,
    Color,
    Bounds,
    Border,
};
use graphics::{
    VisualizationData,
    VisualizationConfig,
};

const CUTOFF:f32 = 0.1;
pub const VISUALIZATION_SIZE_FACTOR:f32 = 1.2;


// must be above this value to create a ripple
const RIPPLE_MIN:f32 = 100.0;
const RIPPLE_RESET:f32 = 120.0;


const BAR_COLOR: Color = Color::new_rgb8(0x27, 0xbf, 0xc2);


pub struct MenuVisualization {
    // data: Vec<FFTEntry>,
    // timer: Instant, // external use only

    bar_height: f32,
    rotation: f32,

    cookie: Option<graphics::Image>,
    // initial_inner_radius: f32,
    current_inner_radius: f32,

    ripples: Vec<Ripple>,
    // current_timing_point: TimingPoint,

    bounds: Bounds,

    other_timer: tataku::Instant, // internal use only
    last_bass: f32, // last bass value. if current is less than this and we havent created a new ripple since, we should do that now
    created_ripple: bool, // have we drawn a ripple since the last fall?
    last_ripple: f32, // bass value of the last ripple

    last_created: f32, // when was the last ripple?

    // index to use for bass
    pub index: usize,

    pub vis_data: VisualizationData,

    hooked: bool,
}
impl MenuVisualization {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // let window_size = WindowSizeHelper::new();
        // let initial_inner_radius = window_size.y / 6.0;

        Self {
            rotation: 0.0,
            // data: Vec::new(),
            // timer: Instant::now(),
            other_timer: tataku::Instant::now(),
            cookie: None,

            vis_data: VisualizationData::new(VisualizationConfig {
                should_lerp: true,
                lerp_factor: 10.0
            }),

            bar_height: 1.0,
            // initial_inner_radius,
            current_inner_radius: 100.0,

            // ripple things
            ripples: Vec::new(),
            // current_timing_point: TimingPoint::default(),
            bounds: Bounds::default(),

            last_bass: 0.0,
            last_ripple: 0.0,

            last_created: 0.0,
            created_ripple: false,
            index: 3,

            hooked: false,
        }
    }

    // TODO: make sure this is correct for the new bounds
    fn inner_radius(&self) -> f32 {
        self.bounds.size.y / 6.0
    }
    fn bounds_center(&self) -> Vector2 {
        self.bounds.pos + self.bounds.size / 2.0
    }

    fn add_ripple(&mut self) {
        let time = self.other_timer.as_millis();
        let duration = 1000.0;

        let trail = graphics::Trail::new(
            self.bounds_center(),
            time,
            duration,
        );

        self.ripples.push(Ripple {
            trail,
            start_radius: self.current_inner_radius,
        });
    }

    pub fn on_click(&self, pos: Vector2) -> bool {
        let circle_pos = self.bounds_center();

        let dist = (pos.x - circle_pos.x).powi(2) 
            + (pos.y - circle_pos.y).powi(2);
        let radius = self.current_inner_radius.powi(2);

        dist <= radius
    }

    pub fn draw_cookie(&self, list: &mut graphics::RenderableCollection) {
        let Some(mut cookie) = self.cookie.clone() else { return };
        cookie.pos = self.bounds_center();
        cookie.rotation = self.rotation * 2.0;
        // cookie.set_size(Vector2::ONE * self.initial_inner_radius);
        cookie.set_size(Vector2::ONE * self.current_inner_radius * 2.05);
        list.push(cookie);
    }

    pub fn draw_vis(&self, list: &mut graphics::RenderableCollection) {
        let time = self.other_timer.as_millis();

        let pos = self.bounds_center();

        // draw ripples
        let ripples = self.ripples.iter()
            .map(|ripple| {
                ripple.trail.ripple(
                    time,
                    ripple.start_radius,
                    ripple.start_radius * 2.0,
                    Color::WHITE.alpha(0.5),
                    Some(Border::new(Color::WHITE, 2.0)),
                )
            });

        list.list.extend(ripples);

        // let since_last = self.vis_data.timer.elapsed().as_secs_f32(); // not ms
        // self.update_data().await;

        if self.vis_data.data.len() < 3 { return }
        let data = &self.vis_data.data[0..self.vis_data.data.len() / 4];

        // TODO: would it maybe be better and more efficient to do this in a shader?

        // bars
        let a = (2.0 * PI) / data.len() as f32;
        let n = (2.0 * PI * self.current_inner_radius) / data.len() as f32 / 2.0;
        const BAR_MULT:f32 = 1.5;

        for (i, val) in data
            .iter()
            .map(|a| a.amplitude())
            .enumerate() 
        {
            if val <= CUTOFF { continue }

            let factor = (i as f32 + 2.0).log10();
            let l = self.current_inner_radius 
                + val 
                * factor 
                * self.bar_height 
                * BAR_MULT;

            let theta = self.rotation + a * i as f32;
            let theta_vector = Vector2::from_angle(theta);

            let p1 = pos + theta_vector * self.current_inner_radius;
            let p2 = pos + theta_vector * l;

            list.push(graphics::Line::new(
                p1,
                p2,
                n,
                // COLORS[i % COLORS.len()]
                if i == self.index { Color::RED } else { BAR_COLOR }
            ));
        }
        
    }


    pub fn draw(&self, list: &mut graphics::RenderableCollection) {
        self.draw_vis(list);
        self.draw_cookie(list);
    }

    pub fn update(&mut self, bounds: Bounds, actions: &mut actions::ActionQueue) {
        if !self.hooked {
            self.hooked = true;
            actions.push(actions::song::SongAction::HookFFT(self.vis_data.get_hook()).into());
        }
        self.bounds = bounds;

        let rotation_increment = 0.2;
        self.rotation += rotation_increment * self.vis_data.timer.as_millis() / 1000.0;

        self.vis_data.update();

        let data = &self.vis_data.data;
        if data.len() > self.index { 
            // update our inner radius
            let inner_radius = self.inner_radius();
            let min = inner_radius / VISUALIZATION_SIZE_FACTOR;
            let max = inner_radius * VISUALIZATION_SIZE_FACTOR;
            let val = data[self.index].amplitude() / 500.0;
            let inner_radius = f32::lerp(min, max, val)
                .clamp(min, max);
            self.current_inner_radius = inner_radius;
        }

        // see if we should add a ripple
        if self.vis_data.data.len() >= self.index {

            // current bass
            let current_bass = self.vis_data.data[self.index].amplitude();
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
        self.ripples.retain(|ripple| !ripple.trail.complete(time));
    }

    pub fn reload_skin(&mut self, skin_manager: &mut dyn graphics::SkinProvider) {
        if let Some(cookie) = skin_manager.get_texture(
            Path::new("menu-osu"), 
            &graphics::TextureSource::Skin, 
            graphics::SkinUsage::Game, 
            false
        ) {
            self.cookie = Some(cookie);
        } else {
            self.cookie = skin_manager.get_texture(
                Path::new("./resources/icon.png"), 
                &graphics::TextureSource::Raw, 
                graphics::SkinUsage::Game, 
                false
            );
        }
    }

    pub fn song_changed(&mut self) {
        self.ripples.clear();
        self.last_bass = 0.0;
        self.last_ripple = 0.0;
        self.created_ripple = false;
    }


    pub fn reset(&mut self) {
        self.vis_data.reset();
        self.ripples.clear();
        // self.timer = Instant::now();
    }
}

struct Ripple {
    trail: graphics::Trail,
    start_radius: f32,
}
