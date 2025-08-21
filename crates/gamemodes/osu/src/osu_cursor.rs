use crate::prelude::*;

const TRAIL_CREATE_TIMER:f32 = 10.0;
const TRAIL_FADEOUT_TIMER_START:f32 = 20.0;
const TRAIL_FADEOUT_TIMER_DURATION:f32 = 100.0;

const TRAIL_CREATE_TIMER_IF_MIDDLE:f32 = 0.1;
const TRAIL_FADEOUT_TIMER_START_IF_MIDDLE:f32 = 20.0;
const TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE:f32 = 500.0;

const DEFAULT_CURSOR_SIZE:f32 = 10.0;
const PRESSED_CURSOR_SCALE:f32 = 1.2;


pub struct OsuCursor {
    /// position of the visible cursor
    pub pos: Vector2,
    last_pos: Vector2,

    pub note_radius: f32,
    // ripple_image: Option<Image>,

    cursor_rotation: f32,

    pub cursor_image: Option<Image>,
    pub cursor_middle_image: Option<Image>,
    pub cursor_trail_image: Option<Image>,
    pub trails: Vec<Trail>,
    last_trail_time: f32,

    trail_create_timer: f32,
    trail_fadeout_timer_start: f32,
    trail_fadeout_timer_duration: f32,

    left_pressed: bool,
    right_pressed: bool,

    skin: SkinSettings,
    beatmap_path: String,

    ripples: Vec<Trail>,
    time: TatakuInstant,

    left_emitter: Emitter,
    right_emitter: Emitter,
    pub emitter_enabled: bool,

    settings: CursorSettings,
}

impl OsuCursor {
    pub fn new(
        note_radius: f32,
        skin: SkinSettings,
        beatmap_path: String,
        settings: &Settings
    ) -> Self {
        let a = PI / 4.0;
        let builder = EmitterBuilder::default()
            .spawn_delay(20.0)
            .angle(EmitterVal::init_only(-a..a))
            .speed(EmitterVal::init_only(0.1..0.5))
            .scale(EmitterVal::init_only(0.3..0.6))
            .life(100.0..300.0)
            .opacity(EmitterVal::new(1.0..1.0, 1.0..0.0))
            .rotation(EmitterVal::new(0.0..0.0, 0.0001..0.0002))
            .should_emit(false)
            .color(Color::WHITE)
            .image(Arc::default());

        let right_emitter = builder.clone().build(0.0);
        let left_emitter = builder.angle(EmitterVal::init_only(-a-PI..a-PI)).build(0.0);

        Self {
            pos: Vector2::ZERO,
            last_pos: Vector2::ZERO,

            trails: Vec::new(),
            cursor_image: None,
            cursor_middle_image: None,
            cursor_trail_image: None,
            last_trail_time: 0.0,
            // ripple_image,

            trail_create_timer: TRAIL_CREATE_TIMER,
            trail_fadeout_timer_start: TRAIL_FADEOUT_TIMER_START,
            trail_fadeout_timer_duration: TRAIL_FADEOUT_TIMER_DURATION,
            cursor_rotation: 0.0,
            emitter_enabled: true,


            left_pressed: false,
            right_pressed: false,
            note_radius,
            skin,
            beatmap_path,

            ripples: Vec::new(),
            time: TatakuInstant::now(),

            left_emitter,
            right_emitter,

            settings: settings.cursor_settings.clone(),
        }
    }
    
    pub fn init(&self, actions: &mut ActionQueue) {
        actions.push(WindowAction::AddEmitter(self.left_emitter.get_ref()));
        actions.push(WindowAction::AddEmitter(self.right_emitter.get_ref()));
    }

    fn add_ripple(&mut self) {
        let duration = 500.0;
        let time = self.time.as_millis();

        self.ripples.push(Trail::new(self.pos, time, duration));
    }


    // fn make_trail_group(
    //     pos: Vector2,
    //     mut trail: Image,
    //     start: f32,
    //     duration: f32,
    //     scale: Vector2,
    //     time: f32
    // ) -> TransformGroup {
    //     trail.scale = scale;
    //     trail.set_blend_mode(Pipeline::SourceAlphaBlending);
    //     let mut g = TransformGroup::new(pos).alpha(1.0).border_alpha(0.0);
    //     g.transforms.push(Transformation::new(
    //         start,
    //         duration,
    //         TransformType::Transparency { start: 1.0, end: 0.0 },
    //         Easing::EaseOutSine,
    //         time
    //     ));
    //     g.push(trail);
    //     g
    // }

    pub fn reset(&mut self) {
        let time = -2000.0;
        self.left_emitter.reset(time);
        self.right_emitter.reset(time);
        self.ripples.clear();
        self.trails.clear();
        self.last_trail_time = time;
    }


    pub fn left_pressed(&mut self, pressed: bool) {
        self.left_pressed = pressed;
        self.left_emitter.should_emit = pressed;
        if pressed && self.settings.cursor_ripples {
            self.add_ripple();
        }
    }
    pub fn right_pressed(&mut self, pressed: bool) {
        self.right_pressed = pressed;
        self.right_emitter.should_emit = pressed;
        if pressed && self.settings.cursor_ripples {
            self.add_ripple();
        }
    }
    pub fn cursor_pos(&mut self, pos: Vector2) {
        self.pos = pos;
    }

    pub fn update(&mut self) {
        let time = self.time.as_millis();

        if self.emitter_enabled {
            self.left_emitter.position = self.pos;
            self.right_emitter.position = self.pos;
            self.left_emitter.update(time);
            self.right_emitter.update(time);
        }


        if self.skin.cursor_rotate {
            self.cursor_rotation = (time / 2000.0) % (PI * 2.0);
        }

        // trail stuff
        self.add_trails();

        self.trails.retain(|trail| !trail.complete(time));
        self.ripples.retain(|ripple| !ripple.complete(time) );
    }

    pub fn add_trails(&mut self) {
        let time = self.time.as_millis();

        if self.last_pos == self.pos { return; }

        // check if we should add a new trail
        let is_solid_trail = self.cursor_middle_image.is_some();

        if let Some(trail) = self.cursor_trail_image.as_ref() {
            if is_solid_trail {
                // solid trail, a bit more to check
                let width = trail.size().x;
                let dist = self.pos.distance(self.last_pos);
                let count = (2.5 * dist / width).ceil() as i32;

                if dist < width { return; }

                for i in 0..count {
                    let pos = Vector2::lerp(self.last_pos, self.pos, i as f32 / count as f32);

                    self.trails.push(Trail::new(
                        pos,
                        time + self.trail_fadeout_timer_start,
                        self.trail_fadeout_timer_duration,
                    ));
                }

                if count > 0 {
                    self.last_pos = self.pos;
                }
            } else if time - self.last_trail_time >= self.trail_create_timer {
                // not a solid trail, just follow the timer
                self.last_trail_time = time;
                self.last_pos = self.pos;

                self.trails.push(Trail::new(
                    self.pos,
                    time + self.trail_fadeout_timer_start,
                    self.trail_fadeout_timer_duration,
                ));
            }
        }
    }

    pub fn draw_above(&self, list: &mut RenderableCollection) {
        let time = self.time.as_millis();

        let mut radius = DEFAULT_CURSOR_SIZE;
        if self.left_pressed || self.right_pressed {
            radius *= PRESSED_CURSOR_SCALE;
        }

        if let Some(image) = &self.cursor_trail_image {
            let mut image = image.clone();
            image.scale = Vector2::ONE * self.settings.cursor_scale;
            image.set_blend_mode(Pipeline::SourceAlphaBlending);

            let trails = self.trails.iter()
                .map(|trail| {
                    let mut image = image.clone();

                    image.color.a = ((1.0 - trail.progress(time))
                        .clamp(0.0, 1.0) * 255.0)
                         as u8;
                    image.pos = trail.position;

                    Box::new(image) as Box<dyn TatakuRenderable>
                });

            list.list.extend(trails);
        }

        if self.emitter_enabled {
            self.left_emitter.draw(list);
            self.right_emitter.draw(list);
        }


        // draw cursor itself
        if let Some(mut cursor) = self.cursor_image.clone() {
            cursor.pos = self.pos;
            cursor.rotation = self.cursor_rotation;
            // cursor.color = self.color;

            if self.left_pressed || self.right_pressed {
                cursor.scale = Vector2::ONE * PRESSED_CURSOR_SCALE * self.settings.cursor_scale;
            } else {
                cursor.scale = Vector2::ONE * self.settings.cursor_scale;
            }

            list.push(cursor);
        } else {
            list.push(Circle::new(
                self.pos,
                radius * self.settings.cursor_scale,
                *self.settings.cursor_color,
            ).border_maybe(
                if self.settings.cursor_border > 0.0 {
                    Some(Border::new(
                        *self.settings.cursor_border_color,
                        self.settings.cursor_border
                    ))
                } else { None }
            ));
        }


        if let Some(mut cursor) = self.cursor_middle_image.clone() {
            cursor.pos = self.pos;
            cursor.scale = Vector2::ONE * self.settings.cursor_scale;

            list.push(cursor);
        }
    }

    pub fn draw_below(&self, list: &mut RenderableCollection) {
        // draw ripples
        for ripple in self.ripples.iter() {
            list.list.push(ripple.ripple(
                self.time.as_millis(),
                0.0,
                self.settings.cursor_ripple_final_radius,
                self.settings.cursor_ripple_color.alpha(0.2),
                Some(Border::new(self.settings.cursor_ripple_color.alpha(0.5), 2.0))
            ));
        }
    }


    #[cfg(feature="graphics")]
    pub fn reload_skin(
        &mut self,
        skin_manager: &mut dyn SkinProvider,
    ) {
        let source = if self.settings.beatmap_cursor { TextureSource::Beatmap(self.beatmap_path.clone()) } else { TextureSource::Skin };

        self.cursor_image = skin_manager.get_texture("cursor", &source, SkinUsage::Gamemode, false);
        self.cursor_trail_image = skin_manager.get_texture("cursortrail", &source, SkinUsage::Gamemode, false);
        self.cursor_middle_image = skin_manager.get_texture("cursormiddle", &source, SkinUsage::Gamemode, false);

        let (trail_create_timer, trail_fadeout_timer_start, trail_fadeout_timer_duration) = if self.cursor_middle_image.is_some() {
            (TRAIL_CREATE_TIMER_IF_MIDDLE, TRAIL_FADEOUT_TIMER_START_IF_MIDDLE, TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE)
        } else {
            (TRAIL_CREATE_TIMER, TRAIL_FADEOUT_TIMER_START, TRAIL_FADEOUT_TIMER_DURATION)
        };

        self.trail_create_timer = trail_create_timer;
        self.trail_fadeout_timer_start = trail_fadeout_timer_start;
        self.trail_fadeout_timer_duration = trail_fadeout_timer_duration;

        self.cursor_rotation = 0.0;


        let tex = skin_manager.get_texture("star2", &source, SkinUsage::Gamemode, false).map(|t| t.tex).unwrap_or_default();
        self.left_emitter.image = tex.clone();
        self.right_emitter.image = tex;
    }

}
