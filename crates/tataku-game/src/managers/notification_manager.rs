use crate::prelude::*;

const NOTIF_Y_OFFSET:f32 = 100.0; // window_size().y - this
const NOTIF_TEXT_SIZE:f32 = 25.0;

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
    pub fn update(&mut self) {
        self.notifications.retain(|n| n.check_time());
    }

    #[cfg(feature="graphics")]
    pub fn reload_skin(&mut self, skin_manager: &mut SkinManager) {
        self.notification_image = skin_manager.get_texture(
            "notification", 
            &TextureSource::Skin, 
            SkinUsage::Game, 
            true
        );
    }

    pub fn draw(
        &self, 
        window_size: Vector2, 
        list: &mut RenderableCollection
    ) {
        let mut current_pos = window_size;

        for i in self.notifications.iter().rev() {
            i.draw(current_pos, self.notification_image.as_ref(), list);
            current_pos.y -= i.size.y + NOTIF_MARGIN.y;
        }
    }


    pub fn on_click(
        &mut self, 
        window_size: Vector2, 
        mouse_pos: Vector2, 
        actions: &mut ActionQueue,
    ) -> bool {
        let mut current_pos = window_size;
        
        for n in self.notifications.iter_mut() {
            let pos = current_pos - Vector2::new(
                n.size.x + NOTIF_MARGIN.x, 
                NOTIF_Y_OFFSET + n.size.y
            );
            
            if Bounds::new(pos, n.size).contains(mouse_pos) {
                n.notification.onclick.do_action(actions);
                n.remove = true;
                return true;
            }

            current_pos.y -= n.size.y + NOTIF_MARGIN.y;
        }

        false
    }

    pub fn add_notification(&mut self, notif: Notification) {
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
        
        Self {
            size: text.measure_text() + NOTIF_PADDING * 2.0,
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

    fn draw(
        &self, 
        pos_offset: Vector2, 
        image: Option<&Image>, 
        list: &mut RenderableCollection
    ) {
        let pos = pos_offset - (
            self.size 
            + Vector2::new(NOTIF_MARGIN.x, NOTIF_Y_OFFSET)
        );

        // bg
        let bounds = Bounds::new(pos, self.size);
        if let Some(mut image) = image.cloned() {
            image.pos = pos;
            image.set_size(self.size);
            image.color = self.notification.color;

            list.push(image);
        } else {
            list.push(Rectangle::new_bounds(
                bounds,
                NOTIF_BG_COLOR,
                Some(Border::new(
                    self.notification.color,
                    1.2
                ))
            ).shape(Shape::Round(NOTIF_BORDER_ROUNDING)));
        }

        list.push(self.text.clone().centered(&bounds));
    }
}
