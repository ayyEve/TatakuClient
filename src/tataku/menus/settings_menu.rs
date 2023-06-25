use crate::prelude::*;

const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
const SECTION_XOFFSET:f32 = 30.0;
const SCROLLABLE_YOFFSET:f32 = 20.0;
const WIDTH:f32 = 600.0;

pub struct SettingsMenu {
    scroll_area: ScrollableArea,
    old_settings: Settings,

    window_size: Arc<WindowSize>,
    change_receiver: AsyncMutex<Receiver<()>>,
    mouse_pos: Vector2,

    should_close: bool,
}
impl SettingsMenu {
    pub async fn new() -> SettingsMenu {
        let settings = get_settings!().clone();
        let p = Vector2::with_x(SECTION_XOFFSET); // scroll area edits the y
        let window_size = WindowSize::get();

        let (sender, change_receiver) = std::sync::mpsc::sync_channel(100);

        // setup items
        let mut scroll_area = ScrollableArea::new(Vector2::new(10.0, SCROLLABLE_YOFFSET), Vector2::new(window_size.x - 20.0, window_size.y - SCROLLABLE_YOFFSET*2.0), true);
        
        let items = settings.get_menu_items(p, Arc::new(sender));
        for i in items {
            scroll_area.add_item(i);
        }
        let font = get_font();

        //TODO: make these not part of the scrollable?!?!

        // revert button
        let mut revert_button = MenuButton::new(p, BUTTON_SIZE, "Revert", font.clone());
        revert_button.set_tag("revert");

        // done button
        let mut done_button = MenuButton::new(p, BUTTON_SIZE, "Done", font.clone());
        done_button.set_tag("done");

        scroll_area.add_item(Box::new(revert_button));
        scroll_area.add_item(Box::new(done_button));


        SettingsMenu {
            scroll_area,
            old_settings: settings.as_ref().clone(),
            window_size,
            change_receiver: AsyncMutex::new(change_receiver),
            should_close: false,
            mouse_pos: Vector2::ZERO,
        }
    }

    pub async fn update_settings(&mut self) {
        let mut settings = get_settings_mut!();
        settings.from_menu(&self.scroll_area);
        settings.check_hashes();
    }
    pub async fn revert(&mut self) { 
        let mut s = get_settings_mut!();
        *s = self.old_settings.clone();
        s.skip_autosaveing = false;

        self.should_close = true;
    }
    pub async fn finalize(&mut self) {
        self.update_settings().await;
        get_settings_mut!().skip_autosaveing = false;

        self.should_close = true;
    }

}

#[async_trait]
impl Dialog<Game> for SettingsMenu {
    fn name(&self) -> &'static str { "settings_menu" }
    fn should_close(&self) -> bool { self.should_close }
    async fn force_close(&mut self) {
        self.finalize().await;
    }

    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::new(10.0, SCROLLABLE_YOFFSET), 
            Vector2::new(WIDTH + SECTION_XOFFSET * 2.0, self.window_size.y - SCROLLABLE_YOFFSET*2.0)
        )
    }

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.scroll_area.set_size(Vector2::new(window_size.x - 20.0, window_size.y - SCROLLABLE_YOFFSET*2.0));
        self.window_size = window_size.clone();
    }
    
    async fn update(&mut self, _game: &mut Game) {
        if let Ok(Ok(_)) = self.change_receiver.try_lock().map(|e|e.try_recv()) {
            self.update_settings().await;
        }

        self.scroll_area.update()
    }
    async fn draw(&mut self, depth: f32, list: &mut RenderableCollection) {
        self.scroll_area.draw(Vector2::ZERO, depth, list);

        // background
        let bounds = self.get_bounds();
        list.push(visibility_bg(
            bounds.pos, 
            bounds.size,
            depth + 10.0
        ));
    }

    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _game:&mut Game) -> bool {
        if !self.get_bounds().contains(pos) { return false }

        if let Some(tag) = self.scroll_area.on_click_tagged(pos, button, *mods) {
            match tag.as_str() {
                "done" => self.finalize().await,
                "revert" => self.revert().await,
                _ => {}
            }
        }

        true
    }

    async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        if !self.get_bounds().contains(pos) { return false }

        self.scroll_area.on_click_release(pos, button);
        true
    }

    async fn on_key_press(&mut self, key:Key, mods:&KeyModifiers, _game:&mut Game) -> bool {
        if self.scroll_area.get_selected_index().is_none() { return false }

        self.scroll_area.on_key_press(key, *mods);

        if key == Key::Escape {
            self.finalize().await;
        }

        true
    }

    async fn on_key_release(&mut self, key:Key, _mods:&KeyModifiers, _game:&mut Game) -> bool {
        if self.scroll_area.get_selected_index().is_none() { return false }

        self.scroll_area.on_key_release(key);
        true
    }

    async fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        self.mouse_pos = pos;
        self.scroll_area.on_mouse_move(pos);
    }
    async fn on_mouse_scroll(&mut self, delta:f32, _game:&mut Game) -> bool {
        if !self.get_bounds().contains(self.mouse_pos) { return false }
        self.scroll_area.on_scroll(delta);
        true
    }
    async fn on_text(&mut self, text:&String) -> bool {
        if self.scroll_area.get_selected_index().is_none() { return false }

        self.scroll_area.on_text(text.clone()); 
        true
    }
}