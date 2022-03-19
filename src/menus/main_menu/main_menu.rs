use crate::prelude::*;
use super::MusicBox;
use super::menu_button::MainMenuButton;

const BUTTON_SIZE: Vector2 = Vector2::new(100.0, 50.0);
const Y_MARGIN: f64 = 20.0;
const Y_OFFSET: f64 = 10.0;

pub struct MainMenu {
    // index 0
    pub play_button: MainMenuButton,
    // index 1
    pub direct_button: MainMenuButton,
    // index 2
    pub settings_button: MainMenuButton,
    // index 3
    pub exit_button: MainMenuButton,

    visualization: MenuVisualization,
    background_game: Option<IngameManager>,

    selected_index: usize,
    menu_visible: bool,

    music_box: MusicBox,
}
impl MainMenu {
    pub fn new() -> MainMenu {
        let middle = Settings::window_size().x /2.0 - BUTTON_SIZE.x/2.0;
        let mut counter = 1.0;
        
        let mut play_button = MainMenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Play");
        counter += 1.0;
        let mut direct_button = MainMenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "osu!Direct");
        counter += 1.0;
        let mut settings_button = MainMenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Settings");
        counter += 1.0;
        let mut exit_button = MainMenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Exit");

        play_button.visible = false;
        direct_button.visible = false;
        settings_button.visible = false;
        exit_button.visible = false;

        MainMenu {
            play_button,
            direct_button,
            settings_button,
            exit_button,

            visualization: MenuVisualization::new(),
            background_game: None,
            selected_index: 99,
            menu_visible: false,
            music_box: MusicBox::new()
        }
    }

    fn setup_manager(&mut self, called_by: &str) {
        println!("setup manager called by {}", called_by);

        let settings = &get_settings!().background_game_settings;
        if !settings.enabled {return}

        let lock = BEATMAP_MANAGER.read();
        let map = match &lock.current_beatmap {
            Some(map) => map,
            None => return println!("manager no map")
        };

        match manager_from_playmode(settings.mode.clone(), &map) {
            Ok(mut manager) => {
                manager.current_mods = Arc::new(ModManager {
                    autoplay: true,
                    ..Default::default()
                });
                manager.menu_background = true;
                manager.start();
                println!("manager started");

                self.background_game = Some(manager);
                self.visualization.song_changed(&mut self.background_game);
            },
            Err(e) => {
                self.visualization.song_changed(&mut None);
                NotificationManager::add_error_notification("Error loading beatmap", e);
            }
        }
        println!("manager setup");
    }

    fn show_menu(&mut self) {
        self.play_button.show(0, 4);
        self.direct_button.show(1, 4);
        self.settings_button.show(2, 4);
        self.exit_button.show(3, 4);
        self.menu_visible = true;
    }

    fn interactables(&mut self, include_buttons: bool) -> Vec<&mut dyn ScrollableItem> {
        if include_buttons {
            vec![
                &mut self.music_box,
                &mut self.play_button,
                &mut self.direct_button,
                &mut self.settings_button,
                &mut self.exit_button,
            ]
        } else {
            vec![
                &mut self.music_box,
            ]
        }
    }

    fn next(&mut self, game: &mut Game) -> bool {
        let mut manager = BEATMAP_MANAGER.write();

        if manager.next_beatmap(game) {
            true
        } else {
            println!("no next");
            false
        }
    }
    fn previous(&mut self, game: &mut Game) -> bool {
        let mut manager = BEATMAP_MANAGER.write();

        if manager.previous_beatmap(game) {
            true
        } else {
            println!("no prev");
            false
        }
    }
}
impl Menu<Game> for MainMenu {
    fn get_name(&self) -> &str {"main_menu"}

    fn on_change(&mut self, _into:bool) {
        self.visualization.reset();

        self.setup_manager("on_change");
    }

    fn update(&mut self, g:&mut Game) {
        let mut song_done = false;

        // run updates on the interactables
        for i in self.interactables(true) {
            i.update()
        }

        #[cfg(feature = "bass_audio")]
        match Audio::get_song() {
            Some(song) => {
                match song.get_playback_state() {
                    Ok(PlaybackState::Playing) | Ok(PlaybackState::Paused) => {},
                    _ => song_done = true,
                }
            }
            _ => song_done = true,
        }
        #[cfg(feature = "neb_audio")]
        if let None = Audio::get_song() {
            song_done = true;
        }

        if song_done {
            println!("song done");
            let map = BEATMAP_MANAGER.read().random_beatmap();

            // it should?
            if let Some(map) = map {
                BEATMAP_MANAGER.write().set_current_beatmap(g, &map, false, false);
                self.setup_manager("update song done");
            }
        }

        let maps = BEATMAP_MANAGER.write().get_new_maps();
        if maps.len() > 0 {
            BEATMAP_MANAGER.write().set_current_beatmap(g, &maps[maps.len() - 1], true, false);
            self.setup_manager("update new map");
        }

        self.visualization.update(&mut self.background_game);

        if let Some(manager) = self.background_game.as_mut() {
            manager.update();

            if manager.completed {
                self.background_game = None;
            }
        }
    }

    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let pos_offset = Vector2::zero();
        let depth = 0.0;
        let window_size = Settings::window_size();

        // // draw welcome text
        // let mut welcome_text = Text::new(
        //     Color::BLACK,
        //     depth-1.0,
        //     pos_offset,
        //     40,
        //     "Welcome to Tataku!".to_owned(),
        //     get_font()
        // );
        // welcome_text.center_text(Rectangle::bounds_only(Vector2::new(0.0, 30.0), Vector2::new(window_size.x , 50.0)));
        
        // const TEXT_PAD:f64 = 5.0;
        // list.push(visibility_bg(
        //     welcome_text.initial_pos - Vector2::new(0.0, TEXT_PAD), 
        //     Vector2::new(welcome_text.measure_text().x , 50.0),
        //     depth+10.0
        // ));
        // list.push(Box::new(welcome_text));

        // draw interactables
        for i in self.interactables(true) {
            i.draw(args, pos_offset, depth, &mut list)
        }

        // visualization
        let mid = window_size / 2.0;
        self.visualization.draw(args, mid, depth + 10.0, &mut list);

        if let Some(manager) = self.background_game.as_mut() {
            manager.draw(args, &mut list);
        }
        
        // draw dim
        list.push(Box::new(Rectangle::new(
            Color::BLACK.alpha(0.5),
            depth + 11.0,
            Vector2::zero(),
            Settings::window_size(),
            None
        )));

        list
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        if self.visualization.on_click(pos) {
            self.show_menu();
        }

        // switch to beatmap selection
        if self.play_button.on_click(pos, button, mods) {
            let menu = game.menus.get("beatmap").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }

        // open direct menu
        if self.direct_button.on_click(pos, button, mods) {
            let mode = get_settings!().background_game_settings.mode.clone();
            let menu:Arc<Mutex<dyn ControllerInputMenu<Game>>> = Arc::new(Mutex::new(DirectMenu::new(mode)));
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }

        // open settings menu
        if self.settings_button.on_click(pos, button, mods) {
            let menu = game.menus.get("settings").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
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

        if self.music_box.get_next_pending() {
            self.next(game);
            self.setup_manager("on_click next_pending")
        }
        if self.music_box.get_prev_pending() {
            self.previous(game);
            self.setup_manager("on_click prev_pending")
        }

    }

    fn on_mouse_move(&mut self, pos:Vector2, _game: &mut Game) {
        for i in self.interactables(true) {
            i.on_mouse_move(pos)
        }
    }

    fn on_key_press(&mut self, key:piston::Key, game:&mut Game, mods:KeyModifiers) {
        use piston::Key::*;

        let mut needs_manager_setup = false;

        if mods.ctrl && key == Key::N {
            NotificationManager::add_text_notification("test notif", 4000.0, Color::CRYSTAL_BLUE);
        }

        // check offset keys
        if let Some(manager) = self.background_game.as_mut() {
            manager.key_down(key, mods);
        }

        if !mods.alt {
            match key {
                Left => needs_manager_setup |= self.previous(game),
                Right => needs_manager_setup |= self.next(game),
                _ => {}
            }
        }
        
        if mods.alt {
            let new_mode = match key {
                D1 => Some("osu".to_owned()),
                D2 => Some("taiko".to_owned()),
                D3 => Some("catch".to_owned()),
                D4 => Some("mania".to_owned()),
                _ => None
            };

            if let Some(new_mode) = new_mode {
                let mut settings = get_settings_mut!();
                if settings.background_game_settings.mode != new_mode {
                    NotificationManager::add_text_notification(&format!("Menu mode changed to {:?}", new_mode), 1000.0, Color::BLUE);
                    needs_manager_setup = true;
                    settings.background_game_settings.mode = new_mode;
                }
            }
        }

        if needs_manager_setup {
            self.setup_manager("key press");
        }

    }
}
impl ControllerInputMenu<Game> for MainMenu {
    fn controller_down(&mut self, game:&mut Game, controller: &Box<dyn Controller>, button: u8) -> bool {
        if !self.menu_visible {
            if let Some(ControllerButton::A) = controller.map_button(button) {
                self.show_menu();
                return true;
            }
            return false;
        }

        let mut changed = false;
        if let Some(ControllerButton::DPad_Down) = controller.map_button(button) {
            self.selected_index += 1;
            if self.selected_index >= 4 {
                self.selected_index = 0;
            }

            changed = true;
        }

        if let Some(ControllerButton::DPad_Up) = controller.map_button(button) {
            if self.selected_index == 0 {
                self.selected_index = 3;
            } else if self.selected_index >= 4 { // original value is 99
                self.selected_index = 0;
            } else {
                self.selected_index -= 1;
            }

            changed = true;
        }

        if changed {
            self.play_button.set_selected(self.selected_index == 0);
            self.direct_button.set_selected(self.selected_index == 1);
            self.settings_button.set_selected(self.selected_index == 2);
            self.exit_button.set_selected(self.selected_index == 3);
        }

        if let Some(ControllerButton::A) = controller.map_button(button) {
            match self.selected_index {
                0 => {
                    let menu = game.menus.get("beatmap").unwrap().clone();
                    game.queue_state_change(GameState::InMenu(menu));
                },
                1 => {
                    let mode = get_settings!().background_game_settings.mode.clone();
                    let menu:Arc<Mutex<dyn ControllerInputMenu<Game>>> = Arc::new(Mutex::new(DirectMenu::new(mode)));
                    game.queue_state_change(GameState::InMenu(menu));
                },
                2 => {
                    let menu = game.menus.get("settings").unwrap().clone();
                    game.queue_state_change(GameState::InMenu(menu));
                },
                3 => game.queue_state_change(GameState::Closing),
                _ => {}
            }
        }

        true
    }
}

