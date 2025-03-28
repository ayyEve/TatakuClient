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

#[derive(Default)]
pub struct NotificationManager {
    notifications: Vec<ProcessedNotif>,
    notification_image: Option<Image>,
}
impl NotificationManager {
    pub async fn update(&mut self) {
        self.notifications.retain(|n| n.check_time());
    }

    #[cfg(feature="graphics")]
    pub async fn reload_skin(&mut self, skin_manager: &mut SkinManager) {
        self.notification_image = skin_manager.get_texture("notification", &TextureSource::Skin, SkinUsage::Game, true).await;
    }

    pub fn draw(&self, window_size: Vector2, list: &mut RenderableCollection) {
        let mut current_pos = window_size;

        for i in self.notifications.iter().rev() {
            i.draw(current_pos, &self.notification_image, list);
            current_pos.y -= i.size.y + NOTIF_MARGIN.y;
        }
    }


    pub async fn on_click(
        &mut self, 
        window_size: Vector2, 
        mouse_pos: Vector2, 
        actions: &mut ActionQueue
    ) -> bool {
        let mut current_pos = window_size;
        
        for n in self.notifications.iter_mut() {
            let pos = current_pos - Vector2::new(n.size.x + NOTIF_MARGIN.x, NOTIF_Y_OFFSET + n.size.y);
            
            if Bounds::new(pos, n.size).contains(mouse_pos) {
                n.notification.onclick.do_action(actions).await;
                n.remove = true;
                return true;
            }

            current_pos.y -= n.size.y + NOTIF_MARGIN.y;
        }

        false
    }

    pub fn add_notification(&mut self, notif: Notification) {
        // trace!("adding notif");
        self.notifications.push(ProcessedNotif::new(notif));
    }
}


#[derive(Clone)]
struct ProcessedNotif {
    size: Vector2,
    time: TatakuInstant,
    text: Text,
    notification: Notification,
    remove: bool
}
impl ProcessedNotif {
    fn new(notification: Notification) -> Self {
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
            time: TatakuInstant::now(),
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
