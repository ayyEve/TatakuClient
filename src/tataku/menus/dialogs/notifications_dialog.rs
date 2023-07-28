use crate::prelude::*;

pub struct NotificationsDialog {
    notifications: HashMap<usize, Arc<Notification>>,
    list: ScrollableArea,

    window_size: Vector2,

    bounds: Bounds,
    should_close: bool,
}
impl NotificationsDialog {
    pub async fn new() -> Self {
        let notifications = NOTIFICATION_MANAGER.read().await.all_notifs.clone();
        let window_size = WindowSize::get().0;
        let bounds = Bounds::new(Vector2::new(window_size.x / 2.0, 0.0), Vector2::new(window_size.x / 2.0, window_size.y));

        let mut list = ScrollableArea::new(bounds.pos, bounds.size, ListMode::VerticalList);

        for notification in notifications.iter().rev().cloned() {
            list.add_item(Box::new(NotificationItem::new(notification, bounds.size.x, false)));
        }

        Self {
            notifications: notifications.into_iter().map(|n|(n.id, n)).collect(),
            list,

            window_size,

            bounds,
            should_close: false,
        }
    }
}


#[async_trait]
impl Dialog<Game> for NotificationsDialog {
    fn name(&self) -> &'static str { "notifications_dialog" }
    fn should_close(&self) -> bool { self.should_close }
    fn get_bounds(&self) -> Bounds { self.bounds }
    async fn force_close(&mut self) { self.should_close = true; }

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size.0;
        self.bounds = Bounds::new(Vector2::new(window_size.x / 2.0, 0.0), Vector2::new(window_size.x / 2.0, window_size.y));
        self.list.set_pos(self.bounds.pos);
        self.list.set_size(self.bounds.size);

        let width = self.bounds.size.x;
        for item in self.list.items.iter_mut() {
            let size = item.size();
            if size != Vector2::ZERO { item.set_size(Vector2::new(width, size.y))}
        }
    }

    // input handlers
    async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) { self.list.on_mouse_move(pos) }
    async fn on_mouse_scroll(&mut self, delta:f32, _g:&mut Game) -> bool { self.list.on_scroll(delta) }
    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, game:&mut Game) -> bool {
        if let Some(tag) = self.list.on_click_tagged(pos, button, *mods) {
            // notif close button was clicked
            if tag.starts_with("remove_") {
                let id = tag.trim_start_matches("remove_").parse::<usize>().unwrap();
                let mut notif_manager = NOTIFICATION_MANAGER.write().await;
                notif_manager.all_notifs.retain(|n|n.id != id);
            } else {
                let id = tag.parse::<usize>().unwrap();
                
                let mut notif_manager = NOTIFICATION_MANAGER.write().await;
                notif_manager.all_notifs.retain(|n|n.id != id);

                if let Some(notif) = self.notifications.values().find(|n| n.id == id) {
                    notif.onclick.do_action(game).await;
                }
            }
        }

        self.bounds.contains(pos)
    }
    async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.list.on_click_release(pos, button);
        self.bounds.contains(pos)
    }

    async fn update(&mut self, _g:&mut Game) {
        if let Ok(notifs) = NOTIFICATION_MANAGER.try_read() {
            let to_add = notifs.all_notifs.iter().filter(|n|!self.notifications.contains_key(&n.id)).collect::<Vec<_>>();

            if !to_add.is_empty() {
                info!("adding new notifs {to_add:?}");
                let width = self.bounds.size.x;
                let last_items = std::mem::take(&mut self.list.items);
                self.list.clear();

                for i in to_add.into_iter().rev() {
                    self.notifications.insert(i.id, i.clone());
                    self.list.add_item(Box::new(NotificationItem::new(i.clone(), width, true)));
                }

                for mut i in last_items {
                    i.set_pos(Vector2::ZERO);
                    self.list.add_item(i);
                }
            }
        }

        if self.list.items.iter().find(|i|i.size() == Vector2::ZERO).is_some() {
            // purge non-zero sized items
            let items = std::mem::take(&mut self.list.items);
            self.list.clear();
            
            for mut i in items.into_iter().filter(|i|i.size() != Vector2::ZERO) {
                i.set_pos(Vector2::ZERO);
                self.list.add_item(i);
            }
        }

        self.list.update();
    }
    async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
        // draw vis rect
        list.push(visibility_bg(self.bounds.pos + offset, self.bounds.size));
        // draw list
        self.list.draw(offset, list);
    }
}


const DELETE_DURATION: f32 = 300.0;
const FONT_SIZE:f32 = 32.0;
const PADDING:f32 = 5.0;
const SQUARE:f32 = FONT_SIZE / 2.0;


#[derive(ScrollableGettersSetters)]
struct NotificationItem {
    pos: Vector2,
    size: Vector2,
    tag: String,
    // ui_scale: Vector2,
    hover: bool,

    notification: Arc<Notification>,
    
    added_timer: Option<Instant>,
    delete_hover: bool,
    delete_time: Option<Instant>,
}
impl NotificationItem {
    fn new(notification: Arc<Notification>, width: f32, is_new: bool) -> Self {
        Self {
            pos: Vector2::ZERO,
            size: Vector2::new(width, notification.text.lines().count() as f32 * FONT_SIZE) + Vector2::ONE * PADDING * 2.0,
            tag: notification.id.to_string(),
            hover: false,
            // ui_scale: Vector2::ONE,

            added_timer: is_new.then(Instant::now),

            notification, 
            delete_hover: false,
            delete_time: None,
        }
    }
}
impl ScrollableItem for NotificationItem {
    fn on_mouse_move(&mut self, p:Vector2) {
        self.check_hover(p);

        if self.hover {
            let bounds = Bounds::new(self.pos + Vector2::ONE * PADDING + Vector2::with_x(self.size.x - SQUARE * 2.0), Vector2::ONE * SQUARE);
            self.delete_hover = bounds.contains(p);
        } else {
            self.delete_hover = false;
        }
    }

    fn on_click_tagged(&mut self, _pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> Option<String> {
        if self.delete_hover {
            self.delete_time = Some(Instant::now());
            Some("remove_".to_owned() + &self.notification.id.to_string())
        } else if self.hover {
            self.delete_time = Some(Instant::now());
            Some(self.tag.clone())
        } else { 
            None
        }
    }
    fn update(&mut self) {
        if let Some(time) = self.delete_time {
            if time.as_millis() >= DELETE_DURATION {
                self.size = Vector2::ZERO;
            }
        }
        if let Some(time) = self.added_timer {
            if time.as_millis() >= DELETE_DURATION {
                self.added_timer = None;
            }
        }
    }

    fn draw(&mut self, pos_offset: Vector2, list: &mut RenderableCollection) {
        let mut pos = self.pos + pos_offset;
        let mut alpha = 1.0;

        // when deleted, have it swipe to the left
        if let Some(time) = &self.delete_time {
            let amount = time.as_millis() / DELETE_DURATION;
            pos.x += self.size.x * amount;
            alpha = 1.0 - amount;
        }
        if let Some(time) = &self.added_timer {
            let amount = time.as_millis() / DELETE_DURATION;
            pos.x += self.size.x - self.size.x * amount;
            alpha = amount;
        }

        // draw bounding rectangle
        let color = self.notification.color;
        list.push(Rectangle::new(pos, self.size, color.alpha(0.5*alpha), Some(Border::new(color.alpha(alpha), 2.0))).shape(Shape::Round(PADDING)));
        
        pos += Vector2::ONE * PADDING;

        // draw delete button
        let (c, c_color) = if self.delete_hover {(FontAwesome::WindowClose, Color::RED)} else {(FontAwesome::WindowCloseOutline, Color::WHITE)};
        list.push(Text::new(pos + Vector2::with_x(self.size.x - SQUARE * 2.0), SQUARE, c, c_color, Font::FontAwesome));

        // text
        // let bounds = Bounds::new(pos + Vector2::ONE * PADDING, self.size - Vector2::ONE * PADDING * 2.0);
        list.push(Text::new(pos - Vector2::with_y(PADDING), FONT_SIZE, &self.notification.text, Color::WHITE.alpha(alpha), Font::Main));
    }
}
