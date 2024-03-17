use crate::prelude::*;

pub struct NotificationsDialog {
    num: usize,
    actions: Vec<TatakuAction>,

    notifications: Vec<(Arc<Notification>, bool)>,
    // list: ScrollableArea,

    // window_size: Vector2,

    // bounds: Bounds,
    should_close: bool,
}
impl NotificationsDialog {
    pub async fn new() -> Self {
        let notifications = NOTIFICATION_MANAGER.read().await.all_notifs.clone().into_iter().map(|n|(n, false)).collect();
        // let window_size = WindowSize::get().0;
        // let bounds = Bounds::new(Vector2::new(window_size.x / 2.0, 0.0), Vector2::new(window_size.x / 2.0, window_size.y));

        // let mut list = ScrollableArea::new(bounds.pos, bounds.size, ListMode::VerticalList);

        // for notification in notifications.iter().rev().cloned() {
        //     list.add_item(Box::new(NotificationItem::new(notification, bounds.size.x, false)));
        // }

        Self {
            num: 0,
            actions: Vec::new(),

            notifications,
            // list,

            // window_size,

            // bounds,
            should_close: false,
        }
    }
}


#[async_trait]
impl Dialog for NotificationsDialog {
    fn name(&self) -> &'static str { "notifications_dialog" }
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }

    fn should_close(&self) -> bool { self.should_close }
    // fn get_bounds(&self) -> Bounds { self.bounds }
    async fn force_close(&mut self) { self.should_close = true; }

    
    async fn handle_message(&mut self, message: Message, _values: &mut ValueCollection) {
        let Some(id) = message.tag.as_number() else { return };

        if let MessageType::Click = &message.message_type {
            NotificationManager::activate_notification(id).await;
        } else {
            let mut notif_manager = NOTIFICATION_MANAGER.write().await;
            notif_manager.all_notifs.retain(|n|n.id != id);
        }

    }

    fn view(&self) -> IcedElement {
        use iced_elements::*;
        col!(
            self.notifications.iter().map(|(n, new)|NotificationItem::new(n.clone(), *new).into_element()).collect(),
            height = Fill
        )

    }


    async fn update(&mut self, _values: &mut ValueCollection) -> Vec<TatakuAction> {
        if let Ok(notifs) = NOTIFICATION_MANAGER.try_read() {
            //TODO: only add new notifs
            self.notifications = notifs.all_notifs.clone().into_iter().map(|n|(n, false)).collect();
            // let to_add = notifs.all_notifs.iter().filter(|n|!self.notifications.contains_key(&n.id)).collect::<Vec<_>>();

            // if !to_add.is_empty() {
            //     info!("adding new notifs {to_add:?}");
            //     // let width = self.bounds.size.x;
            //     // let last_items = std::mem::take(&mut self.list.items);
            //     // self.list.clear();

            //     for i in to_add.into_iter().rev() {
            //         self.notifications.insert(i.id, i.clone());
            //         // self.list.add_item(Box::new(NotificationItem::new(i.clone(), width, true)));
            //     }

            //     // for mut i in last_items {
            //     //     i.set_pos(Vector2::ZERO);
            //     //     self.list.add_item(i);
            //     // }
            // }
        
        }

        // if self.list.items.iter().find(|i|i.size() == Vector2::ZERO).is_some() {
        //     // purge non-zero sized items
        //     let items = std::mem::take(&mut self.list.items);
        //     self.list.clear();
            
        //     for mut i in items.into_iter().filter(|i|i.size() != Vector2::ZERO) {
        //         i.set_pos(Vector2::ZERO);
        //         self.list.add_item(i);
        //     }
        // }

        self.actions.take()
    }
}


const DELETE_DURATION: f32 = 300.0;
const FONT_SIZE:f32 = 32.0;
const PADDING:f32 = 5.0;
const SQUARE:f32 = FONT_SIZE / 2.0;

// #[derive(ScrollableGettersSetters)]
struct NotificationItem {
    content: IcedElement,
    // pos: Vector2,
    // size: Vector2,
    // tag: String,
    // // ui_scale: Vector2,
    // hover: bool,

    is_new: bool,

    notification: Arc<Notification>,
    
    // added_timer: Option<Instant>,
    delete_hover: bool,
    // delete_time: Option<Instant>,
}
impl NotificationItem {
    fn new(notification: Arc<Notification>, is_new: bool) -> Self {
        use iced_elements::*;

        let content = row!(
            // text
            Text::new(notification.text.clone()).width(Fill).size(FONT_SIZE),

            // delete button
            Text::new(FontAwesome::WindowCloseOutline.to_string()).width(Shrink).size(SQUARE).color(Color::WHITE)
            
            ;
            width = Fill,
            padding = PADDING
        );


        Self {
            content,
            
            // pos: Vector2::ZERO,
            // size: Vector2::new(width, notification.text.lines().count() as f32 * FONT_SIZE) + Vector2::ONE * PADDING * 2.0,
            // tag: notification.id.to_string(),
            // hover: false,
            // ui_scale: Vector2::ONE,

            // added_timer: is_new.then(Instant::now),

            notification, 
            delete_hover: false,
            is_new,
            // delete_time: None,
        }
    }
}
// impl ScrollableItem for NotificationItem {
//     fn update(&mut self) {
//         if let Some(time) = self.delete_time {
//             if time.as_millis() >= DELETE_DURATION {
//                 self.size = Vector2::ZERO;
//             }
//         }
//         if let Some(time) = self.added_timer {
//             if time.as_millis() >= DELETE_DURATION {
//                 self.added_timer = None;
//             }
//         }
//     }

//     fn draw(&mut self, pos_offset: Vector2, list: &mut RenderableCollection) {
//         let mut pos = self.pos + pos_offset;
//         let mut alpha = 1.0;

//         // when deleted, have it swipe to the left
//         if let Some(time) = &self.delete_time {
//             let amount = time.as_millis() / DELETE_DURATION;
//             pos.x += self.size.x * amount;
//             alpha = 1.0 - amount;
//         }
//         if let Some(time) = &self.added_timer {
//             let amount = time.as_millis() / DELETE_DURATION;
//             pos.x += self.size.x - self.size.x * amount;
//             alpha = amount;
//         }

//         // draw bounding rectangle
//         let color = self.notification.color;
//         list.push(Rectangle::new(pos, self.size, color.alpha(0.5*alpha), Some(Border::new(color.alpha(alpha), 2.0))).shape(Shape::Round(PADDING)));
        
//         pos += Vector2::ONE * PADDING;

//         // draw delete button
//         let (c, c_color) = if self.delete_hover {(FontAwesome::WindowClose, Color::RED)} else {(FontAwesome::WindowCloseOutline, Color::WHITE)};
//         list.push(crate::prelude::Text::new(pos + Vector2::with_x(self.size.x - SQUARE * 2.0), SQUARE, c, c_color, Font::FontAwesome));

//         // text
//         // let bounds = Bounds::new(pos + Vector2::ONE * PADDING, self.size - Vector2::ONE * PADDING * 2.0);
//         list.push(Text::new(pos - Vector2::with_y(PADDING), FONT_SIZE, &self.notification.text, Color::WHITE.alpha(alpha), Font::Main));
//     }
// }

impl iced::advanced::Widget<Message, IcedRenderer> for NotificationItem {
    fn width(&self) -> iced::Length { iced::Length::Fill }
    fn height(&self) -> iced::Length { iced::Length::Shrink }

    fn state(&self) -> iced_runtime::core::widget::tree::State {
        iced_runtime::core::widget::tree::State::new(NotificationItemState::new(self.is_new))
    }

    fn layout(
        &self,
        renderer: &IcedRenderer,
        limits: &iced_runtime::core::layout::Limits,
    ) -> iced_runtime::core::layout::Node {
        self.content.as_widget().layout(renderer, &limits.width(self.width()).height(self.height()))
    }

    fn draw(
        &self,
        state: &iced_runtime::core::widget::Tree,
        renderer: &mut IcedRenderer,
        theme: &iced::Theme,
        style: &iced_runtime::core::renderer::Style,
        layout: iced_runtime::core::Layout<'_>,
        cursor: iced_runtime::core::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        
    }
}

impl From<NotificationItem> for IcedElement {
    fn from(value: NotificationItem) -> Self {
        Self::new(value)
    }
}



struct NotificationItemState {
    added_timer: Option<Instant>,
    delete_time: Option<Instant>,
}
impl NotificationItemState {
    fn new(is_new: bool) -> Self {
        Self {
            added_timer: is_new.then(Instant::now),
            delete_time: None,
        }
    }
}
