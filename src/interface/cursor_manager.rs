/**
 * Cursor Manager
 * 
 * this uses an mpsc channel because it may be inaccessible to things that need access
 * ie, a gamemode might want to hide the cursor, however it does not have direct access to the cursor field in game
 */
use crate::prelude::*;
use tokio::sync::mpsc::{Sender, Receiver, channel};

const TRAIL_CREATE_TIMER:f32 = 10.0;
const TRAIL_FADEOUT_TIMER_START:f32 = 20.0;
const TRAIL_FADEOUT_TIMER_DURATION:f32 = 100.0;

const TRAIL_CREATE_TIMER_IF_MIDDLE:f32 = 0.1;
const TRAIL_FADEOUT_TIMER_START_IF_MIDDLE:f32 = 20.0;
const TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE:f32 = 500.0;

const DEFAULT_CURSOR_SIZE:f32 = 10.0;
const PRESSED_CURSOR_SCALE:f32 = 1.2;


static CURSOR_EVENT_QUEUE:OnceCell<Sender<CursorEvent>> = OnceCell::const_new();

pub struct CursorManager {
    /// position of the visible cursor
    pub pos: Vector2,
    last_pos: Vector2,

    // cached settings
    pub color: Color,
    pub border_color: Color,
    ripple_color: Color,
    ripple_radius_override: Option<f32>,
    // ripple_image: Option<Image>,

    cursor_rotation: f32,

    pub cursor_image: Option<Image>,
    pub cursor_middle_image: Option<Image>,
    pub cursor_trail_image: Option<Image>,
    pub trail_images: Vec<TransformGroup>,
    last_trail_time: f32,

    trail_create_timer: f32,
    trail_fadeout_timer_start: f32,
    trail_fadeout_timer_duration: f32,

    // event_receiver: Arc<Receiver<CursorEvent>>,
    event_receiver: Receiver<CursorEvent>,

    // event vals

    /// should the cursor be visible?
    visible: bool,
    show_system_cursor: bool,
    /// is the game mode overriding the cursor pos?
    gamemode_override: bool,

    left_pressed: bool,
    right_pressed: bool,

    settings: SettingsHelper,
    current_skin: CurrentSkinHelper,

    ripples: Vec<TransformGroup>,
    time: Instant,

    left_emitter: Emitter,
    right_emitter: Emitter,
}

impl CursorManager {
    pub async fn new() -> Self {
        let cursor_image = SkinManager::get_texture("cursor", true).await;
        let cursor_trail_image = SkinManager::get_texture("cursortrail", true).await;
        let cursor_middle_image = SkinManager::get_texture("cursormiddle", false).await;

        let (trail_create_timer, trail_fadeout_timer_start, trail_fadeout_timer_duration) = if cursor_middle_image.is_some() {
            (TRAIL_CREATE_TIMER_IF_MIDDLE, TRAIL_FADEOUT_TIMER_START_IF_MIDDLE, TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE)
        } else {
            (TRAIL_CREATE_TIMER, TRAIL_FADEOUT_TIMER_START, TRAIL_FADEOUT_TIMER_DURATION)
        };

        // let ripple_image = SkinManager::get_texture("cursor-ripple", true).await;

        let (sender, event_receiver) = channel(1000);
        CURSOR_EVENT_QUEUE.set(sender).expect("Cursor event queue already exists");

        let a = PI / 4.0;
        let tex = SkinManager::get_texture("star", true).await.map(|t|t.tex).unwrap_or_default();
        let builder = EmitterBuilder::new()
            .spawn_delay(20.0)
            .angle(EmitterVal::init_only(-a..a))
            .speed(EmitterVal::init_only(0.1..0.5))
            .scale(EmitterVal::init_only(0.3..0.6))
            .life(100.0..300.0)
            .opacity(EmitterVal::new(1.0..1.0, 1.0..0.0))
            .rotation(EmitterVal::new(0.0..0.0, 0.0001..0.0002))
            .should_emit(false)
            .color(Color::WHITE)
            .image(tex);

        let right_emitter = builder.clone().build(0.0);
        let left_emitter = builder.angle(EmitterVal::init_only(-a-PI..a-PI)).build(0.0);

        let settings = SettingsHelper::new();
        Self {
            pos: Vector2::ZERO,
            last_pos: Vector2::ZERO,
            color: Color::from_hex(&settings.cursor_color),
            border_color: Color::from_hex(&settings.cursor_border_color),
            ripple_color: Color::from_hex(&settings.cursor_ripple_color),
            
            current_skin: CurrentSkinHelper::new(),

            trail_images: Vec::new(),
            cursor_image,
            cursor_middle_image,
            cursor_trail_image,
            last_trail_time: 0.0,
            // ripple_image,

            trail_create_timer, 
            trail_fadeout_timer_start,
            trail_fadeout_timer_duration,
            cursor_rotation: 0.0,

            event_receiver,

            left_pressed: false,
            right_pressed: false,
            visible: true,
            show_system_cursor: false,
            ripple_radius_override: None,
            gamemode_override: false,
            settings,

            ripples: Vec::new(),
            time: Instant::now(),

            left_emitter,
            right_emitter,
        }
    }


    pub async fn reload_skin(&mut self) {
        self.cursor_image = SkinManager::get_texture("cursor", true).await;
        self.cursor_trail_image = SkinManager::get_texture("cursortrail", true).await;
        self.cursor_middle_image = SkinManager::get_texture("cursormiddle", false).await;

        let (trail_create_timer, trail_fadeout_timer_start, trail_fadeout_timer_duration) = if self.cursor_middle_image.is_some() {
            (TRAIL_CREATE_TIMER_IF_MIDDLE, TRAIL_FADEOUT_TIMER_START_IF_MIDDLE, TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE)
        } else {
            (TRAIL_CREATE_TIMER, TRAIL_FADEOUT_TIMER_START, TRAIL_FADEOUT_TIMER_DURATION)
        };

        self.trail_create_timer = trail_create_timer;
        self.trail_fadeout_timer_start = trail_fadeout_timer_start;
        self.trail_fadeout_timer_duration = trail_fadeout_timer_duration;

        self.cursor_rotation = 0.0;

        
        let star = SkinManager::get_texture("star", true).await.expect("no star image");
        self.left_emitter.image = star.tex;
        self.right_emitter.image = star.tex;
    }


    pub async fn update(&mut self, time: f32, cursor_pos: Vector2) {

        // check settings update 
        if self.settings.update() {
            self.color = Color::from_hex(&self.settings.cursor_color);
            self.border_color = Color::from_hex(&self.settings.cursor_border_color);
            self.ripple_color = Color::from_hex(&self.settings.cursor_ripple_color);
        }

        if !self.show_system_cursor && !self.gamemode_override {
            self.pos = cursor_pos;
        }

        self.left_emitter.position = self.pos;
        self.right_emitter.position = self.pos;
        self.left_emitter.update(time);
        self.right_emitter.update(time);

        // work through the event queue
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                CursorEvent::OverrideRippleRadius(radius_maybe) => self.ripple_radius_override = radius_maybe,
                CursorEvent::SetVisible(show) => self.visible = show,

                CursorEvent::SetLeftDown(down, is_gamemode) => {
                    if is_gamemode || (!is_gamemode && !self.show_system_cursor) {
                        self.left_pressed = down;
                        self.left_emitter.should_emit = down;
                        if down && self.settings.cursor_ripples {
                            self.add_ripple()
                        }
                    }
                }
                CursorEvent::SetRightDown(down, is_gamemode) => {
                    if is_gamemode || (!is_gamemode && !self.show_system_cursor) {
                        self.right_pressed = down;
                        self.right_emitter.should_emit = down;
                        if down && self.settings.cursor_ripples {
                            self.add_ripple()
                        }
                    }
                }

                CursorEvent::SetPos(pos) => self.pos = pos,
                CursorEvent::GameModeOverride(enabled) => self.gamemode_override = enabled,

                CursorEvent::ShowSystemCursor(show) => {
                    self.show_system_cursor = show;
                    if show {
                        GameWindow::send_event(Game2WindowEvent::ShowCursor);
                    } else {
                        GameWindow::send_event(Game2WindowEvent::HideCursor);
                    }
                }
            }
        }


        if self.current_skin.update() {
            self.reload_skin().await;
        }

        if self.current_skin.cursor_rotate {
            self.cursor_rotation = (time / 2000.0) % (PI * 2.0);
        }

        // trail stuff

        // check if we should add a new trail
        let is_solid_trail = self.cursor_middle_image.is_some();

        if let Some(trail) = self.cursor_trail_image.as_ref().filter(|_|self.last_pos != self.pos) {
            let scale = Vector2::ONE * self.settings.cursor_scale;

            if is_solid_trail {
                // solid trail, a bit more to check
                let width = trail.size().x;
                let dist = self.pos.distance(self.last_pos);
                let count = (dist / width).ceil() as i32;

                for i in 0..count {
                    let pos = Vector2::lerp(self.last_pos, self.pos, i as f32 / count as f32);
                    self.trail_images.push(Self::make_trail_group(
                        pos, 
                        trail.clone(), 
                        self.trail_fadeout_timer_start, 
                        self.trail_fadeout_timer_duration,
                        scale,
                        time
                    ));
                }

                if count > 0 {
                    self.last_pos = self.pos;
                }
            } else if time - self.last_trail_time >= self.trail_create_timer {
                // not a solid trail, just follow the timer
                self.last_trail_time = time;
                self.last_pos = self.pos;
                self.trail_images.push(Self::make_trail_group(
                    self.pos, 
                    trail.clone(), 
                    self.trail_fadeout_timer_start, 
                    self.trail_fadeout_timer_duration,
                    scale, 
                    time
                ));
            }

        }

    
        // update the transforms, removing any that are not visible
        self.trail_images.retain_mut(|i| {
            i.update(time);
            i.visible()
        });

        // update ripples
        let time = self.time.as_millis();
        self.ripples.retain_mut(|ripple| {
            ripple.update(time);
            ripple.visible()
        });

    }

    pub fn draw_ripples(&self, list: &mut RenderableCollection) {
        if !self.visible { return }
        
        // draw ripples
        for ripple in self.ripples.iter() {
            list.push(ripple.clone())
            // ripple.draw(list)
        }
    }

    pub fn draw(&mut self, list: &mut RenderableCollection) {
        if !self.visible { return }

        let mut radius = DEFAULT_CURSOR_SIZE;
        if self.left_pressed || self.right_pressed {
            radius *= PRESSED_CURSOR_SCALE;
        }

        if self.cursor_trail_image.is_some() {
            // draw the transforms
            for i in self.trail_images.iter() {
                // i.draw(list);
                list.push(i.clone())
            }
        }

        self.left_emitter.draw(list);
        self.right_emitter.draw(list);
        

        // draw cursor itself
        if let Some(cursor) = &self.cursor_image {
            let mut cursor = cursor.clone();
            cursor.pos = self.pos;
            cursor.rotation = self.cursor_rotation;
            // cursor.current_color = self.color;
            
            if self.left_pressed || self.right_pressed {
                cursor.scale = Vector2::ONE * PRESSED_CURSOR_SCALE * self.settings.cursor_scale;
            } else {
                cursor.scale = Vector2::ONE * self.settings.cursor_scale;
            }
            
            list.push(cursor.clone());
        } else {
            list.push(Circle::new(
                self.pos,
                radius * self.settings.cursor_scale,
                self.color,
                if self.settings.cursor_border > 0.0 {
                    Some(Border::new(
                        self.border_color,
                        self.settings.cursor_border
                    ))
                } else { None }
            ));
        }
    
        
        if let Some(cursor) = &self.cursor_middle_image {
            let mut cursor = cursor.clone();
            cursor.pos = self.pos;
            cursor.scale = Vector2::ONE * self.settings.cursor_scale;
            
            list.push(cursor.clone());
        }
    }



    fn add_ripple(&mut self) {
        let mut group = TransformGroup::new(self.pos).alpha(0.0).border_alpha(1.0);
        let duration = 500.0;
        let time = self.time.as_millis();

        // if let Some(mut ripple) = self.ripple_image.clone() {

        //     ripple.color.a = self.ripple_color.a;
        //     ripple.pos = self.pos;

        //     // set scale
        //     const SCALE:f64 = 0.25;
        //     ripple.scale = Vector2::ONE * SCALE;

        //     let end_scale = self
        //         .ripple_radius_override
        //         .map(|r|r / ripple.size().x / 2.0)
        //         .unwrap_or(self.settings.cursor_ripple_final_scale)
        //         * SCALE;

        //     // add to transform group and make it ripple
        //     group.push(ripple);
        //     group.ripple_scale_range(0.0, duration, time, end_scale..SCALE, Some(2.0..0.0), Some(0.2));
        // } else {

            // primitive ripple, not always correct
            let radius = DEFAULT_CURSOR_SIZE / 2.0 * self.settings.cursor_scale * PRESSED_CURSOR_SCALE;
            let end_radius = self.ripple_radius_override.unwrap_or(radius * self.settings.cursor_ripple_final_scale);

            let end_scale = end_radius / radius;

            // let end_scale = self.settings.cursor_ripple_final_scale * self.ripple_radius_override.map(|r| DEFAULT_CURSOR_SIZE / r).unwrap_or(1.0);

            group.push(Circle::new(
                Vector2::ZERO,
                radius,
                Color::WHITE.alpha(0.5),
                Some(Border::new(Color::WHITE, 2.0 / end_scale))
            ));
            group.ripple(0.0, duration, time, end_scale, true, Some(0.2));
        // }


        self.ripples.push(group);
    }


    fn make_trail_group(pos: Vector2, mut trail: Image, start: f32, duration: f32, scale:Vector2, time: f32) -> TransformGroup {
        trail.scale = scale;
        let mut g = TransformGroup::new(pos).alpha(1.0).border_alpha(0.0);
        g.transforms.push(Transformation::new(
            start, 
            duration, 
            TransformType::Transparency { start: 1.0, end: 0.0 }, 
            Easing::EaseOutSine, 
            time
        ));
        g.push(trail);
        g
    }

}

impl CursorManager {
    fn add_event(event: CursorEvent) {
        // should always be okay
        if let Some(q) = CURSOR_EVENT_QUEUE.get() {
            if let Err(e) = q.try_send(event) {
                error!("cursor channel error: {e}")
            }
        }
    }

    pub fn set_pos(pos: Vector2) {
        Self::add_event(CursorEvent::SetPos(pos));
    }

    pub fn left_pressed(pressed: bool, is_gamemode: bool) {
        Self::add_event(CursorEvent::SetLeftDown(pressed, is_gamemode));
    }
    pub fn right_pressed(pressed: bool, is_gamemode: bool) {
        Self::add_event(CursorEvent::SetRightDown(pressed, is_gamemode));
    }

    pub fn set_visible(visible: bool) {
        Self::add_event(CursorEvent::SetVisible(visible));
    }
    pub fn show_system_cursor(show: bool) {
        Self::add_event(CursorEvent::ShowSystemCursor(show));
    }
    pub fn set_ripple_override(radius: Option<f32>) {
        Self::add_event(CursorEvent::OverrideRippleRadius(radius));
    }
    pub fn set_gamemode_override(enabled: bool) {
        Self::add_event(CursorEvent::GameModeOverride(enabled));
    }
}


#[derive(Copy, Clone)]
enum CursorEvent {
    SetLeftDown(bool, bool), 
    SetRightDown(bool, bool),
    SetPos(Vector2),
    ShowSystemCursor(bool),
    SetVisible(bool),
    OverrideRippleRadius(Option<f32>),

    GameModeOverride(bool),
}

