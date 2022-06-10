/**
 * Cursor Manager
 * 
 * this uses an mpsc channel because it may be inaccessible to things that need access
 * ie, a gamemode might want to hide the cursor, however it does not have direct access to the cursor field in game
 */
use crate::prelude::*;
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};

const TRAIL_CREATE_TIMER:f64 = 10.0;
const TRAIL_FADEOUT_TIMER_START:f64 = 20.0;
const TRAIL_FADEOUT_TIMER_DURATION:f64 = 100.0;

const TRAIL_CREATE_TIMER_IF_MIDDLE:f64 = 0.1;
const TRAIL_FADEOUT_TIMER_START_IF_MIDDLE:f64 = 20.0;
const TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE:f64 = 500.0;


static CURSOR_EVENT_QUEUE:OnceCell<SyncSender<CursorEvent>> = OnceCell::const_new();

pub static CURSOR_RENDER_QUEUE: OnceCell<Mutex<TripleBufferReceiver<Vec<Box<dyn Renderable>>>>> = OnceCell::const_new();


pub struct CursorManager {
    /// position of the visible cursor
    pub pos: Vector2,

    // cached settings
    pub color: Color,
    pub border_color: Color,
    cursor_border: f32,
    cursor_scale: f64,


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

    show_system_cursor: bool,


    cursor_render_sender: TripleBufferSender<Vec<Box<dyn Renderable>>>,

    settings: SettingsHelper,
}

impl CursorManager {
    pub fn init() {
        tokio::spawn(async move {
            let (cursor_render_sender, receiver) = TripleBuffer::default().split();
            CURSOR_RENDER_QUEUE.set(Mutex::new(receiver)).ok().expect("no");

            let mut s = Self::new(cursor_render_sender).await;
            let timer = Instant::now();

            loop {
                let now = Instant::now();
                
                let diff = now.duration_since(timer).as_secs_f64() * 1000.0;
                s.update(diff).await;


                let mut list = Vec::new();
                s.draw(&mut list).await;
                s.cursor_render_sender.write(list);

                // timer = now;
                // tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });
    }

    pub async fn new(cursor_render_sender: TripleBufferSender<Vec<Box<dyn Renderable>>>) -> Self {
        let mut cursor_image = SkinManager::get_texture("cursor", true).await;
        if let Some(cursor) = &mut cursor_image {
            cursor.depth = -f64::MAX;
        }

        let mut cursor_trail_image = SkinManager::get_texture("cursortrail", true).await;
        if let Some(trail) = &mut cursor_trail_image {
            trail.depth = (-f64::MAX) + 50.0;
        }


        let has_middle = SkinManager::get_texture("cursormiddle", false).await;
        let has_middle = has_middle.is_some();
        let (trail_create_timer, trail_fadeout_timer_start, trail_fadeout_timer_duration) = if has_middle {
            (TRAIL_CREATE_TIMER_IF_MIDDLE, TRAIL_FADEOUT_TIMER_START_IF_MIDDLE, TRAIL_FADEOUT_TIMER_DURATION_IF_MIDDLE)
        } else {
            (TRAIL_CREATE_TIMER, TRAIL_FADEOUT_TIMER_START, TRAIL_FADEOUT_TIMER_DURATION)
        };

        let (sender, event_receiver) = sync_channel(1000);
        if let Err(_) = CURSOR_EVENT_QUEUE.set(sender) {panic!("Cursor event queue already exists")}

        let settings = SettingsHelper::new().await;

        Self {
            pos: Vector2::zero(),
            color: Color::from_hex(&settings.cursor_color),
            border_color: Color::from_hex(&settings.cursor_border_color),
            cursor_scale: settings.cursor_scale,
            cursor_border: settings.cursor_border,
            
            skin_change_helper: SkinChangeHelper::new().await,

            trail_images: Vec::new(),
            cursor_image,
            cursor_trail_image,
            last_trail_time: 0.0,

            trail_create_timer, 
            trail_fadeout_timer_start,
            trail_fadeout_timer_duration,

            event_receiver,
            cursor_render_sender,

            left_pressed: false,
            right_pressed: false,
            visible: true,
            show_system_cursor: false,
            settings
        }
    }


    pub async fn reload_skin(&mut self) {
        self.cursor_image = SkinManager::get_texture("cursor", true).await;
        self.cursor_trail_image = SkinManager::get_texture("cursortrail", true).await;
        let has_middle = SkinManager::get_texture("cursormiddle", false).await.is_some();
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

    pub async fn update(&mut self, time: f64) {

        // check settings update 
        if self.settings.update() {
            self.color = Color::from_hex(&self.settings.cursor_color);
            self.border_color = Color::from_hex(&self.settings.cursor_border_color);
            self.cursor_scale =  self.settings.cursor_scale;
            self.cursor_border = self.settings.cursor_border;
        }


        // work through the event queue
        if let Ok(event) = self.event_receiver.try_recv() {
            match event {
                CursorEvent::SetLeftDown(down, is_gamemode) => {
                    if is_gamemode || (!is_gamemode && !self.show_system_cursor) {
                        self.left_pressed = down
                    }
                },
                CursorEvent::SetRightDown(down, is_gamemode) => {
                    if is_gamemode || (!is_gamemode && !self.show_system_cursor) {
                        self.right_pressed = down
                    }
                },
                CursorEvent::SetVisible(show) => self.visible = show,

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


        if self.skin_change_helper.check().await {
            self.reload_skin().await;
        }

        // trail stuff

        // check if we should add a new trail
        if self.cursor_trail_image.is_some() && time - self.last_trail_time >= self.trail_create_timer {
            if let Some(mut trail) = self.cursor_trail_image.clone() {
                let mut g = TransformGroup::new();
                g.transforms.push(Transformation::new(
                    self.trail_fadeout_timer_start, 
                    self.trail_fadeout_timer_duration, 
                    TransformType::Transparency {start: 1.0, end: 0.0}, 
                    TransformEasing::EaseOutSine, 
                    time
                ));
                trail.current_pos = self.pos;
                g.items.push(DrawItem::Image(trail));

                self.trail_images.push(g);
                self.last_trail_time = time;
            }

        }
    
        // update the transforms, removing any that are not visible
        self.trail_images.retain_mut(|i| {
            i.update(time);
            i.items[0].visible()
        });
    }


    pub async fn draw(&mut self, list:&mut Vec<Box<dyn Renderable>>) {
        if !self.visible {return}

        let mut radius = 5.0;
        if self.left_pressed || self.right_pressed {
            radius *= 2.0;
        }

        if self.cursor_trail_image.is_some() {
            // draw the transforms
            for i in self.trail_images.iter_mut() {
                i.draw(list);
            }
        }
        
        if let Some(cursor) = &self.cursor_image {
            let mut cursor = cursor.clone();
            cursor.current_pos = self.pos;
            cursor.current_color = self.color;
            
            if self.left_pressed || self.right_pressed {
                cursor.current_scale = Vector2::one() * 1.2;
            }
            
            list.push(Box::new(cursor.clone()));
        } else {
            list.push(Box::new(Circle::new(
                self.color,
                -f64::MAX,
                self.pos,
                radius * self.cursor_scale,
                if self.cursor_border > 0.0 {
                    Some(Border::new(
                        self.border_color,
                        self.cursor_border as f64
                    ))
                } else {None}
            )));
        }

    }
}

impl CursorManager {
    fn add_event(event: CursorEvent) {
        // should always be okay
        if let Some(q) = CURSOR_EVENT_QUEUE.get() {
            match q.try_send(event) {
                Ok(_) => {},
                Err(e) => {
                    error!("cursor channel error: {}", e)
                }
            }
            // q.send().expect("cursor channel dead?");
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
}

#[derive(Copy, Clone)]
pub enum CursorEvent {
    SetLeftDown(bool, bool), 
    SetRightDown(bool, bool),
    SetPos(Vector2, bool),
    ShowSystemCursor(bool),
    SetVisible(bool),
}

