use crate::prelude::*;

const TOOLBAR_HEIGHT:f32 = 40.0;
const BUTTON_MARGIN:f32 = 4.0;

pub struct DraggableDialog<G:Send+Sync> {
    inner: Box<dyn Dialog<G>>,

    /// toolbar pos is also this dialog pos (both are top-left)
    toolbar_bounds: Bounds,
    /// (old_pos, click_pos)
    drag_offset_pos: Option<(Vector2, Vector2)>,
    
    // minimized: bool,
    
    close_button_bounds: Bounds,
    close_button_hover: bool,
}
impl<G:Send+Sync> DraggableDialog<G> {
    pub fn new(pos: impl Into<DraggablePosition>, inner: Box<dyn Dialog<G>>) -> Self {
        let inner_bounds = inner.get_bounds();
        let pos:DraggablePosition = pos.into();
        let pos = pos.get_pos(inner_bounds.size + Vector2::with_y(TOOLBAR_HEIGHT), Vector2::ZERO, WindowSize::get().0);

        let toolbar_bounds = Bounds::new(pos, Vector2::new(inner_bounds.size.x, TOOLBAR_HEIGHT));
        let close_button_bounds = Bounds::new(
            toolbar_bounds.pos + Vector2::new(inner_bounds.size.x-(TOOLBAR_HEIGHT + BUTTON_MARGIN * 2.0), BUTTON_MARGIN), 
            Vector2::ONE * (TOOLBAR_HEIGHT - BUTTON_MARGIN * 2.0)
        );

        Self {
            inner, 
            toolbar_bounds,
            drag_offset_pos: None,

            close_button_bounds,
            close_button_hover: false
        }
    }

    fn on_move(&mut self, new_pos: Vector2) {
        self.toolbar_bounds.pos = new_pos;
        self.close_button_bounds.pos = self.toolbar_bounds.pos + Vector2::new(self.inner.get_bounds().size.x-(TOOLBAR_HEIGHT + BUTTON_MARGIN * 2.0 ), BUTTON_MARGIN);
    }

    fn map_pos(&self, p: Vector2) -> Vector2 {
        p - (self.toolbar_bounds.pos + self.toolbar_bounds.size.y_portion())
    }
}

#[async_trait]
impl<G:Send+Sync> Dialog<G> for DraggableDialog<G> {
    fn name(&self) -> &'static str { self.inner.name() }
    fn should_close(&self) -> bool { self.inner.should_close() }
    fn get_bounds(&self) -> Bounds {
        let mut bounds = self.toolbar_bounds;
        bounds.size.y += self.inner.get_bounds().size.y;
        bounds
    }

    async fn force_close(&mut self) { self.inner.force_close().await; }
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) { self.inner.window_size_changed(window_size).await; }

    async fn resized(&mut self, new_size: Vector2) {
        self.toolbar_bounds.size.x = new_size.x;
        self.close_button_bounds.pos = self.toolbar_bounds.pos + Vector2::new(self.inner.get_bounds().size.x-(TOOLBAR_HEIGHT + BUTTON_MARGIN * 2.0 ), BUTTON_MARGIN);

        self.inner.resized(new_size).await;
    }

    async fn update(&mut self, g:&mut G) { self.inner.update(g).await; }
    async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
        let inner_bounds = self.inner.get_bounds();
        let pos = self.toolbar_bounds.pos + offset;
        
        // draw inner first so its behind everything
        list.push_scissor([pos.x, pos.y + TOOLBAR_HEIGHT, inner_bounds.size.x, inner_bounds.size.y]);
        self.inner.draw(pos + self.toolbar_bounds.size.y_portion(), list).await;
        list.pop_scissor();
        
        // draw toolbar
        let toolbar = Rectangle::new(pos, Vector2::new(inner_bounds.size.x, TOOLBAR_HEIGHT), Color::GRAY.alpha(0.5), Some(Border::new(Color::BLACK, 2.0)));
        list.push(toolbar);

        // title
        let title = self.inner.title();
        if !title.is_empty() {
            let font_size = TOOLBAR_HEIGHT * 0.8;
            let pad = (TOOLBAR_HEIGHT-font_size) / 2.0;
            let title = Text::new(pos + Vector2::new(pad, pad), font_size, title.to_owned(), Color::BLACK, Font::Main);
            list.push(title);
        }

        // close button
        let close_color = if self.close_button_hover { Color::RED } else { Color::TRANSPARENT_WHITE };
        let close_border_color = if self.close_button_hover { Color::GRAY } else { Color::BLACK };
        let close_rect = Rectangle::new(self.close_button_bounds.pos+offset, self.close_button_bounds.size, close_color, Some(Border::new(close_border_color, 1.0)));
        list.push(close_rect);

        // FontAwesome::WindowClose
        let mut x = Text::new(Vector2::ZERO, self.close_button_bounds.size.y * 0.8, "X".to_owned(), Color::BLACK, Font::Main);
        x.center_text(&*close_rect);
        list.push(x);
    }

    // input handlers
    async fn on_mouse_scroll(&mut self, delta:f32, g:&mut G) -> bool { self.inner.on_mouse_scroll(delta, g).await }
    async fn on_mouse_move(&mut self, pos:Vector2, g:&mut G) {
        // check for drags
        if let Some((old_pos, click_pos)) = self.drag_offset_pos {
            self.on_move(old_pos + (pos - click_pos));
            return;
        }

        // check for close button hover
        self.close_button_hover = self.close_button_bounds.contains(pos);

        // do inner mouse move
        self.inner.on_mouse_move(self.map_pos(pos), g).await;
    }
    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, g:&mut G) -> bool {
        if self.close_button_hover {
            self.force_close().await;
            return true;
        }
        if self.toolbar_bounds.contains(pos) {
            self.drag_offset_pos = Some((self.toolbar_bounds.pos, pos));
            return true;
        }
        
        self.inner.on_mouse_down(self.map_pos(pos), button, mods, g).await
    }
    async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, g:&mut G) -> bool {
        if self.drag_offset_pos.is_some() {
            self.drag_offset_pos = None;
            return true;
        }

        self.inner.on_mouse_up(self.map_pos(pos), button, mods, g).await
    }

    async fn on_text(&mut self, text:&String) -> bool { self.inner.on_text(text).await }
    async fn on_key_press(&mut self, key:Key, mods:&KeyModifiers, g:&mut G) -> bool { self.inner.on_key_press(key, mods, g).await }
    async fn on_key_release(&mut self, key:Key, mods:&KeyModifiers, g:&mut G) -> bool { self.inner.on_key_release(key, mods, g).await }

    async fn on_controller_press(&mut self, controller: &GamepadInfo, button: ControllerButton) -> bool { self.inner.on_controller_press(controller, button).await }
    async fn on_controller_release(&mut self, controller: &GamepadInfo, button: ControllerButton) -> bool { self.inner.on_controller_release(controller, button).await }
    async fn on_controller_axis(&mut self, controller: &GamepadInfo, axis_data: &HashMap<Axis, (bool, f32)>) { self.on_controller_axis(controller, axis_data).await }
}

#[allow(unused)]
pub enum DraggablePosition {
    TopLeft, TopMiddle, TopRight,
    CenterLeft, CenterMiddle, CenterRight,
    BottomLeft, BottomMiddle, BottomRight,
    Custom(Vector2)
}
impl DraggablePosition {
    pub fn get_pos(&self, obj_size: Vector2, container_pos: Vector2, container_size: Vector2) -> Vector2 {
        container_pos + match self {
            DraggablePosition::TopLeft => Vector2::ZERO,
            DraggablePosition::TopMiddle => Vector2::with_x((container_size.x - obj_size.x) / 2.0),
            DraggablePosition::TopRight => Vector2::with_x(container_size.x - obj_size.x),

            DraggablePosition::CenterLeft => Vector2::with_y((container_size.y - obj_size.y) / 2.0),
            DraggablePosition::CenterMiddle => (container_size - obj_size) / 2.0,
            DraggablePosition::CenterRight => Vector2::new(container_size.x - obj_size.x, (container_size.y - obj_size.y) / 2.0),

            DraggablePosition::BottomLeft => Vector2::with_y(container_size.y - obj_size.y),
            DraggablePosition::BottomMiddle => Vector2::new((container_size.x - obj_size.x) / 2.0, container_size.y - obj_size.y),
            DraggablePosition::BottomRight => container_size - obj_size,

            DraggablePosition::Custom(pos) => return *pos,
        }
    }
}
impl From<Vector2> for DraggablePosition {
    fn from(value: Vector2) -> Self {
        Self::Custom(value)
    }
}
