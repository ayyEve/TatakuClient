use crate::prelude::*;
use graphics::SkinProvider;
use tataku::{
    Color,
    Border,
    Bounds,
    Vector2,
    Alignment,
};

use engine::{
    actions,
    Notification,
};


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
    notification_image: Option<graphics::Image>,
}
impl NotificationManager {
    pub fn update(
        &mut self, 
        font_contexts: &mut ui::widget::TextLayoutContexts,
    ) {
        self.notifications.retain_mut(|n| {
            if n.text_layout.is_none() {
                n.init_font(font_contexts);
            }
            n.check_time()
        });
    }

    #[cfg(feature="graphics")]
    pub fn reload_skin(&mut self, skin_manager: &mut SkinManager) {
        self.notification_image = skin_manager.get_texture(
            Path::new("notification"),
            &graphics::TextureSource::Skin,
            graphics::SkinUsage::Game,
            true
        );
    }

    pub fn draw(
        &self,
        window_size: Vector2,
        list: &mut graphics::RenderableCollection
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
        actions: &mut actions::ActionQueue,
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
    time: tataku::Instant,
    notification: Notification,
    remove: bool,

    text_layout: Option<Arc<parley::Layout<Color>>>,
}
impl ProcessedNotif {
    fn new(notification: Notification) -> Self {
        // let text = Text::new(
        //     Vector2::ZERO,
        //     NOTIF_TEXT_SIZE,
        //     notification.text.clone(),
        //     Color::WHITE,
        //     DefaultFont::Main
        // );

        Self {
            size: Vector2::ZERO,
            time: tataku::Instant::now(),
            notification,
            remove: false,
            text_layout: None,
        }
    }

    fn init_font(
        &mut self, 
        font_contexts: &mut ui::widget::TextLayoutContexts,
    ) {
        let mut layout = font_contexts.simple_text(
            &self.notification.text, 
            &ui::style::TextStyle {
                color: Color::WHITE,
                font_size: NOTIF_TEXT_SIZE,
                ..Default::default()
            }
        );

        layout.break_all_lines(None);

        self.size = Vector2::new(
            layout.width(),
            layout.height(),
        ) + NOTIF_PADDING * 2.0;

        self.text_layout = Some(Arc::new(layout));
    }

    /// returns if the time has not expired
    fn check_time(&self) -> bool {
        if self.remove { return false }
        self.time.as_millis() < self.notification.duration
    }

    fn draw(
        &self,
        pos_offset: Vector2,
        image: Option<&graphics::Image>,
        list: &mut graphics::RenderableCollection,
    ) {
        let pos = pos_offset - (
            self.size
            + Vector2::new(NOTIF_MARGIN.x, NOTIF_Y_OFFSET)
        );

        // bg
        if let Some(mut image) = image.cloned() {
            let stretch = graphics::ImageStretch::Cover;

            let transform = graphics::Transform {
                pos,
                scale: stretch.fit_to(image.size(), self.size),
                ..graphics::Transform::identity()
            };

            image.color = self.notification.color;

            list.push(image.with_transform(transform.matrix()));
        } else {
            list.push(graphics::Rectangle::new(self.size, NOTIF_BG_COLOR)
            .border(Border::new(
                self.notification.color,
                1.2
            ))
            .shape(graphics::Shape::Round(NOTIF_BORDER_ROUNDING))
            .with_transform(tataku::Matrix::identity()
                .trans(pos)
            ));
        }

        if let Some(layout) = &self.text_layout {
            let centered = Alignment::CENTER.resolve(
                &Bounds::new(pos, self.size),
                Vector2::new(layout.width(), layout.height()),
                true, 
                true
            );

            list.push(graphics::Text::new(layout.clone())
                .with_transform(tataku::Matrix::identity()
                    .trans(centered)
            ));
        }

        // list.push(self.text.clone().centered(&bounds));
    }
}
