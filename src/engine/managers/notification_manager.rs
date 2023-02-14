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
    pub static ref NOTIFICATION_MANAGER: Arc<AsyncMutex<NotificationManager>> = Arc::new(AsyncMutex::new(NotificationManager::new()));
}



pub struct NotificationManager {
    processed_notifs: Vec<ProcessedNotif>,
    pending_notifs: Vec<Notification>,

    current_skin: CurrentSkinHelper,
    window_size: WindowSizeHelper,
    notification_image: Option<Image>,
}
impl NotificationManager { // static
    pub async fn add_notification(notif: Notification) {
        NOTIFICATION_MANAGER.lock().await.pending_notifs.push(notif);
    }
    pub async fn add_text_notification(text: impl ToString, duration: f32, color: Color) {
        let text = text.to_string();
        trace!("adding text notif '{text}'");
        let notif = Notification::new(text, color, duration, NotificationOnClick::None);

        Self::add_notification(notif).await;
    }
    pub async fn add_error_notification<E: Into<TatakuError>>(msg: impl std::fmt::Display, error:E) {
        let error:TatakuError = error.into();
        let text = format!("{msg}:\n{error}");
        error!("{text}");
        
        Self::add_text_notification(
            text, 
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
            
            current_skin: CurrentSkinHelper::new(),
            window_size: WindowSizeHelper::new(),
            notification_image: None
        }
    }

    pub async fn update(&mut self) {
        self.window_size.update();
        if self.current_skin.update() {
            self.notification_image = SkinManager::get_texture("notification", true).await;
        }

        for notif in std::mem::take(&mut self.pending_notifs) {
            // trace!("adding notif");
            let new = ProcessedNotif::new(notif);

            // add the new one
            self.processed_notifs.push(new);
        }

        self.processed_notifs.retain(|n| {
            let keep = n.check_time();
            keep
        });
    }

    pub fn draw(&mut self, list: &mut RenderableCollection) {
        let mut current_pos = self.window_size.0;

        for i in self.processed_notifs.iter().rev() {
            i.draw(current_pos, &self.notification_image, list);
            current_pos.y -= i.size.y + NOTIF_MARGIN.y;
        }
    }


    pub fn on_click(&mut self, mouse_pos: Vector2, _game: &mut Game) -> bool {
        let mut current_pos = self.window_size.0;
        
        for n in self.processed_notifs.iter_mut() {
            let pos = current_pos - Vector2::new(n.size.x + NOTIF_MARGIN.x, NOTIF_Y_OFFSET + n.size.y);
            
            if Rectangle::bounds_only(pos, n.size).contains(mouse_pos) {
                match &n.notification.onclick {
                    NotificationOnClick::None => {}
                    NotificationOnClick::Url(url) => {
                        debug!("open url {url}");
                        open_link(url.clone());
                    }
                    NotificationOnClick::Menu(menu_name) => {
                        debug!("goto menu {menu_name}");
                    }

                    NotificationOnClick::File(file_path) => {
                        let path = Path::new(file_path);
                        let folder = path.parent().unwrap().to_string_lossy().to_string();
                        let file = path.file_name().unwrap().to_string_lossy().to_string();

                        open_folder(folder, Some(file));
                    }
                    NotificationOnClick::Folder(folder) => {
                        open_folder(folder.clone(), None);
                    }
                }
                n.remove = true;
                return true;
            }

            current_pos.y -= n.size.y + NOTIF_MARGIN.y;
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
    size: Vector2,
    time: Instant,
    text: Text,
    notification: Notification,
    remove: bool
}
impl ProcessedNotif {
    fn new(notification: Notification) -> Self {
        let font = get_font();

        let text = Text::new(
            Color::WHITE,
            NOTIF_DEPTH - 0.1,
            Vector2::ZERO,
            NOTIF_TEXT_SIZE,
            notification.text.clone(),
            font.clone()
        );

        let size = text.measure_text() + NOTIF_PADDING * 2.0;
        // let pos = window_size - Vector2::new(size.x + NOTIF_MARGIN.x, NOTIF_Y_OFFSET + size.y);

        Self {
            size,
            time: Instant::now(),
            text,
            notification,
            remove: false
        }
    }

    /// returns if the time has not expired
    fn check_time(&self) -> bool {
        if self.remove { return false }
        self.time.elapsed().as_secs_f32() * 1000.0 < self.notification.duration
    }

    fn draw(&self, pos_offset: Vector2, image: &Option<Image>, list: &mut RenderableCollection) {
        let pos = pos_offset - Vector2::new(self.size.x + NOTIF_MARGIN.x, NOTIF_Y_OFFSET + self.size.y);

        // bg
        if let Some(mut image) = image.clone() {
            image.pos = pos;
            image.set_size(self.size);
            image.color = self.notification.color;

            list.push(image);
        } else {
            list.push(Rectangle::new(
                NOTIF_BG_COLOR,
                NOTIF_DEPTH + 0.1,
                pos,
                self.size,
                Some(Border::new(
                    self.notification.color,
                    1.2
                ))
            ).shape(Shape::Round(NOTIF_BORDER_ROUNDING, 10)));
        }

        let mut text = self.text.clone();
        text.pos = pos + NOTIF_PADDING;
        list.push(text);
    }
}



#[derive(Clone)]
#[allow(unused, dead_code)]
pub enum NotificationOnClick {
    None,
    Url(String),
    Menu(String),

    File(String),
    Folder(String),
}
