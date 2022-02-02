use crate::prelude::*;

use crate::visualization::{MenuVisualization, Visualization};

const BUTTON_SIZE: Vector2 = Vector2::new(100.0, 50.0);
const Y_MARGIN: f64 = 20.0;
const Y_OFFSET: f64 = 10.0;

pub struct MainMenu {
    // index 0
    pub play_button: MenuButton,
    // index 1
    pub direct_button: MenuButton,
    // index 2
    pub settings_button: MenuButton,
    // index 3
    pub exit_button: MenuButton,

    visualization: MenuVisualization,
    background_game: Option<IngameManager>,

    selected_index: usize,
    menu_visible: bool
}
impl MainMenu {
    pub fn new() -> MainMenu {
        let middle = Settings::window_size().x /2.0 - BUTTON_SIZE.x/2.0;
        let mut counter = 1.0;
        
        let mut play_button = MenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Play");
        counter += 1.0;
        let mut direct_button = MenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "osu!Direct");
        counter += 1.0;
        let mut settings_button = MenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Settings");
        counter += 1.0;
        let mut exit_button = MenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Exit");

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
            menu_visible: false
        }
    }

    fn setup_manager(&mut self, called_by: &str) {
        println!("setup manager called by {}", called_by);

        let settings = Settings::get().background_game_settings;
        if !settings.enabled {return}

        let lock = BEATMAP_MANAGER.lock();
        let map = match &lock.current_beatmap {
            Some(map) => map,
            None => return println!("manager no map")
        };

        match manager_from_playmode(settings.mode, &map) {
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
}
impl Menu<Game> for MainMenu {
    fn on_change(&mut self, _into:bool) {
        self.visualization.reset();

        self.setup_manager("on_change");
    }

    fn update(&mut self, g:&mut Game) {
        let mut song_done = false;

        // run updates on the transforms
        self.play_button.update();
        self.direct_button.update();
        self.settings_button.update();
        self.exit_button.update();


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
            let map = BEATMAP_MANAGER.lock().random_beatmap();

            // it should?
            if let Some(map) = map {
                BEATMAP_MANAGER.lock().set_current_beatmap(g, &map, false, false);
                self.setup_manager("update song done");
            }
        }

        let maps = BEATMAP_MANAGER.lock().get_new_maps();
        if maps.len() > 0 {
            BEATMAP_MANAGER.lock().set_current_beatmap(g, &maps[maps.len() - 1], true, false);
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

        // draw welcome text
        let mut welcome_text = Text::new(
            Color::BLACK,
            depth-1.0,
            pos_offset,
            40,
            "Welcome to Tataku!".to_owned(),
            get_font("main")
        );
        welcome_text.center_text(Rectangle::bounds_only(Vector2::new(0.0, 30.0), Vector2::new(window_size.x , 50.0)));
        
        const TEXT_PAD:f64 = 5.0;
        list.push(visibility_bg(
            welcome_text.initial_pos - Vector2::new(0.0, TEXT_PAD), 
            Vector2::new(welcome_text.measure_text().x , 50.0),
            depth+10.0
        ));
        list.push(Box::new(welcome_text));

        // draw buttons
        self.play_button.draw(args, pos_offset, depth, &mut list);
        self.direct_button.draw(args, pos_offset, depth, &mut list);
        self.settings_button.draw(args, pos_offset, depth, &mut list);
        self.exit_button.draw(args, pos_offset, depth, &mut list);

        // visualization
        let mid = window_size / 2.0;
        self.visualization.draw(args, mid, depth + 10.0, &mut list);

        if let Some(manager) = self.background_game.as_mut() {
            manager.draw(args, &mut list);
        }

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
            let mode = Settings::get_mut("MainMenu::on_click").background_game_settings.mode;
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
    }

    fn on_mouse_move(&mut self, pos:Vector2, _game: &mut Game) {
        self.play_button.check_hover(pos);
        self.direct_button.check_hover(pos);
        self.settings_button.check_hover(pos);
        self.exit_button.check_hover(pos);
    }

    fn on_key_press(&mut self, key:piston::Key, game:&mut Game, mods:KeyModifiers) {
        use piston::Key::*;

        let mut needs_manager_setup = false;

        // check offset keys
        if let Some(manager) = self.background_game.as_mut() {
            manager.key_down(key, mods);
        }

        if !mods.alt {
            match key {
                Left => {
                    let mut manager = BEATMAP_MANAGER.lock();

                    if manager.previous_beatmap(game) {
                        needs_manager_setup = true;
                    } else {
                        println!("no prev")
                    }
                }
                Right => {
                    let mut manager = BEATMAP_MANAGER.lock();

                    if manager.next_beatmap(game) {
                        needs_manager_setup = true;
                    } else {
                        println!("no next")
                    }
                }

                _ => {}
            }
        }
        
        if mods.alt {
            let new_mode = match key {
                D1 => Some(PlayMode::Standard),
                D2 => Some(PlayMode::Taiko),
                D3 => Some(PlayMode::Catch),
                D4 => Some(PlayMode::Mania),
                _ => None
            };

            if let Some(new_mode) = new_mode {
                let mut settings = Settings::get_mut("MainMenu::on_key_press");
                if settings.background_game_settings.mode != new_mode {
                    needs_manager_setup = true;
                    settings.background_game_settings.mode = new_mode;
                    NotificationManager::add_text_notification(&format!("Menu mode changed to {:?}", new_mode), 1000.0, Color::BLUE);
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
                    let mode = Settings::get_mut("MainMenu::on_click").background_game_settings.mode;
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


#[allow(dead_code)]
#[derive(Clone)]
pub struct MenuButton {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    selected: bool,
    text: String,

    shapes: TransformGroup,
    disposable_shapes: Vec<TransformGroup>,
    visible: bool,
    timer: Instant
}
impl MenuButton {
    pub fn new(_pos: Vector2, size: Vector2, text:&str) -> MenuButton {
        let font_size: u32 = 12;
        let pos = Vector2::zero();
        
        // draw box
        let r = Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            1.0,
            pos,
            size,
            Some(Border::new(Color::RED, 0.0))
        );
        
        // draw text
        let mut txt = Text::new(
            Color::WHITE,
            0.0,
            Vector2::zero(),
            font_size,
            text.to_owned(),
            get_font("main")
        );
        txt.center_text(r);


        let mut shapes = TransformGroup::new();
        shapes.items.push(DrawItem::Rectangle(r));
        shapes.items.push(DrawItem::Text(txt));

        MenuButton {
            pos, 
            size, 
            text: text.to_owned(),

            hover: false,
            selected: false,

            shapes,
            disposable_shapes: Vec::new(),
            visible: true,
            timer: Instant::now()
        }
    }

    /// num: this button number, count: number of buttons
    pub fn show(&mut self, num: usize, count: usize) {
        if self.visible {return}
        self.visible = true;
        let time = self.time();


        const X_OFFSET:f64 = 10.0;
        let radius = crate::visualization::initial_radius() * crate::visualization::SIZE_FACTOR + X_OFFSET;
        let center = Settings::window_size() / 2.0;

        const ITEM_PADDING:usize = 2;

        let height = self.size.y;
        let angle = (PI / (count + 2 * ITEM_PADDING - 1) as f64) * (num + ITEM_PADDING) as f64 - PI / 2.0;


        let end = center + Vector2::new(
            angle.cos() * radius,
            angle.sin() * radius,
        ) - Vector2::new(0.0, height / 2.0);

        let start = Vector2::new(
            center.x,
            end.y
        );

        let t1 = Transformation::new(
            0.0,
            500.0,
            TransformType::Position {start, end},
            TransformEasing::Linear,
            time
        );




        // let transform = Transformation::new(
        //     0.0,
        //     500.0,
        //     TransformType::Transparency {start: 0.0, end: 1.0},
        //     TransformEasing::EaseInSine,
        //     time
        // );

        // let transform2 = Transformation::new(
        //     0.0,
        //     500.0,
        //     TransformType::Rotation {start: 0.0, end: PI * 2.0},
        //     TransformEasing::Linear,
        //     time
        // );

        // let transform3 = Transformation::new(
        //     500.0,
        //     500.0,
        //     TransformType::Scale {start: 1.0, end: 3.0},
        //     TransformEasing::Linear,
        //     time
        // );

        self.shapes.transforms.push(t1);
        // self.shapes.transforms.push(transform);
        // self.shapes.transforms.push(transform2);
        // self.shapes.transforms.push(transform3);
        for i in self.disposable_shapes.iter_mut() {
            i.transforms.push(t1);
            // i.transforms.push(transform);
            // i.transforms.push(transform2);
            // i.transforms.push(transform3);
        }
    }

    pub fn time(&self) -> f64 {
        self.timer.elapsed().as_secs_f64() * 1000.0
    }
}
impl ScrollableItem for MenuButton {
    fn size(&self) -> Vector2 {self.size}
    fn get_pos(&self) -> Vector2 {self.shapes.items[0].get_pos()}

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, mut hover:bool) {
        if !self.visible {hover = false}
        self.hover = hover;

        let size = if hover {
            1.0
        } else {
            0.0
        };

        let transform = Transformation::new(
            0.0, 
            1.0,
            TransformType::BorderSize {start: size, end: size},
            TransformEasing::Linear,
            self.time()
        );

        self.shapes.transforms.push(transform);
    }
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, mut selected:bool) {
        if !self.visible {selected = false}
        self.selected = selected;
        println!("setting selected: {}", selected);

        if selected {
            let transform2 = Transformation::new(
                0.0, 
                1.0,
                TransformType::BorderSize {start: 1.0, end: 1.0},
                TransformEasing::Linear,
                self.time()
            );
            let transform = Transformation::new(
                1.0, 
                1.0,
                TransformType::BorderColor {start: Color::BLUE, end: Color::BLUE},
                TransformEasing::Linear,
                self.time()
            );

            self.shapes.transforms.push(transform);
            self.shapes.transforms.push(transform2);
        } else {
            self.set_hover(self.hover)
        }
    }
    fn get_selectable(&self) -> bool {false}

    fn update(&mut self) {
        let time = self.timer.elapsed().as_secs_f64() * 1000.0;
        self.shapes.update(time);

        self.disposable_shapes.retain_mut(|i|{
            i.update(time);
            i.items.find(|s|s.visible()).is_some()
        });
    }

    fn draw(&mut self, _args:piston::RenderArgs, _pos_offset:Vector2, _parent_depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        if !self.visible {return}
        self.shapes.draw(list);

        for i in self.disposable_shapes.iter_mut() {
            i.draw(list);
        }
    }
}
