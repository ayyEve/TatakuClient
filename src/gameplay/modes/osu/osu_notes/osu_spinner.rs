use crate::prelude::*;
use super::super::prelude::*;


const SPINNER_RADIUS:f64 = 200.0;

#[derive(Clone)]
pub struct StandardSpinner {
    // def: SpinnerDef,
    pos: Vector2,
    time: f32, // ms
    end_time: f32, // ms
    last_update: f32,
    current_time: f32,

    /// current angle of the spinner
    rotation: f64,
    /// how fast the spinner is spinning
    rotation_velocity: f64,
    mouse_pos: Vector2,

    /// what was the last rotation value?
    last_mouse_angle: f64,
    /// how many rotations is needed to pass this spinner
    rotations_required: u16,
    /// how many rotations have been completed?
    rotations_completed: u16,

    /// should we count mouse movements?
    holding: bool,

    scaling_helper: Arc<ScalingHelper>,

    /// main spinny
    spinner_circle: Option<Image>,
    /// bg, no spin
    spinner_background: Option<Image>,
    /// also bg, no spin
    spinner_bottom: Option<Image>,
    /// gets smaller towards end of spinner, from 100% to 0%
    spinner_approach: Option<Image>,
}
impl StandardSpinner {
    pub async fn new(def: SpinnerDef, scaling_helper: Arc<ScalingHelper>, _diffcalc_only: bool) -> Self {
        let time = def.time;
        let end_time = def.end_time;

        Self {
            pos: scaling_helper.scale_coords(FIELD_SIZE / 2.0),
            // def,
            time, 
            end_time,
            current_time: 0.0,

            holding: false,
            rotation: 0.0,
            rotation_velocity: 0.0,
            last_mouse_angle: 0.0,
            scaling_helper,

            rotations_required: 0,
            rotations_completed: 0,
            mouse_pos: Vector2::zero(),

            last_update: 0.0,

            spinner_circle: None,
            spinner_bottom: None,
            spinner_approach: None,
            spinner_background: None,
        }
    }
}
#[async_trait]
impl HitObject for StandardSpinner {
    fn time(&self) -> f32 { self.time }
    fn end_time(&self,_:f32) -> f32 { self.end_time }
    fn note_type(&self) -> NoteType { NoteType::Spinner }

    async fn update(&mut self, beatmap_time: f32) {
        let mut diff = 0.0;
        let pos_diff = self.mouse_pos - self.pos;
        let mouse_angle = pos_diff.y.atan2(pos_diff.x);

        if beatmap_time >= self.time && beatmap_time <= self.end_time {
            // if the mouse is being held, use the mouse angle.
            // otherwise, it should be 0 since the user's spins do not count
            if self.holding {
                diff = mouse_angle - self.last_mouse_angle;
            }
            // fix diff (this is stupid)
            if diff > PI {diff -= 2.0 * PI}
            else if diff < -PI {diff += 2.0 * PI}
            
            self.rotation_velocity = diff / (beatmap_time - self.last_update) as f64;
            self.rotation += self.rotation_velocity * (beatmap_time - self.last_update) as f64;
            // self.rotation_velocity = f64::lerp(self.rotation_velocity, diff, 0.005 * (beatmap_time - self.last_update) as f64);
            // self.rotation += self.rotation_velocity * (beatmap_time - self.last_update) as f64;
            // debug!("vel: {}", self.rotation_velocity);

            // debug!("rotation: {}, diff: {}", self.rotation, diff);
        }

        self.last_mouse_angle = mouse_angle;
        self.last_update = beatmap_time;
        self.current_time = beatmap_time;
    }
    async fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if !(self.last_update >= self.time && self.last_update <= self.end_time) { return }

        let border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));

        // bg circle
        if let Some(i) = self.spinner_background.clone() {
            list.push(Box::new(i))
        } else {}

        // bottom circle
        if let Some(i) = self.spinner_bottom.clone() {
            list.push(Box::new(i))
        } else {
            if !(self.spinner_approach.is_some() || self.spinner_circle.is_some()) {
                list.push(Box::new(Circle::new(
                    Color::YELLOW,
                    -10.0,
                    self.pos,
                    SPINNER_RADIUS,
                    border.clone()
                )));
            }
        }


        // draw another circle on top which increases in radius as the counter gets closer to the reqired
        if let Some(mut i) = self.spinner_approach.clone() {
            i.current_scale = Vector2::one() * f64::lerp(1.0, 0.0, ((self.current_time - self.time) / (self.end_time - self.time)) as f64) * self.scaling_helper.scale;
            list.push(Box::new(i))
        } else {
            list.push(Box::new(Circle::new(
                Color::WHITE,
                -11.0,
                self.pos,
                SPINNER_RADIUS * (self.rotations_completed as f64 / self.rotations_required as f64),
                border.clone()
            )));
        }


        // draw line to show rotation
        if let Some(mut i) = self.spinner_circle.clone() {
            i.current_scale = Vector2::one() * self.scaling_helper.scale;
            i.current_rotation = self.rotation;
            list.push(Box::new(i))
        } else {
            let p2 = self.pos + Vector2::new(self.rotation.cos(), self.rotation.sin()) * SPINNER_RADIUS;
            list.push(Box::new(Line::new(
                self.pos,
                p2,
                5.0,
                -20.0,
                Color::GREEN
            )));
        }
        
        // draw a counter
        let rpm = (self.rotation_velocity / (2.0 * PI)) * 1000.0 * 60.0;
        let mut txt = Text::new(
            Color::BLACK,
            -999.9,
            Vector2::zero(),
            30,
            format!("{:.0}rpm", rpm.abs()), // format!("{:.0}rpm", rpm.abs()),
            get_font()
        );
        txt.center_text(Rectangle::bounds_only(
            Vector2::new(0.0, self.pos.y + 50.0),
            Vector2::new(self.pos.x * 2.0, 50.0)
        ));
        list.push(Box::new(txt));
    }

    async fn reset(&mut self) {
        self.holding = false;
        self.rotation = 0.0;
        self.rotation_velocity = 0.0;
        self.rotations_completed = 0;
    }

    async fn reload_skin(&mut self) {
        let pos = self.scaling_helper.scale_coords(FIELD_SIZE / 2.0);
        let scale = Vector2::one() * self.scaling_helper.scale;

        self.spinner_circle = IngameManager::load_texture_maybe("spinner-circle", false, |i| {
            // const SIZE:f64 = 700.0;
            i.initial_pos = pos;
            i.current_pos = pos;
            i.initial_scale = scale;
            i.current_scale = scale;
        }).await;

        self.spinner_background = IngameManager::load_texture_maybe("spinner-background", false, |i| {
            // const SIZE:f64 = 667.0; 
            i.initial_pos = pos;
            i.current_pos = pos;
            i.initial_scale = scale;
            i.current_scale = scale;
        }).await;

        self.spinner_bottom = IngameManager::load_texture_maybe("spinner-bottom", false, |i| {
            i.initial_pos = pos;
            i.current_pos = pos;
            i.initial_scale = scale;
            i.current_scale = scale;
        }).await;

        self.spinner_approach = IngameManager::load_texture_maybe("spinner-approachcircle", false, |i| {
            // const SIZE:f64 = 320.0;
            i.initial_pos = pos;
            i.current_pos = pos;
            i.initial_scale = scale;
            i.current_scale = scale;
        }).await;

    }
}
#[async_trait]
impl StandardHitObject for StandardSpinner {
    fn miss(&mut self) {}
    fn was_hit(&self) -> bool { self.last_update >= self.end_time } 
    fn get_preempt(&self) -> f32 { 0.0 }
    fn point_draw_pos(&self, _: f32) -> Vector2 { Vector2::zero() } //TODO
    fn causes_miss(&self) -> bool { self.rotations_completed < self.rotations_required } // if the spinner wasnt completed in time, cause a miss
    fn set_hitwindow_miss(&mut self, _window: f32) {}

    fn press(&mut self, _time:f32) { self.holding = true; }
    fn release(&mut self, _time:f32) { self.holding = false; }
    fn mouse_move(&mut self, pos:Vector2) { self.mouse_pos = pos; }

    async fn playfield_changed(&mut self, new_scale: Arc<ScalingHelper>) {
        let scale = Vector2::one() * new_scale.scale;

        self.pos = new_scale.scale_coords(FIELD_SIZE / 2.0);
        self.scaling_helper = new_scale;

        for i in [
            &mut self.spinner_circle, 
            &mut self.spinner_bottom,
            &mut self.spinner_background,
            &mut self.spinner_approach,
        ] {
            if let Some(i) = i {
                i.initial_pos = self.pos;
                i.current_pos = self.pos;

                i.initial_scale = scale;
                i.current_scale = scale;
            }
        }

    } 

    fn pos_at(&self, time: f32) -> Vector2 {
        // debug!("time: {}, {}, {}", time, self.time, self.end_time);

        if time < self.time || time >= self.end_time {
            
            return self.pos
        }

        let r = self.last_mouse_angle + (time - self.last_update) as f64 / (4.0*PI);
        self.pos + Vector2::new(
            r.cos(),
            r.sin()
        ) * self.scaling_helper.scale * 20.0
    }


    fn hit(&mut self, _time: f32) {}
    fn check_distance(&self, _:Vector2) -> bool { true }

    fn set_settings(&mut self, _settings: Arc<StandardSettings>) {
        // self.standard_settings = settings;
    }

    

    fn set_ar(&mut self, _ar: f32) {
        // self.time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
    }

    fn get_hitsound(&self) -> Vec<Hitsound> {
        vec![]
    }
}
