use crate::prelude::*;

const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
const SECTION_XOFFSET:f32 = 30.0;
const SCROLLABLE_YOFFSET:f32 = 20.0;
const WIDTH:f32 = 600.0;

const SEARCH_HEIGHT:f32 = 50.0;

pub struct SettingsMenu {
    scroll_area: ScrollableArea,
    search_text: TextInput,

    old_settings: Settings,

    window_size: Arc<WindowSize>,
    change_receiver: AsyncMutex<Receiver<()>>,
    mouse_pos: Vector2,

    should_close: bool,
}
impl SettingsMenu {
    pub async fn new() -> SettingsMenu {
        let settings = Settings::get().clone();
        let p = Vector2::with_x(SECTION_XOFFSET - 10.0); // scroll area edits the y
        let window_size = WindowSize::get();
        let font = get_font();

        let (sender, change_receiver) = std::sync::mpsc::sync_channel(100);

        // setup items
        let search_text = TextInput::new(p + Vector2::with_y(SCROLLABLE_YOFFSET + 5.0), Vector2::new(WIDTH - SECTION_XOFFSET, SEARCH_HEIGHT), "Search", "", font.clone());
        let mut scroll_area = ScrollableArea::new(Vector2::new(10.0, SCROLLABLE_YOFFSET + SEARCH_HEIGHT + 10.0), Vector2::new(WIDTH + SECTION_XOFFSET+1.0, window_size.y - (SEARCH_HEIGHT + SCROLLABLE_YOFFSET*2.0)), ListMode::VerticalList);
        
        let items = settings.get_menu_items(p, String::new(), Arc::new(sender));
        for i in items {
            scroll_area.add_item(i);
        }


        //TODO: make these not part of the scrollable?!?!

        // revert button
        scroll_area.add_item(Box::new(MenuButton::new(p, BUTTON_SIZE, "Revert", font.clone()).with_tag("revert")));

        // done button
        scroll_area.add_item(Box::new(MenuButton::new(p, BUTTON_SIZE, "Done", font.clone()).with_tag("done")));

        SettingsMenu {
            scroll_area,
            search_text,

            old_settings: settings.as_ref().clone(),
            window_size,
            change_receiver: AsyncMutex::new(change_receiver),
            should_close: false,
            mouse_pos: Vector2::ZERO,
        }
    }

    pub async fn update_settings(&mut self) {
        let mut settings = Settings::get_mut();

        // need to re-add all items  before we run the from_menu fn
        self.scroll_area.rejoin_items();

        // update settings
        settings.from_menu(String::new(), &self.scroll_area);

        // reapply filter
        self.apply_filter(false);

        settings.check_hashes();
    }
    pub async fn revert(&mut self) { 
        let mut s = Settings::get_mut();
        *s = self.old_settings.clone();
        s.skip_autosaveing = false;

        self.should_close = true;
    }
    pub async fn finalize(&mut self) {
        self.update_settings().await;
        Settings::get_mut().skip_autosaveing = false;

        self.should_close = true;
    }

    fn apply_filter(&mut self, refresh: bool) {
        let text = self.search_text.get_text();
        if text.is_empty() {
            self.scroll_area.apply_filter(&Vec::new(), refresh);
        } else {
            let query = text.split(" ").map(|a|a.to_owned()).collect::<Vec<String>>();
            self.scroll_area.apply_filter(&query, refresh);
        }
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
            Vector2::ZERO, 
            Vector2::new(WIDTH + SECTION_XOFFSET * 2.0, self.window_size.y)
        )
    }

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.scroll_area.set_size(Vector2::new(WIDTH + SECTION_XOFFSET+1.0, window_size.y - (SEARCH_HEIGHT + SCROLLABLE_YOFFSET*2.0)));
        self.window_size = window_size.clone();
    }
    
    async fn update(&mut self, _game: &mut Game) {
        if let Ok(Ok(_)) = self.change_receiver.try_lock().map(|e|e.try_recv()) {
            self.update_settings().await;
        }

        self.scroll_area.update();
        
        let old_text = self.search_text.get_text();
        self.search_text.update();

        if old_text != self.search_text.get_text() {
            self.apply_filter(true);
        }
    }
    async fn draw(&mut self, list: &mut RenderableCollection) {
        // background
        let bounds = self.get_bounds();
        list.push(visibility_bg(
            bounds.pos, 
            bounds.size,
        ));

        self.search_text.draw(Vector2::ZERO, list);
        self.scroll_area.draw(Vector2::ZERO, list);
    }

    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _game:&mut Game) -> bool {
        if !self.get_bounds().contains(pos) { return false }
        self.search_text.on_click(pos, button, *mods);

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
        self.search_text.on_click_release(pos, button);

        self.scroll_area.on_click_release(pos, button);
        true
    }

    async fn on_key_press(&mut self, key:Key, mods:&KeyModifiers, _game:&mut Game) -> bool {
        // if esc is pressed, override whether we're selected or not
        if key == Key::Escape {
            self.finalize().await;
            return true;
        }

        if self.search_text.get_selected() {
            let old_text = self.search_text.get_text();
            self.search_text.on_key_press(key, *mods);

            let new_text = self.search_text.get_text();
            if new_text != old_text {
                self.apply_filter(true);
            }

            return true;
        }

        if self.scroll_area.get_selected_index().is_none() { return false }
        self.scroll_area.on_key_press(key, *mods);

        true
    }

    async fn on_key_release(&mut self, key:Key, _mods:&KeyModifiers, _game:&mut Game) -> bool {
        if self.search_text.get_selected() {
            self.search_text.on_key_release(key);
            return true
        }
        if self.scroll_area.get_selected_index().is_none() { return false }

        self.scroll_area.on_key_release(key);
        true
    }

    async fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        self.mouse_pos = pos;
        self.scroll_area.on_mouse_move(pos);
        self.search_text.on_mouse_move(pos)
    }
    async fn on_mouse_scroll(&mut self, delta:f32, _game:&mut Game) -> bool {
        if !self.get_bounds().contains(self.mouse_pos) { return false }
        self.scroll_area.on_scroll(delta);
        true
    }
    async fn on_text(&mut self, text:&String) -> bool {        
        if self.search_text.get_selected() {
            let old_text = self.search_text.get_text();
            self.search_text.on_text(text.clone());
            let new_text = self.search_text.get_text();
            if new_text != old_text {
                self.apply_filter(true);
            }

            return true;
        }
        if self.scroll_area.get_selected_index().is_none() { return false }

        self.scroll_area.on_text(text.clone()); 
        true
    }
}