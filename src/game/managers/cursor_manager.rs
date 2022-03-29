/**
 * Cursor Manager
 * 
 * this uses an mpsc channel because it may be inaccessible to things that need access
 * ie, a gamemode might want to hide the cursor, however it does not have direct access to the cursor field in game
 */
use crate::prelude::*;
use std::sync::mpsc::{SyncSender, Receiver, channel, sync_channel};

const TRAIL_CREATE_TIMER:f64 = 10.0;
const TRAIL_FADEOUT_TIMER_START:f64 = 20.0;
const TRAIL_FADEOUT_TIMER_DURATION:f64 = 100.0;

const TRAIL_CREATE_TIMER_IF_MIDDLE:f64 = 0.1;
const TRAIL_FADEOUT_TIMER_START_IF_MIDDLE:f64 = 20.0;
const TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE:f64 = 500.0;


static CURSOR_EVENT_QUEUE:OnceCell<SyncSender<CursorEvent>> = OnceCell::const_new();



pub struct CursorManager {
    /// position of the visible cursor
    pub pos: Vector2,


    pub color: Color,
    pub border_color: Color,


    pub cursor_image: Option<Image>,
    pub cursor_trail_image: Option<Image>,
    pub trail_images: Vec<TransformGroup>,
    last_trail_time: f64,
    skin_change_helper: SkinChangeHelper,

    trail_create_timer: f64,
    trail_fadeout_timer_start: f64,
    trail_fadeout_timer_duration: f64,

    event_receiver: Receiver<CursorEvent>,

    // event vals

    /// should the cursor be visible?
    visible: bool,

    left_pressed: bool,
    right_pressed: bool,

    show_system_cursor: bool
}

impl CursorManager {
    pub fn new() -> Self {
        let mut cursor_image = SKIN_MANAGER.write().get_texture("cursor", true);
        if let Some(cursor) = &mut cursor_image {
            cursor.depth = -f64::MAX;
        }

        let mut cursor_trail_image = SKIN_MANAGER.write().get_texture("cursortrail", true);
        if let Some(trail) = &mut cursor_trail_image {
            trail.depth = (-f64::MAX) + 50.0;
        }

        let settings = get_settings!();

        let has_middle = SKIN_MANAGER.write().get_texture("cursormiddle", false).is_some();
        let (trail_create_timer, trail_fadeout_timer_start, trail_fadeout_timer_duration) = if has_middle {
            (TRAIL_CREATE_TIMER_IF_MIDDLE, TRAIL_FADEOUT_TIMER_START_IF_MIDDLE, TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE)
        } else {
            (TRAIL_CREATE_TIMER, TRAIL_FADEOUT_TIMER_START, TRAIL_FADEOUT_TIMER_DURATION)
        };

        let (sender, event_receiver) = sync_channel(1000);
        if let Ok(_) = CURSOR_EVENT_QUEUE.set(sender) {
            info!("cursor event queue set");
        } else {
            error!("hjkjugtfgu")
        }

        Self {
            pos: Vector2::zero(),
            color: Color::from_hex(&settings.cursor_color),
            border_color: Color::from_hex(&settings.cursor_border_color),
            
            skin_change_helper: SkinChangeHelper::new(),


            trail_images: Vec::new(),
            cursor_image,
            cursor_trail_image,
            last_trail_time: 0.0,

            trail_create_timer, 
            trail_fadeout_timer_start,
            trail_fadeout_timer_duration,

            event_receiver,

            left_pressed: false,
            right_pressed: false,
            visible: true,
            show_system_cursor: false,
        }
    }


    pub fn reload_skin(&mut self) {
        // TODO: this
        self.cursor_image = SKIN_MANAGER.write().get_texture("cursor", true);
        self.cursor_trail_image = SKIN_MANAGER.write().get_texture("cursortrail", true);
        let has_middle = SKIN_MANAGER.write().get_texture("cursormiddle", false).is_some();
        let (trail_create_timer, trail_fadeout_timer_start, trail_fadeout_timer_duration) = if has_middle {
            (TRAIL_CREATE_TIMER_IF_MIDDLE, TRAIL_FADEOUT_TIMER_START_IF_MIDDLE, TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE)
        } else {
            (TRAIL_CREATE_TIMER, TRAIL_FADEOUT_TIMER_START, TRAIL_FADEOUT_TIMER_DURATION)
        };

        self.trail_create_timer = trail_create_timer;
        self.trail_fadeout_timer_start = trail_fadeout_timer_start;
        self.trail_fadeout_timer_duration = trail_fadeout_timer_duration;

        if let Some(trail) = &mut self.cursor_trail_image {
            trail.depth = (-f64::MAX) + 50.0;
        }
        if let Some(cursor) = &mut self.cursor_image {
            cursor.depth = -f64::MAX;
        }
    }

    pub fn update(&mut self, game_time: f64, window: &mut glfw_window::GlfwWindow) {

        // work through the event queue
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                CursorEvent::SetLeftDown(down) => self.left_pressed = down,
                CursorEvent::SetRightDown(down) => self.right_pressed = down,
                CursorEvent::SetVisible(show) => self.visible = show,

                CursorEvent::SetPos(pos, is_game) => {
                    if !is_game || (is_game && !self.show_system_cursor) {
                        self.pos = pos
                    }
                }
                CursorEvent::ShowSystemCursor(show) => {
                    use glfw::CursorMode::{Normal, Hidden};
                    window.window.set_cursor_mode(if show {Normal} else {Hidden});

                    self.show_system_cursor = show;
                }
            }
        }


        if self.skin_change_helper.check() {
            self.reload_skin();
        }

        // trail stuff

        // check if we should add a new trail
        if self.cursor_trail_image.is_some() && game_time - self.last_trail_time >= self.trail_create_timer {
            if let Some(mut trail) = self.cursor_trail_image.clone() {
                let mut g = TransformGroup::new();
                g.transforms.push(Transformation::new(
                    self.trail_fadeout_timer_start, 
                    self.trail_fadeout_timer_duration, 
                    TransformType::Transparency {start: 1.0, end: 0.0}, 
                    TransformEasing::EaseOutSine, 
                    game_time
                ));
                trail.current_pos = self.pos;
                g.items.push(DrawItem::Image(trail));

                self.trail_images.push(g);
                self.last_trail_time = game_time;
            }

        }
    
        // update the transforms, removing any that are not visible
        self.trail_images.retain_mut(|i| {
            i.update(game_time);
            i.items[0].visible()
        });
    }


    pub fn draw(&mut self, list:&mut Vec<Box<dyn Renderable>>) {
        if !self.visible {return}

        let mut radius = 5.0;
        if self.left_pressed || self.right_pressed {
            radius *= 2.0;
        }

        let settings = get_settings!();

        if self.cursor_trail_image.is_some() {
            // draw the transforms
            for i in self.trail_images.iter_mut() {
                i.draw(list);
            }
        }
        
        if let Some(cursor) = &mut self.cursor_image {
            cursor.current_pos = self.pos;
            cursor.current_color = self.color;
            list.push(Box::new(cursor.clone()));
        } else {
            list.push(Box::new(Circle::new(
                self.color,
                -f64::MAX,
                self.pos,
                radius * settings.cursor_scale,
                if settings.cursor_border > 0.0 {
                    Some(Border::new(
                        self.border_color,
                        settings.cursor_border as f64
                    ))
                } else {None}
            )));
        }

    }
}

impl CursorManager {
    fn add_event(event: CursorEvent) {
        if let Some(q) = CURSOR_EVENT_QUEUE.get() {
            q.send(event).expect("cursor channel dead?");
        } else {
            error!("hgbyvtfbgymjiinhbgvtfrbgy")
        }
    }

    pub fn set_pos(pos: Vector2, is_game: bool) {
        Self::add_event(CursorEvent::SetPos(pos, is_game));
    }

    pub fn left_pressed(pressed: bool) {
        Self::add_event(CursorEvent::SetLeftDown(pressed));
    }
    pub fn right_pressed(pressed: bool) {
        Self::add_event(CursorEvent::SetRightDown(pressed));
    }

    pub fn set_visible(visible: bool) {
        Self::add_event(CursorEvent::SetVisible(visible));
    }
    pub fn show_system_cursor(show: bool) {
        Self::add_event(CursorEvent::ShowSystemCursor(show));
    }
}

#[derive(Copy, Clone)]
pub enum CursorEvent {
    SetLeftDown(bool), 
    SetRightDown(bool),
    SetPos(Vector2, bool),
    ShowSystemCursor(bool),
    SetVisible(bool),
}

