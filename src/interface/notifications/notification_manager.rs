use crate::prelude::*;


// const NOTIF_WIDTH:f64 = 300.0; // TODO: have this as the max width instead
const NOTIF_Y_OFFSET:f32 = 100.0; // window_size().y - this
const NOTIF_TEXT_SIZE:f32 = 15.0;
// const NOTIF_TEXT_HEIGHT:f64 = 20.0;

/// how many pixels of space should there be between notifications?
const NOTIF_MARGIN:Vector2 = Vector2::new(5.0, 5.0);

/// how rounded the borders are
const NOTIF_BORDER_ROUNDING:f32 = 5.0;

/// how many pixels of padding should the notif text have?
const NOTIF_PADDING:Vector2 = Vector2::new(4.0, 4.0);

/// what background color should the notifs have?
const NOTIF_BG_COLOR:Color = Color::new(0.0, 0.0, 0.0, 0.6);


lazy_static::lazy_static! {
    pub static ref NOTIFICATION_MANAGER: Arc<AsyncRwLock<NotificationManager>> = Arc::new(AsyncRwLock::new(NotificationManager::new()));
}


pub struct NotificationManager {
    // every notif thats been added
    pub all_notifs: Vec<Arc<Notification>>,

    processed_notifs: Vec<ProcessedNotif>,
    pending_notifs: Vec<Arc<Notification>>,
    
    current_skin: CurrentSkinHelper,
    window_size: WindowSizeHelper,
    notification_image: Option<Image>,

    queued_actions: Vec<NotificationOnClick>
}
impl NotificationManager {
    fn new() -> Self { // technically static but i dont care
        Self {
            all_notifs: Vec::new(),
            processed_notifs: Vec::new(),
            pending_notifs: Vec::new(),
            
            current_skin: CurrentSkinHelper::new(),
            window_size: WindowSizeHelper::new(),
            notification_image: None,
            queued_actions: Vec::new(),
        }
    }

    pub async fn add_notification(notif: Notification) {
        let mut manager = NOTIFICATION_MANAGER.write().await;
        let notif = Arc::new(notif);
        manager.pending_notifs.push(notif.clone());
        manager.all_notifs.push(notif);
    }
    pub async fn add_text_notification(text: impl ToString, duration: f32, color: Color) {
        let text = text.to_string();
        trace!("adding text notif '{text}'");
        let notif = Notification::new(text, color, duration, NotificationOnClick::None);

        Self::add_notification(notif).await;
    }
    pub async fn add_error_notification<E: Into<TatakuError>>(msg: impl std::fmt::Display, error:E) {
        let error:TatakuError = error.into();
        let text = format!("{msg}: {error}");
        error!("{text}");
        
        Self::add_text_notification(
            text, 
            5_000.0, 
            Color::RED
        ).await;
    }


    pub async fn update(&mut self, game: &mut Game) {
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

        self.processed_notifs.retain(|n| n.check_time());

        for i in self.queued_actions.take() {
            i.do_action(game).await;
        }
    }

    pub fn draw(&self, list: &mut RenderableCollection) {
        let mut current_pos = self.window_size.0;

        for i in self.processed_notifs.iter().rev() {
            i.draw(current_pos, &self.notification_image, list);
            current_pos.y -= i.size.y + NOTIF_MARGIN.y;
        }
    }


    pub async fn on_click(&mut self, mouse_pos: Vector2, game: &mut Game) -> bool {
        let mut current_pos = self.window_size.0;
        
        for n in self.processed_notifs.iter_mut() {
            let pos = current_pos - Vector2::new(n.size.x + NOTIF_MARGIN.x, NOTIF_Y_OFFSET + n.size.y);
            
            if Bounds::new(pos, n.size).contains(mouse_pos) {
                n.notification.onclick.do_action(game).await;
                n.remove = true;
                return true;
            }

            current_pos.y -= n.size.y + NOTIF_MARGIN.y;
        }

        false
    }


    pub async fn activate_notification(notif_id: usize) {
        let mut manager = NOTIFICATION_MANAGER.write().await;

        if let Some((n, _)) = manager.all_notifs.iter().enumerate().find(|(_, n)|n.id == notif_id) {
            let notif = manager.all_notifs.remove(n);
            manager.queued_actions.push(notif.onclick.clone());
        }
    }
}


#[derive(Clone)]
struct ProcessedNotif {
    size: Vector2,
    time: Instant,
    text: Text,
    notification: Arc<Notification>,
    remove: bool
}
impl ProcessedNotif {
    fn new(notification: Arc<Notification>) -> Self {
        let text = Text::new(
            Vector2::ZERO,
            NOTIF_TEXT_SIZE,
            notification.text.clone(),
            Color::WHITE,
            Font::Main
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
                pos,
                self.size,
                NOTIF_BG_COLOR,
                Some(Border::new(
                    self.notification.color,
                    1.2
                ))
            ).shape(Shape::Round(NOTIF_BORDER_ROUNDING)));
        }

        let mut text = self.text.clone();
        text.pos = pos + NOTIF_PADDING;
        list.push(text);
    }
}
