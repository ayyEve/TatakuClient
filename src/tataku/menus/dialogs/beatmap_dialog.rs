use crate::prelude::*;

//TODO: proper window size
const Y_PADDING:f32 = 5.0;
const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 30.0);

pub struct BeatmapDialog {
    bounds: Rectangle,
    target_map: Md5Hash,
    delete_map: MenuButton,
    copy_hash: MenuButton,
    should_close: bool,
}
impl BeatmapDialog {
    pub fn new(map_hash: Md5Hash) -> Self {
        let window = WindowSize::get();

        let offset = 100.0;
        let mut count = 0;

        let delete_map = MenuButton::new(
            Vector2::new((window.x - BUTTON_SIZE.x) / 2.0, offset + (count as f32 * (BUTTON_SIZE.y + Y_PADDING))),
            BUTTON_SIZE,
            "Delete Map",
            Font::Main,
        );
        count += 1;

        let copy_hash = MenuButton::new(
            Vector2::new((window.x - BUTTON_SIZE.x) / 2.0, offset + (count as f32 * (BUTTON_SIZE.y + Y_PADDING))),
            BUTTON_SIZE,
            "Copy Hash",
            Font::Main,
        );


        let bounds = Rectangle::new(
            Vector2::ZERO,
            window.0,
            Color::BLACK.alpha(0.7),
            Some(Border::new(
                Color::BLACK, 
                1.5
            ))
        );
        
        Self {
            bounds,
            delete_map,
            copy_hash,

            target_map: map_hash,

            should_close: false
        }
    }
}

#[async_trait]
impl Dialog<Game> for BeatmapDialog {
    fn should_close(&self) -> bool { self.should_close }
    fn get_bounds(&self) -> Bounds { *self.bounds }
    async fn force_close(&mut self) { self.should_close = true; }

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        // self.window_size = window_size;
        self.bounds.size = window_size.0;
    }


    async fn on_key_press(&mut self, _key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        false
    }

    async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
        self.delete_map.on_mouse_move(pos)
    }
    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, game:&mut Game) -> bool {
        if self.delete_map.on_click(pos, button, *mods) {
            trace!("delete map {}", self.target_map);

            BEATMAP_MANAGER.write().await.delete_beatmap(self.target_map, game).await;
            self.should_close = true;
        }

        if self.copy_hash.on_click(pos, button, *mods) {
            trace!("copy hash map {}", self.target_map);
            match GameWindow::set_clipboard(self.target_map.to_string()) {
                Ok(_) => NotificationManager::add_text_notification("Hash copied to clipboard!", 3000.0, Color::LIGHT_BLUE).await,
                Err(e) => NotificationManager::add_error_notification("Failed to copy hash to clipboard", e).await,
            }
            self.should_close = true;
        }

        true
    }

    async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
        // background and border
        let mut bounds = self.bounds;
        bounds.pos += offset;
        list.push(bounds);

        // draw buttons
        self.delete_map.draw(offset, list);
        self.copy_hash.draw(offset, list);
    }

}
