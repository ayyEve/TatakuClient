use std::time::Instant;

use crate::errors::TaikoError;
use crate::sync::*;
use crate::window_size;
use crate::game::{Game, get_font};
use crate::render::{Border, Color, Rectangle, Renderable, Text, Vector2};


const NOTIF_WIDTH:f64 = 200.0;
const NOTIF_Y_OFFSET:f64 = 100.0; // window_size().y - this
const NOTIF_TEXT_SIZE:u32 = 15;
const NOTIF_DEPTH:f64 = -8_000.0;
const NOTIF_TEXT_HEIGHT:f64 = 20.0;
const NOTIF_Y_MARGIN:f64 = 5.0;


lazy_static::lazy_static! {
    pub static ref NOTIFICATION_MANAGER: Arc<Mutex<NotificationManager>> = Arc::new(Mutex::new(NotificationManager::new()));
}



pub struct NotificationManager {
    processed_notifs: Vec<ProcessedNotif>
}
impl NotificationManager { // static
    pub fn add_notification(notif: Notification) {
        let new = ProcessedNotif::new(notif);
        let mut locked = NOTIFICATION_MANAGER.lock();

        // move all the other notifs up
        let offset = new.size.y + NOTIF_Y_MARGIN;
        for n in locked.processed_notifs.iter_mut() {
            n.pos.y -= offset;
        }

        // add the new one
        locked.processed_notifs.push(new);
    }
    pub fn add_text_notification(text: &str, duration: f32, color: Color) {
        let notif = Notification::new(text.to_owned(), color, duration, NotificationOnClick::None);
        Self::add_notification(notif);
    }
    pub fn add_error_notification(msg:&str, error:TaikoError) {
        let text = format!("{}:\n{}", msg, error);
        
        println!("{}", text);
        Self::add_text_notification(
            &text, 
            5_000.0, 
            Color::RED
        );
    }
}
impl NotificationManager { // non-static
    fn new() -> Self { // technically static but i dont care
        Self {
            processed_notifs: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        // let mut removed_height = 0.0;
        self.processed_notifs.retain(|n| {
            let keep = n.check_time();
            // if !keep {removed_height += n.size.y + NOTIF_Y_MARGIN}
            keep
        });

        // if removed_height > 0.0 {
        //     for i in self.processed_notifs.iter_mut() {
        //         i.pos.y += removed_height;
        //     }
        // }
    }

    pub fn draw(&mut self, list: &mut Vec<Box<dyn Renderable>>) {
        for i in self.processed_notifs.iter() {
            i.draw(list);
        }
    }


    pub fn on_click(&mut self, mouse_pos: Vector2, _game: &mut Game) -> bool {
        for n in self.processed_notifs.iter_mut() {
            if Rectangle::bounds_only(n.pos, n.size).contains(mouse_pos) {
                match &n.notification.onclick {
                    NotificationOnClick::None => {}
                    NotificationOnClick::Url(url) => {
                        println!("open url {}", url);
                    }
                    NotificationOnClick::Menu(menu_name) => {
                        println!("goto menu {}", menu_name);
                    }
                }
                n.remove = true;
                return true;
            }
        }

        false
    }
}


#[derive(Clone)]
pub struct Notification {
    /// text to display
    pub text: String,
    /// color of the bounding box
    pub color: Color,
    /// how long this message should last, in ms
    pub duration: f32,
    /// what shold happen on click?
    pub onclick: NotificationOnClick
}
impl Notification {
    pub fn new(text: String, color: Color, duration: f32, onclick: NotificationOnClick) -> Self {
        Self {
            text,
            color,
            duration,
            onclick
        }
    }
}

#[derive(Clone)]
struct ProcessedNotif {
    pos: Vector2,
    size: Vector2,
    time: Instant,
    lines: Vec<Text>,
    notification: Notification,
    remove: bool
}
impl ProcessedNotif {
    fn new(notification: Notification) -> Self {
        let font = get_font("main");

        let mut lines = Vec::new();
        let split = notification.text.split('\n').collect::<Vec<&str>>();
        for i in 0..split.len() {
            let i = (split.len() - 1) - i; // reverse
            lines.push(Text::new(
                Color::WHITE,
                NOTIF_DEPTH - 0.1,
                Vector2::new(2.0, 15.0 + NOTIF_TEXT_HEIGHT * i as f64 ),
                NOTIF_TEXT_SIZE,
                split[i].to_owned(),
                font.clone()
            ))
        }

        let size = Vector2::new(NOTIF_WIDTH, NOTIF_TEXT_HEIGHT * lines.len() as f64);
        let pos = Vector2::new(window_size().x - NOTIF_WIDTH, window_size().y - (NOTIF_Y_OFFSET + size.y));

        Self {
            pos,
            size,
            time: Instant::now(),
            lines,
            notification,
            remove: false
        }
    }

    /// returns if the time has not expired
    fn check_time(&self) -> bool {
        if self.remove {return false}
        self.time.elapsed().as_secs_f32() * 1000.0 < self.notification.duration
    }

    fn draw(&self, list: &mut Vec<Box<dyn Renderable>>) {
        // bg
        list.push(Box::new(Rectangle::new(
            Color::new(0.0, 0.0, 0.0, 0.6),
            NOTIF_DEPTH + 0.1,
            self.pos,
            self.size,
            Some(Border::new(
                self.notification.color,
                1.2
            ))
        )));

        for text in self.lines.iter() {
            let mut text = text.clone();
            text.pos = text.pos + self.pos;
            list.push(Box::new(text));
        }
    }
}



#[derive(Clone)]
#[allow(unused, dead_code)]
pub enum NotificationOnClick {
    None,
    Url(String),
    Menu(String),
    
}
