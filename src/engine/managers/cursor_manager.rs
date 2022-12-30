/**
 * Cursor Manager
 * 
 * this uses an mpsc channel because it may be inaccessible to things that need access
 * ie, a gamemode might want to hide the cursor, however it does not have direct access to the cursor field in game
 */
use crate::prelude::*;
use tokio::sync::mpsc::{Sender, Receiver, channel};

const TRAIL_CREATE_TIMER:f64 = 10.0;
const TRAIL_FADEOUT_TIMER_START:f64 = 20.0;
const TRAIL_FADEOUT_TIMER_DURATION:f64 = 100.0;

const TRAIL_CREATE_TIMER_IF_MIDDLE:f64 = 0.1;
const TRAIL_FADEOUT_TIMER_START_IF_MIDDLE:f64 = 20.0;
const TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE:f64 = 500.0;

const DEFAULT_CURSOR_SIZE:f64 = 10.0;
const PRESSED_CURSOR_SCALE:f64 = 1.2;


static CURSOR_EVENT_QUEUE:OnceCell<Sender<CursorEvent>> = OnceCell::const_new();

pub struct CursorManager {
    /// position of the visible cursor
    pub pos: Vector2,

    // cached settings
    pub color: Color,
    pub border_color: Color,
    ripple_color: Color,
    ripple_radius_override: Option<f64>,
    // ripple_image: Option<Image>,

    cursor_rotation: f64,

    pub cursor_image: Option<Image>,
    pub cursor_middle_image: Option<Image>,
    pub cursor_trail_image: Option<Image>,
    pub trail_images: Vec<TransformGroup>,
    last_trail_time: f64,

    trail_create_timer: f64,
    trail_fadeout_timer_start: f64,
    trail_fadeout_timer_duration: f64,

    // event_receiver: Arc<Receiver<CursorEvent>>,
    event_receiver: Receiver<CursorEvent>,

    // event vals

    /// should the cursor be visible?
    visible: bool,
    show_system_cursor: bool,

    left_pressed: bool,
    right_pressed: bool,

    settings: SettingsHelper,
    current_skin: CurrentSkinHelper,

    ripples: Vec<TransformGroup>,
    time: Instant,
}

impl CursorManager {
    pub async fn new() -> Self {
        let mut cursor_image = SkinManager::get_texture("cursor", true).await;
        if let Some(cursor) = &mut cursor_image {
            cursor.depth = -f64::MAX;
        }

        let mut cursor_trail_image = SkinManager::get_texture("cursortrail", true).await;
        if let Some(trail) = &mut cursor_trail_image {
            trail.depth = (-f64::MAX) + 50.0;
        }


        let mut cursor_middle_image = SkinManager::get_texture("cursormiddle", false).await;
        if let Some(cursor) = &mut cursor_middle_image {
            cursor.depth = -f64::MAX;
        }

        let (trail_create_timer, trail_fadeout_timer_start, trail_fadeout_timer_duration) = if cursor_middle_image.is_some() {
            (TRAIL_CREATE_TIMER_IF_MIDDLE, TRAIL_FADEOUT_TIMER_START_IF_MIDDLE, TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE)
        } else {
            (TRAIL_CREATE_TIMER, TRAIL_FADEOUT_TIMER_START, TRAIL_FADEOUT_TIMER_DURATION)
        };

        // let mut ripple_image = SkinManager::get_texture("cursor-ripple", true).await;
        // if let Some(r) = &mut ripple_image {
        //     r.depth = 1_000.0;
        // }

        let (sender, event_receiver) = channel(1000);
        CURSOR_EVENT_QUEUE.set(sender).expect("Cursor event queue already exists");

        let settings = SettingsHelper::new();
        Self {
            pos: Vector2::ZERO,
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
            settings,

            ripples: Vec::new(),
            time: Instant::now()
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
        if let Some(trail) = &mut self.cursor_trail_image {
            trail.depth = (-f64::MAX) + 50.0;
        }
        if let Some(cursor) = &mut self.cursor_image {
            cursor.depth = -f64::MAX;
        }
        
        if let Some(cursor) = &mut self.cursor_middle_image {
            cursor.depth = -f64::MAX;
        }
    }


    pub async fn update(&mut self, time: f64) {

        // check settings update 
        if self.settings.update() {
            self.color = Color::from_hex(&self.settings.cursor_color);
            self.border_color = Color::from_hex(&self.settings.cursor_border_color);
            self.ripple_color = Color::from_hex(&self.settings.cursor_ripple_color);
        }


        // work through the event queue
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                CursorEvent::OverrideRippleRadius(radius_maybe) => self.ripple_radius_override = radius_maybe,
                CursorEvent::SetVisible(show) => self.visible = show,

                CursorEvent::SetLeftDown(down, is_gamemode) => {
                    if is_gamemode || (!is_gamemode && !self.show_system_cursor) {
                        self.left_pressed = down;
                        if down && self.settings.cursor_ripples {
                            self.add_ripple()
                        }
                    }
                }
                CursorEvent::SetRightDown(down, is_gamemode) => {
                    if is_gamemode || (!is_gamemode && !self.show_system_cursor) {
                        self.right_pressed = down;
                        if down && self.settings.cursor_ripples {
                            self.add_ripple()
                        }
                    }
                }

                CursorEvent::SetPos(pos, is_gamemode) => {
                    if is_gamemode || (!is_gamemode && !self.show_system_cursor) {
                        self.pos = pos
                    }
                }

                CursorEvent::ShowSystemCursor(show) => {
                    self.show_system_cursor = show;
                    if show {
                        let _ = WINDOW_EVENT_QUEUE.get().unwrap().send(WindowEvent::ShowCursor);
                    } else {
                        let _ = WINDOW_EVENT_QUEUE.get().unwrap().send(WindowEvent::HideCursor);
                    }
                }
            }
        }


        if self.current_skin.update() {
            self.reload_skin().await;
        }

        if self.current_skin.cursor_rotate {
            self.cursor_rotation -= 0.0003 * (time - self.time.as_millis64())
        }

        // trail stuff

        // check if we should add a new trail
        if self.cursor_trail_image.is_some() && time - self.last_trail_time >= self.trail_create_timer {
            if let Some(mut trail) = self.cursor_trail_image.clone() {
                trail.scale = Vector2::ONE * self.settings.cursor_scale;
                let mut g = TransformGroup::new(self.pos, trail.depth).alpha(1.0).border_alpha(0.0);
                g.transforms.push(Transformation::new(
                    self.trail_fadeout_timer_start, 
                    self.trail_fadeout_timer_duration, 
                    TransformType::Transparency { start: 1.0, end: 0.0 }, 
                    TransformEasing::EaseOutSine, 
                    time
                ));
                g.push(trail);

                self.trail_images.push(g);
                self.last_trail_time = time;
            }

        }
    
        // update the transforms, removing any that are not visible
        self.trail_images.retain_mut(|i| {
            i.update(time);
            i.visible()
        });

        // update ripples
        let time = self.time.as_millis64();
        self.ripples.retain_mut(|ripple| {
            ripple.update(time);
            ripple.visible()
        });


    }


    pub async fn draw(&mut self, list: &mut RenderableCollection) {
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
        
        // draw ripples
        for ripple in self.ripples.iter() {
            list.push(ripple.clone())
            // ripple.draw(list)
        }

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
                self.color,
                -f64::MAX,
                self.pos,
                radius * self.settings.cursor_scale,
                if self.settings.cursor_border > 0.0 {
                    Some(Border::new(
                        self.border_color,
                        self.settings.cursor_border as f64
                    ))
                } else { None }
            ));
        }
    
        
        if let Some(cursor) = &self.cursor_middle_image {
            let mut cursor = cursor.clone();
            cursor.pos = self.pos;
            cursor.rotation = self.cursor_rotation;
            cursor.scale = Vector2::ONE * self.settings.cursor_scale;
            
            list.push(cursor.clone());
        }
    }



    fn add_ripple(&mut self) {
        let mut group = TransformGroup::new(self.pos, 1000.0).alpha(0.0).border_alpha(1.0);
        let duration = 500.0;
        let time = self.time.elapsed().as_secs_f64() * 1000.0;

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
                Color::WHITE.alpha(0.5),
                0.0,
                Vector2::ZERO,
                radius,
                Some(Border::new(Color::WHITE, 2.0 / end_scale))
            ));
            group.ripple(0.0, duration, time, end_scale, true, Some(0.2));
        // }


        self.ripples.push(group);
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

    pub fn set_pos(pos: Vector2, is_gamemode: bool) {
        Self::add_event(CursorEvent::SetPos(pos, is_gamemode));
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
    pub fn set_ripple_override(radius: Option<f64>) {
        Self::add_event(CursorEvent::OverrideRippleRadius(radius));
    }
}


#[derive(Copy, Clone)]
pub enum CursorEvent {
    SetLeftDown(bool, bool), 
    SetRightDown(bool, bool),
    SetPos(Vector2, bool),
    ShowSystemCursor(bool),
    SetVisible(bool),
    OverrideRippleRadius(Option<f64>),
}

