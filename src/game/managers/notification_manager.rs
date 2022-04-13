use crate::prelude::*;


// const NOTIF_WIDTH:f64 = 300.0; // TODO: have this as the max width instead
const NOTIF_Y_OFFSET:f64 = 100.0; // window_size().y - this
const NOTIF_TEXT_SIZE:u32 = 15;
const NOTIF_DEPTH:f64 = -800_000_000.0;
// const NOTIF_TEXT_HEIGHT:f64 = 20.0;

/// how many pixels of space should there be between notifications?
const NOTIF_MARGIN:Vector2 = Vector2::new(5.0, 5.0);

/// how rounded the borders are
const NOTIF_BORDER_ROUNDING:f64 = 5.0;

/// how many pixels of padding should the notif text have?
const NOTIF_PADDING:Vector2 = Vector2::new(4.0, 4.0);

/// what background color should the notifs have?
const NOTIF_BG_COLOR:Color = Color::new(0.0, 0.0, 0.0, 0.6);


lazy_static::lazy_static! {
    pub static ref NOTIFICATION_MANAGER: Arc<Mutex<NotificationManager>> = Arc::new(Mutex::new(NotificationManager::new()));
}



pub struct NotificationManager {
    processed_notifs: Vec<ProcessedNotif>,
    pending_notifs: Vec<Notification>
}
impl NotificationManager { // static
    pub async fn add_notification(notif: Notification) {
        NOTIFICATION_MANAGER.lock().await.pending_notifs.push(notif);
    }
    pub async fn add_text_notification(text: &str, duration: f32, color: Color) {
        let notif = Notification::new(text.to_owned(), color, duration, NotificationOnClick::None);

        trace!("adding text notif");
        Self::add_notification(notif).await;
    }
    pub async fn add_error_notification<E: Into<TatakuError>>(msg:&str, error:E) {
        let error:TatakuError = error.into();
        let text = format!("{}:\n{}", msg, error);
        
        // debug!("{}", text);
        Self::add_text_notification(
            &text, 
            5_000.0, 
            Color::RED
        ).await;
    }
}
impl NotificationManager { // non-static
    fn new() -> Self { // technically static but i dont care
        Self {
            processed_notifs: Vec::new(),
            pending_notifs: Vec::new(),
        }
    }

    pub async fn update(&mut self) {
        for notif in std::mem::take(&mut self.pending_notifs) {
            // trace!("adding notif");
            let new = ProcessedNotif::new(notif);

            // move all the other notifs up
            let offset = new.size.y + NOTIF_MARGIN.y;
            for n in self.processed_notifs.iter_mut() {
                n.pos.y -= offset;
            }

            // add the new one
            self.processed_notifs.push(new);
        }

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
                        debug!("open url {}", url);
                    }
                    NotificationOnClick::Menu(menu_name) => {
                        debug!("goto menu {}", menu_name);
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
    text: Text,
    notification: Notification,
    remove: bool
}
impl ProcessedNotif {
    fn new(notification: Notification) -> Self {
        let font = get_font();
        let window_size = Settings::window_size();

        let text = Text::new(
            Color::WHITE,
            NOTIF_DEPTH - 0.1,
            Vector2::zero(), // set in draw
            NOTIF_TEXT_SIZE,
            notification.text.clone(),
            font.clone()
        );

        let size = text.measure_text() + NOTIF_PADDING * 2.0;
        let pos = window_size - Vector2::new(size.x + NOTIF_MARGIN.x, NOTIF_Y_OFFSET + size.y);

        Self {
            pos,
            size,
            time: Instant::now(),
            text,
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
            NOTIF_BG_COLOR,
            NOTIF_DEPTH + 0.1,
            self.pos,
            self.size,
            Some(Border::new(
                self.notification.color,
                1.2
            ))
        ).shape(Shape::Round(NOTIF_BORDER_ROUNDING, 10))));

        let mut text = self.text.clone();
        text.current_pos = self.pos + NOTIF_PADDING;
        list.push(Box::new(text));
    }
}



#[derive(Clone)]
#[allow(unused, dead_code)]
pub enum NotificationOnClick {
    None,
    Url(String),
    Menu(String),
}
