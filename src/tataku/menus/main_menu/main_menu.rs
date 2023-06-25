use crate::prelude::*;
use super::MusicBox;
use super::menu_button::MainMenuButton;

const BUTTON_SIZE: Vector2 = Vector2::new(100.0, 50.0);
const Y_MARGIN:f32 = 20.0;
const Y_OFFSET:f32 = 10.0;

const MENU_HIDE_TIMER:f32 = 5_000.0;
// const COOKIE_HIDE_TIMER:f32 = 10_000.0;
// const COOKIE_FADE_TIME:f32 = 10_000.0;

pub struct MainMenu {
    // index 0
    pub play_button: MainMenuButton,
    // // index 1
    // pub direct_button: MainMenuButton,
    // index 2
    pub settings_button: MainMenuButton,
    // index 3
    pub exit_button: MainMenuButton,

    visualization: MenuVisualization,
    menu_game: MenuGameHelper,

    selected_index: usize,
    menu_visible: bool,
    last_input: Instant,

    settings: SettingsHelper,
    window_size: Arc<WindowSize>,
    song_display: CurrentSongDisplay,
    new_map_helper: LatestBeatmapHelper,
    current_skin: CurrentSkinHelper,

    music_box: MusicBox,
    media_controls: MediaControlHelper,
    event_sender: AsyncUnboundedSender<MediaControlHelperEvent>,
    event_receiver: AsyncUnboundedReceiver<MediaControlHelperEvent>,
}
impl MainMenu {
    pub async fn new() -> MainMenu {
        let window_size = WindowSize::get();
        let middle = window_size.x /2.0 - BUTTON_SIZE.x/2.0;
        let mut counter = 1.0;
        
        let mut play_button = MainMenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Play", "menu-button-play").await;
        // counter += 1.0;
        // let mut direct_button = MainMenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "osu!Direct").await;
        counter += 1.0;
        let mut settings_button = MainMenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Settings", "menu-button-options").await;
        counter += 1.0;
        let mut exit_button = MainMenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Exit", "menu-button-exit").await;

        play_button.visible = false;
        // direct_button.visible = false;
        settings_button.visible = false;
        exit_button.visible = false;

        let (event_sender, event_receiver) = async_unbounded_channel();

        MainMenu {
            play_button,
            // direct_button,
            settings_button,
            exit_button,

            visualization: MenuVisualization::new().await,
            menu_game: MenuGameHelper::new(false, false, Box::new(|s|s.background_game_settings.main_menu_enabled)),
            selected_index: 99,
            menu_visible: false,
            music_box: MusicBox::new(event_sender.clone()).await,
            media_controls: MediaControlHelper::new(event_sender.clone()),

            event_sender, 
            event_receiver,


            settings: SettingsHelper::new(),
            window_size,
            last_input: Instant::now(),
            song_display: CurrentSongDisplay::new(),
            new_map_helper: LatestBeatmapHelper::new(),
            current_skin: CurrentSkinHelper::new(),
        }
    }

    async fn setup_manager(&mut self, called_by: &str) {
        trace!("setup manager called by {}", called_by);
        self.settings.update();

        if let Some(song) = AudioManager::get_song().await {
            let duration = song.get_duration();
            self.music_box.update_song_duration(duration);

            let map = &CurrentBeatmapHelper::new().0;
            self.media_controls.update_info(map, duration);
        }

        let settings = self.settings.background_game_settings.clone();
        if !settings.main_menu_enabled { return }

        self.menu_game.setup().await;
        self.visualization.song_changed(&mut self.menu_game.manager);

        trace!("manager setup");
    }

    fn show_menu(&mut self) {
        self.menu_visible = true;

        // ensure they have the latest window size
        self.play_button.window_size = self.window_size.0;
        // self.direct_button.window_size = self.window_size.0;
        self.settings_button.window_size = self.window_size.0;
        self.exit_button.window_size = self.window_size.0;

        // show
        let count = 3;
        let mut counter = 0;
        self.play_button.show(counter, count, true); counter += 1;
        // self.direct_button.show(counter, count, true); counter += 1;
        self.settings_button.show(counter, count, true); counter += 1;
        self.exit_button.show(counter, count, true); // counter += 1;
    }

    fn hide_menu(&mut self) {
        self.menu_visible = false;

        // ensure they have the latest window size
        self.play_button.window_size = self.window_size.0;
        // self.direct_button.window_size = self.window_size.0;
        self.settings_button.window_size = self.window_size.0;
        self.exit_button.window_size = self.window_size.0;

        // hide
        let count = 3;
        let mut counter = 0;
        self.play_button.hide(counter, count, true); counter += 1;
        // self.direct_button.hide(1, 4, true); counter += 1;
        self.settings_button.hide(counter, count, true); counter += 1;
        self.exit_button.hide(counter, count, true); // counter += 1;
    }
    

    fn interactables(&mut self, include_buttons: bool) -> Vec<&mut dyn ScrollableItem> {
        if include_buttons {
            vec![
                &mut self.music_box,
                &mut self.play_button,
                // &mut self.direct_button,
                &mut self.settings_button,
                &mut self.exit_button,
            ]
        } else {
            vec![
                &mut self.music_box,
            ]
        }
    }

    fn update_online() {
        tokio::spawn(async move {
            let Some(map) = BEATMAP_MANAGER.read().await.current_beatmap.clone() else { return };

            if let Some(song) = AudioManager::get_song().await {
                OnlineManager::set_action(SetAction::Listening { 
                    artist: map.artist.clone(), 
                    title: map.title.clone(),
                    elapsed: song.get_position(),
                    duration: song.get_duration()
                }, None);
            }
        });
    }

    async fn next(&mut self, game: &mut Game) -> bool {
        let mut manager = BEATMAP_MANAGER.write().await;

        if manager.next_beatmap(game).await {
            Self::update_online();
            true
        } else {
            trace!("no next");
            false
        }
    }
    async fn previous(&mut self, game: &mut Game) -> bool {
        let mut manager = BEATMAP_MANAGER.write().await;
        
        if manager.previous_beatmap(game).await {
            Self::update_online();
            true
        } else {
            trace!("no prev");
            false
        }
    }

    fn reset_timer(&mut self) {
        self.last_input = Instant::now()
    }
}
#[async_trait]
impl AsyncMenu<Game> for MainMenu {
    fn get_name(&self) -> &str {"main_menu"}

    async fn on_change(&mut self, into:bool) {
        if into {
            // update our window size
            self.window_size_changed(WindowSize::get()).await;
            self.new_map_helper.update();

            self.visualization.reset();

            // play song if it exists
            if let Some(song) = AudioManager::get_song().await {
                // reset any time mods
                song.set_rate(1.0);
                // // play
                // song.play(true).unwrap();
            }

            // update online to what song we're listening to
            Self::update_online();

            self.setup_manager("on_change").await;
            
            self.hide_menu();
        } else {
            debug!("leaving main menu");
        }
    }

    async fn update(&mut self, g:&mut Game) {
        self.settings.update();
        self.song_display.update();

        if self.current_skin.update() {
            self.visualization.reload_skin().await;

            self.play_button = MainMenuButton::new(Vector2::ZERO, BUTTON_SIZE, "Play", "menu-button-play").await;
            // self.direct_button = MainMenuButton::new(Vector2::ZERO, BUTTON_SIZE, "osu!Direct").await;
            self.settings_button = MainMenuButton::new(Vector2::ZERO, BUTTON_SIZE, "Settings", "menu-button-options").await;
            self.exit_button = MainMenuButton::new(Vector2::ZERO, BUTTON_SIZE, "Exit", "menu-button-exit").await;

            self.window_size_changed(self.window_size.clone()).await;

            if self.menu_visible {
                self.show_menu();
            } else {
                self.hide_menu();
            }

        }

        let mut song_done = false;

        // run updates on the interactables
        for i in self.interactables(true) {
            i.update()
        }


        match AudioManager::get_song().await {
            Some(song) => {
                let elapsed = song.get_position();
                let state = if song.is_stopped() {
                    MediaPlaybackState::Stopped
                } else if song.is_paused() {
                    MediaPlaybackState::Paused(elapsed)
                } else if song.is_playing() {
                    MediaPlaybackState::Playing(elapsed)
                } else {
                    //  ??
                    unreachable!()
                };

                self.music_box.update_song_time(elapsed);
                self.music_box.update_song_paused(song.is_paused());
                self.media_controls.update(state, self.settings.integrations.media_controls).await;

                let mut needs_manager_setup = false;
                if let Ok(event) = self.event_receiver.try_recv() {
                    match event {
                        MediaControlHelperEvent::Play => song.play(false),
                        MediaControlHelperEvent::Pause => song.pause(),
                        MediaControlHelperEvent::Stop => song.stop(),
                        MediaControlHelperEvent::Toggle => {
                            if song.is_stopped() { 
                                song.play(true); 
                            } else if song.is_playing() {
                                song.pause()
                            } else if song.is_paused() {
                                song.play(false);
                            }
                        }
                        MediaControlHelperEvent::Next     => needs_manager_setup |= self.next(g).await,
                        MediaControlHelperEvent::Previous => needs_manager_setup |= self.previous(g).await,
                        MediaControlHelperEvent::SeekForward => song.set_position(elapsed + 100.0),
                        MediaControlHelperEvent::SeekBackward => song.set_position(elapsed - 100.0),
                        MediaControlHelperEvent::SeekForwardBy(amt) => song.set_position(elapsed + amt),
                        MediaControlHelperEvent::SeekBackwardBy(amt) => song.set_position(elapsed - amt),
                        MediaControlHelperEvent::SetPosition(pos) => song.set_position(pos),
                        // MediaControlHelperEvent::OpenUri(_) => todo!(),
                        // MediaControlHelperEvent::Raise => todo!(),
                        // MediaControlHelperEvent::Quit => todo!(),
                        _ => {}
                    }
                    
                    if needs_manager_setup {
                        self.setup_manager("media event").await;
                        return;
                    }
                }

                if !song.is_playing() && !song.is_paused() {
                    song_done = true;
                }
            }
            _ => song_done = true,
        }

        if song_done {
            trace!("song done");
            let map = BEATMAP_MANAGER.read().await.random_beatmap();

            // it should?
            if let Some(map) = map {
                BEATMAP_MANAGER.write().await.set_current_beatmap(g, &map, false).await;
                self.setup_manager("update song done").await;
                Self::update_online();
            }
        }

        
        if self.new_map_helper.update() {
            BEATMAP_MANAGER.write().await.set_current_beatmap(g, &self.new_map_helper.0, false).await;
            self.setup_manager("update new map").await;
        }

        self.visualization.update(&mut self.menu_game.manager).await;
        self.menu_game.update().await;
    
        // check last input timer
        let last_input = self.last_input.as_millis();
        if last_input > MENU_HIDE_TIMER {
            if self.menu_visible {
                self.hide_menu();
            }
        }

    }

    async fn draw(&mut self, list: &mut RenderableCollection) {
        let pos_offset = Vector2::ZERO;
        let depth = 0.0;

        // draw interactables
        for i in self.interactables(true) {
            i.draw(pos_offset, depth, list)
        }

        // visualization
        let mid = self.window_size.0 / 2.0;
        self.visualization.draw(mid, depth + 10.0, list).await;

        self.menu_game.draw(list).await;

        self.song_display.draw(list);
        
        // draw dim
        list.push(Rectangle::new(
            Color::BLACK.alpha(0.5),
            depth + 11.0,
            Vector2::ZERO,
            self.window_size.0,
            None
        ));
    }

    async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        self.reset_timer();
        if self.visualization.on_click(pos) {
            self.show_menu();
        }

        // switch to beatmap selection
        if self.play_button.on_click(pos, button, mods) {
            // let menu = game.menus.get("beatmap").unwrap().clone();
            game.queue_state_change(GameState::InMenu(Box::new(BeatmapSelectMenu::new().await)));
            return;
        }

        // // open direct menu
        // if self.direct_button.on_click(pos, button, mods) {
        //     let mode = self.settings.background_game_settings.mode.clone();
        //     let menu:Arc<tokio::sync::Mutex<dyn ControllerInputMenu<Game>>> = Arc::new(Mutex::new(DirectMenu::new(mode).await));
        //     game.queue_state_change(GameState::InMenu(menu));
        //     return;
        // }

        // open settings menu
        if self.settings_button.on_click(pos, button, mods) {
            // let menu = Arc::new(Mutex::new());
            game.add_dialog(Box::new(SettingsMenu::new().await), false);
            // game.queue_state_change(GameState::InMenu(Box::new(SettingsMenu::new().await)));
            return;
        }

        // quit game
        if self.exit_button.on_click(pos, button, mods) {
            game.queue_state_change(GameState::Closing);
            return;
        }

        // anything else
        for i in self.interactables(false) {
            if i.on_click(pos, button, mods) {
                break
            }
        }

        // if self.music_box.get_next_pending() {
        //     self.next(game).await;
        //     self.setup_manager("on_click next_pending").await
        // }
        // if self.music_box.get_prev_pending() {
        //     self.previous(game).await;
        //     self.setup_manager("on_click prev_pending").await
        // }

    }

    async fn on_mouse_move(&mut self, pos:Vector2, _game: &mut Game) {
        self.reset_timer();
        for i in self.interactables(true) {
            i.on_mouse_move(pos)
        }
    }

    async fn on_key_press(&mut self, key:Key, game:&mut Game, mods:KeyModifiers) {
        self.reset_timer();
        if mods.ctrl && key == Key::N {
            NotificationManager::add_text_notification("test notif\nnewline1\nnewline2", 4000.0, Color::CRYSTAL_BLUE).await;
        }

        let mut needs_manager_setup = false;
        // if mods.ctrl && key == Key::Up {
        //     self.visualization.index += 1;
        //     info!("i: {}", self.visualization.index)
        // }
        // if mods.ctrl && key == Key::Down {
        //     if self.visualization.index > 0 {
        //         self.visualization.index -= 1;
        //     }
        //     info!("i: {}", self.visualization.index)
        // }

        

        // check offset keys
        self.menu_game.key_down(key, mods).await;

        if !mods.alt {
            match key {
                Key::Left => needs_manager_setup |= self.previous(game).await,
                Key::Right => needs_manager_setup |= self.next(game).await,
                _ => {}
            }
        }
        
        if mods.alt {
            let new_mode = match key {
                Key::Key1 => Some("osu".to_owned()),
                Key::Key2 => Some("taiko".to_owned()),
                Key::Key3 => Some("catch".to_owned()),
                Key::Key4 => Some("mania".to_owned()),
                _ => None
            };

            if let Some(new_mode) = new_mode {
                let mut settings = get_settings_mut!();
                if settings.background_game_settings.mode != new_mode {
                    NotificationManager::add_text_notification(&format!("Menu mode changed to {:?}", new_mode), 1000.0, Color::BLUE).await;
                    needs_manager_setup = true;
                    settings.background_game_settings.mode = new_mode;
                }
            }
        }

        if needs_manager_setup {
            self.setup_manager("key press").await;
        }

    }

    
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.play_button.window_size_changed(&window_size);
        // self.direct_button.window_size_changed(&window_size);
        self.settings_button.window_size_changed(&window_size);
        self.exit_button.window_size_changed(&window_size);

        self.window_size = window_size.clone();
        self.music_box = MusicBox::new(self.event_sender.clone()).await;

        self.menu_game.window_size_changed(window_size).await;
    }
}
#[async_trait]
impl ControllerInputMenu<Game> for MainMenu {
    async fn controller_down(&mut self, game:&mut Game, _controller: &GamepadInfo, button: ControllerButton) -> bool {
        self.reset_timer();

        if !self.menu_visible {
            if button == ControllerButton::South {
                self.show_menu();
                return true;
            }
            return false;
        }

        let mut changed = false;

        match button {
            ControllerButton::DPadDown => {
                self.selected_index += 1;
                if self.selected_index >= 4 {
                    self.selected_index = 0;
                }

                changed = true;
            }

            ControllerButton::DPadUp => {
                if self.selected_index == 0 {
                    self.selected_index = 3;
                } else if self.selected_index >= 4 { // original value is 99
                    self.selected_index = 0;
                } else {
                    self.selected_index -= 1;
                }
                changed = true;
            }

            ControllerButton::South => {
                match self.selected_index {
                    0 => game.queue_state_change(GameState::InMenu(Box::new(BeatmapSelectMenu::new().await))),
                    1 => game.queue_state_change(GameState::InMenu(Box::new(DirectMenu::new(self.settings.background_game_settings.mode.clone()).await))),
                    2 => game.add_dialog(Box::new(SettingsMenu::new().await), false), //game.queue_state_change(GameState::InMenu(Box::new(SettingsMenu::new().await))),
                    3 => game.queue_state_change(GameState::Closing),
                    _ => {}
                }
            }

            _ => {}
        }

        if changed {
            self.play_button.set_selected(self.selected_index == 0);
            // self.direct_button.set_selected(self.selected_index == 1);
            self.settings_button.set_selected(self.selected_index == 2);
            self.exit_button.set_selected(self.selected_index == 3);
        }

        true
    }
}

