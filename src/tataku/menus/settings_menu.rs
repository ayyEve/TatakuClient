use crate::prelude::*;

const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
const SECTION_XOFFSET:f64 = 30.0;
const SCROLLABLE_YOFFSET:f64 = 20.0;
const WIDTH:f64 = 600.0;

pub struct SettingsMenu {
    scroll_area: ScrollableArea,
    old_settings: Settings,

    window_size: Arc<WindowSize>,

    change_receiver: Mutex<Receiver<()>>,
    menu_game: MenuGameHelper,
}
impl SettingsMenu {
    pub async fn new() -> SettingsMenu {
        let settings = get_settings!().clone();
        let p = Vector2::new(SECTION_XOFFSET, 0.0); // scroll area edits the y
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
        let mut revert_button = MenuButton::<Font2, Text>::new(p, BUTTON_SIZE, "Revert", font.clone());
        revert_button.set_tag("revert");

        // done button
        let mut done_button = MenuButton::<Font2, Text>::new(p, BUTTON_SIZE, "Done", font.clone());
        done_button.set_tag("done");

        scroll_area.add_item(Box::new(revert_button));
        scroll_area.add_item(Box::new(done_button));


        SettingsMenu {
            scroll_area,
            old_settings: settings.as_ref().clone(),
            window_size,
            change_receiver: Mutex::new(change_receiver),
            menu_game: MenuGameHelper::new(true, false)
        }
    }

    pub async fn update_settings(&mut self) {
        // write settings to settings
        let mut settings = get_settings_mut!();
        settings.from_menu(&self.scroll_area);

        settings.check_hashes();
        // drop to make sure changes propogate correctly
        drop(settings);

        self.menu_game.force_update_settings().await;
    }
    pub async fn revert(&mut self, game:&mut Game) { 
        {
            let mut s = get_settings_mut!();
            *s = self.old_settings.clone();
            s.skip_autosaveing = false;
        }
        
        let menu = game.menus.get("main").unwrap().clone();
        game.queue_state_change(GameState::InMenu(menu));
    }
    pub async fn finalize(&mut self, game:&mut Game) {
        self.update_settings().await;
        get_settings_mut!().skip_autosaveing = false;

        let menu = game.menus.get("main").unwrap().clone();
        game.queue_state_change(GameState::InMenu(menu));
    }

}

#[async_trait]
impl AsyncMenu<Game> for SettingsMenu {
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.scroll_area.set_size(Vector2::new(window_size.x - 20.0, window_size.y - SCROLLABLE_YOFFSET*2.0));
        self.window_size = window_size.clone();
        self.menu_game.window_size_changed(window_size).await;


        let pos = Vector2::new(WIDTH, 0.0);
        let window_size = self.window_size.0;
        let size = Vector2::new(
            window_size.x - WIDTH,
            window_size.y
        );

        self.menu_game.fit_to_area(pos, size).await;
    }
    
    async fn on_change(&mut self, into:bool) {
        if into {
            get_settings_mut!().skip_autosaveing = true;

            // update our window size
            self.window_size_changed(WindowSize::get()).await;

            // play song if it exists
            if let Some(song) = AudioManager::get_song().await {
                // reset any time mods

                song.set_rate(1.0);
                // // play
                // song.play(true);
            }

            self.menu_game.setup().await;
        } else {
            debug!("leaving settings menu");
        }
    }

    
    async fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        self.scroll_area.draw(args, Vector2::zero(), 0.0, &mut list);

        // background
        list.push(visibility_bg(
            Vector2::new(10.0, SCROLLABLE_YOFFSET), 
            Vector2::new(WIDTH + SECTION_XOFFSET * 2.0, self.window_size.y - SCROLLABLE_YOFFSET*2.0),
            10.0
        ));
        
        self.menu_game.draw(args, &mut list).await;

        list
    }

    async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        if let Some(tag) = self.scroll_area.on_click_tagged(pos, button, mods) {
            match tag.as_str() {
                "done" => self.finalize(game).await,
                "revert" => self.revert(game).await,
                _ => {}
            }
        }
    }

    async fn on_click_release(&mut self, pos:Vector2, button:MouseButton, _g:&mut Game) {
        self.scroll_area.on_click_release(pos, button);
    }

    async fn on_key_press(&mut self, key:piston::Key, game:&mut Game, mods:KeyModifiers) {
        self.scroll_area.on_key_press(key, mods);

        if key == piston::Key::Escape {
            self.finalize(game).await;
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }
    }

    async fn on_key_release(&mut self, key:piston::Key, _game:&mut Game) {
        self.scroll_area.on_key_release(key);
    }

    async fn update(&mut self, game: &mut Game) {
        if let Ok(Ok(_)) = self.change_receiver.try_lock().map(|e|e.try_recv()) {
            self.update_settings().await;
        }

        self.menu_game.update().await;
        
        
        let mut song_done = false;
        match AudioManager::get_song().await {
            Some(song) => {
                if !song.is_playing() && !song.is_paused() {
                    song_done = true;
                }
            }
            _ => song_done = true,
        }

        if song_done {
            trace!("song done");
            BEATMAP_MANAGER.write().await.next_beatmap(game).await;
            self.menu_game.setup().await;
        }


        self.scroll_area.update()
    }
    async fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {self.scroll_area.on_mouse_move(pos)}
    async fn on_scroll(&mut self, delta:f64, _game:&mut Game) {self.scroll_area.on_scroll(delta);}
    async fn on_text(&mut self, text:String) {self.scroll_area.on_text(text)}
}
impl ControllerInputMenu<Game> for SettingsMenu {
    
}