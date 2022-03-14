
use crate::prelude::*;
use crate::databases::get_scores;


// constants
const INFO_BAR_HEIGHT: f64 = 60.0;
const BEATMAPSET_ITEM_SIZE: Vector2 = Vector2::new(550.0, 50.0);
const BEATMAPSET_PAD_RIGHT: f64 = 5.0;

const BEATMAP_ITEM_PADDING: f64 = 5.0;
const BEATMAP_ITEM_SIZE: Vector2 = Vector2::new(450.0, 50.0);

const LEADERBOARD_PADDING: f64 = 100.0;
const LEADERBOARD_POS: Vector2 = Vector2::new(10.0, LEADERBOARD_PADDING);
const LEADERBOARD_ITEM_SIZE: Vector2 = Vector2::new(200.0, 50.0);

const RECT_ROUND:Shape = Shape::Round(5.0, 10);


pub struct BeatmapSelectMenu {
    current_scores: HashMap<String, Arc<Mutex<Score>>>,
    beatmap_scroll: ScrollableArea,
    leaderboard_scroll: ScrollableArea,
    back_button: MenuButton,
    // pending_refresh: bool,

    /// is changing, update loop detected that it was changing, how long since it changed
    map_changing: (bool, bool, u32),

    // drag: Option<DragData>,
    // mouse_down: bool

    /// internal search box
    search_text: TextInput,
    no_maps_notif_sent: bool,

    sort_method: SortBy,
    mode: PlayMode,

    sort_by_dropdown: Dropdown<SortBy>,
    playmode_dropdown: Dropdown<PlayModeDropdown>,
}
impl BeatmapSelectMenu {
    pub fn new() -> BeatmapSelectMenu {
        let window_size = Settings::window_size();
        let font = get_font();

        let sort_by = SortBy::Title;
        let sort_by_dropdown = Dropdown::new(
            Vector2::new(0.0, 5.0),
            200.0,
            15,
            "Sort",
            Some(sort_by),
            font.clone()
        );

        let mode = get_settings!().last_played_mode.clone();
        let playmode_dropdown = Dropdown::new(
            Vector2::new(200.0, 5.0),
            200.0,
            15,
            "Mode",
            Some(PlayModeDropdown::Mode(mode.clone())),
            font.clone()
        );
        

        // // update diffs
        // // TODO: this shouldnt lock for too long, but theres probably a better place to put it
        // let mode_clone = mode.clone();
        // tokio::spawn(async {
        //     BEATMAP_MANAGER.write().update_diffs(mode_clone, &ModManager::get());
        // });
        

        BeatmapSelectMenu {
            no_maps_notif_sent: false,

            // mouse_down: false,
            // drag: None,

            // pending_refresh: false,
            map_changing: (false, false, 0),
            current_scores: HashMap::new(),
            back_button: MenuButton::back_button(window_size, font.clone()),

            // beatmap_scroll: ScrollableArea::new(Vector2::new(window_size().x - (BEATMAPSET_ITEM_SIZE.x + BEATMAPSET_PAD_RIGHT), INFO_BAR_HEIGHT), Vector2::new(window_size().x - LEADERBOARD_ITEM_SIZE.x, window_size().y - INFO_BAR_HEIGHT), true),
            beatmap_scroll: ScrollableArea::new(Vector2::new(LEADERBOARD_POS.x + LEADERBOARD_ITEM_SIZE.x, INFO_BAR_HEIGHT), Vector2::new(window_size.x - LEADERBOARD_ITEM_SIZE.x, window_size.y - INFO_BAR_HEIGHT), true),
            leaderboard_scroll: ScrollableArea::new(LEADERBOARD_POS, Vector2::new(LEADERBOARD_ITEM_SIZE.x, window_size.y - (LEADERBOARD_PADDING + INFO_BAR_HEIGHT)), true),
            search_text: TextInput::new(Vector2::new(window_size.x - (window_size.x / 4.0), 0.0), Vector2::new(window_size.x / 4.0, INFO_BAR_HEIGHT), "Search", "", font.clone()),

            mode: mode.clone(),
            sort_method: sort_by,

            playmode_dropdown,
            sort_by_dropdown,
        }
    }

    fn set_selected_mode(&mut self, new_mode: PlayMode, game: Option<&mut Game>) {
        // update values
        self.mode = new_mode.clone();
        self.playmode_dropdown.value = Some(PlayModeDropdown::Mode(new_mode.clone()));
        get_settings_mut!().last_played_mode = new_mode.clone();

        // recalc diffs
        BEATMAP_MANAGER.write().update_diffs(new_mode.clone(), &ModManager::get());
        
        // set modes and update diffs
        self.beatmap_scroll.on_text(new_mode.clone());
        if let Some(game) = game {
            self.on_key_press(Key::Calculator, game, KeyModifiers::default());
        }
    }

    pub fn refresh_maps(&mut self, beatmap_manager:&mut BeatmapManager) {
        let filter_text = self.search_text.get_text().to_ascii_lowercase();
        self.beatmap_scroll.clear();

        // used to select the current map in the list
        let current_hash = if let Some(map) = &beatmap_manager.current_beatmap {map.beatmap_hash.clone()} else {String::new()};

        let sets = beatmap_manager.all_by_sets();
        let mut full_list = Vec::new();

        for mut maps in sets {
            if !filter_text.is_empty() {
                maps.retain(|bm|bm.filter(&filter_text));
                if maps.len() == 0 {continue}
            }

            let mut i = BeatmapsetItem::new(maps, self.mode.clone());
            i.check_selected(&current_hash);
            full_list.push(Box::new(i));
        }

        macro_rules! sort {
            ($property:tt, String) => {
                full_list.sort_by(|a, b| a.beatmaps[0].$property.to_lowercase().cmp(&b.beatmaps[0].$property.to_lowercase()))
            };
            ($property:ident, Float) => {
                full_list.sort_by(|a, b| a.beatmaps[0].$property.partial_cmp(&b.beatmaps[0].$property).unwrap())
            }
        }

        match self.sort_method {
            SortBy::Title => sort!(title, String),
            SortBy::Artist => sort!(artist, String),
            SortBy::Creator => sort!(creator, String),
            SortBy::Difficulty => sort!(diff, Float),
        }

        // sort by artist
        // full_list.sort_by(|a, b| a.beatmaps[0].artist.to_lowercase().cmp(&b.beatmaps[0].artist.to_lowercase()));
        for i in full_list {self.beatmap_scroll.add_item(i)}

        self.beatmap_scroll.scroll_to_selection();
    }

    pub fn load_scores(&mut self) {
        // if nothing is selected, leave
        if let Some(map) = &BEATMAP_MANAGER.read().current_beatmap {

            // clear lists
            self.leaderboard_scroll.clear();
            self.current_scores.clear();

            // load scores
            let mut scores = get_scores(&map.beatmap_hash, map.check_mode_override(self.mode.clone()));
            scores.sort_by(|a, b| b.score.cmp(&a.score));

            // add scores to list
            for s in scores.iter() {
                self.current_scores.insert(s.username.clone(), Arc::new(Mutex::new(s.clone())));
                self.leaderboard_scroll.add_item(Box::new(LeaderboardItem::new(s.to_owned())));
            }
        }
    }

    fn play_map(&self, game: &mut Game, map: &BeatmapMeta) {
        // Audio::stop_song();
        match manager_from_playmode(self.mode.clone(), map) {
            Ok(manager) => game.queue_state_change(GameState::Ingame(manager)),
            Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e)
        }
    }

    fn select_map(&mut self, game: &mut Game, map: String, can_start: bool) {
        let mut lock = BEATMAP_MANAGER.write();

        // compare last clicked map hash with the new hash.
        // if the hashes are the same, the same map was clicked twice in a row.
        // play it
        if let Some(current) = &lock.current_beatmap {
            if current.beatmap_hash == map && can_start {
                self.play_map(game, current);
                self.map_changing = (true, false, 0);
                return;
            }
        }

        // set the current map to the clicked
        self.map_changing = (true, false, 0);
        match lock.get_by_hash(&map) {
            Some(clicked) => {
                lock.set_current_beatmap(game, &clicked, true, true);
            }
            None => {
                println!("no map?");
                // map was deleted?
                return
            }
        }
        drop(lock);

        self.beatmap_scroll.refresh_layout();
        self.load_scores();
    }

    fn interactables(&mut self) -> Vec<&mut dyn ScrollableItem> {
        vec![
            &mut self.playmode_dropdown,
            &mut self.sort_by_dropdown,
            &mut self.search_text
        ]
    }
}
impl Menu<Game> for BeatmapSelectMenu {
    fn update(&mut self, game:&mut Game) {
        self.search_text.set_selected(true); // always have it selected
        let old_text = self.search_text.get_text();

        for i in self.interactables() {
            i.update();
        }

        if old_text != self.search_text.get_text() {
            self.refresh_maps(&mut BEATMAP_MANAGER.write());
        }

        {
            let mut lock = BEATMAP_MANAGER.write();
            let maps = lock.get_new_maps();
            if maps.len() > 0  {
                lock.set_current_beatmap(game, &maps[maps.len() - 1], false, true);
                self.refresh_maps(&mut lock);
            }
            if lock.force_beatmap_list_refresh {
                lock.force_beatmap_list_refresh = false;
                self.refresh_maps(&mut lock);
            }
        }

    
        #[cfg(feature="bass_audio")]
        match Audio::get_song() {
            Some(song) => {
                match song.get_playback_state() {
                    Ok(PlaybackState::Playing) => {},
                    _ => {
                        // restart the song at the preview point

                        let lock = BEATMAP_MANAGER.read();
                        let map = lock.current_beatmap.as_ref().unwrap();
                        if let Err(e) = song.set_position(map.audio_preview as f64) {
                            println!("error setting position: {:?}", e);
                        }
                        song.play(false).unwrap();
                    },
                }
            }

            // no value, set it to something
            _ => {
                let lock = BEATMAP_MANAGER.read();
                match &lock.current_beatmap {
                    Some(map) => {
                        Audio::play_song(map.audio_filename.clone(), true, map.audio_preview).unwrap();
                    }
                    None => if !self.no_maps_notif_sent {
                        NotificationManager::add_text_notification("No beatmaps\nHold on...", 5000.0, Color::GREEN);
                        self.no_maps_notif_sent = true;
                    }
                }
            },
        }

        #[cfg(feature="neb_audio")] {
            self.map_changing.2 += 1;
            match self.map_changing {
                // we know its changing but havent detected the previous song stop yet
                (true, false, n) => {
                    // give it up to 1s before assuming its already loaded
                    if Audio::get_song().is_none() || n > 1000 {
                        // println!("song loading");
                        self.map_changing = (true, true, 0);
                    }
                }
                // we know its changing, and the previous song has ended
                (true, true, _) => {
                    if Audio::get_song().is_some() {
                        // println!("song loaded");
                        self.map_changing = (false, false, 0);
                    }
                }
    
                // the song hasnt ended and we arent changing
                (false, false, _) | (false, true, _) => {
                    if Audio::get_song().is_none() {
                        // println!("song done");
                        self.map_changing = (true, false, 0);
                        tokio::spawn(async move {
                            let lock = BEATMAP_MANAGER.lock();
                            let map = lock.current_beatmap.as_ref().unwrap();
                            Audio::play_song(map.audio_filename.clone(), true, map.audio_preview);
                        });
                    }
                }
            }
    
        }

        // old audio shenanigans
        /*
        // self.map_changing.2 += 1;
        // let mut song_done = false;
        // match Audio::get_song() {
        //     Some(song) => {
        //         match song.get_playback_state() {
        //             Ok(PlaybackState::Playing) | Ok(PlaybackState::Paused) => {},
        //             _ => song_done = true,
        //         }
        //     }
        //     _ => song_done = true,
        // }
        // i wonder if this can be simplified now
        // match self.map_changing {
        //     // we know its changing but havent detected the previous song stop yet
        //     (true, false, n) => {
        //         // give it up to 1s before assuming its already loaded
        //         if song_done || n > 1000 {
        //             // println!("song loading");
        //             self.map_changing = (true, true, 0);
        //         }
        //     }
        //     // we know its changing, and the previous song has ended
        //     (true, true, _) => {
        //         if !song_done {
        //             // println!("song loaded");
        //             self.map_changing = (false, false, 0);
        //         }
        //     }

        //     // the song hasnt ended and we arent changing
        //     (false, false, _) | (false, true, _) => {
        //         if song_done {
        //             // println!("song done");
        //             self.map_changing = (true, false, 0);
        //             tokio::spawn(async move {
        //                 let lock = BEATMAP_MANAGER.lock();
        //                 let map = lock.current_beatmap.as_ref().unwrap();
        //                 Audio::play_song(map.audio_filename.clone(), true, map.audio_preview).unwrap();
        //             });
        //         }
        //     }
        // }


        // if self.mouse_down {

        // } else {
        //     if self.drag.is_some() {
        //         let data = self.drag.as_ref().unwrap();

        //     }
        // }

        // if game.input_manager.mouse_buttons.contains(&MouseButton::Left) && game.input_manager.mouse_moved {
        //     if self.drag.is_none() {
        //         self.drag = Some(DragData {
        //             start_pos: game.input_manager.mouse_pos.y,
        //             current_pos: game.input_manager.mouse_pos.y,
        //             start_time: Instant::now()
        //         });
        //     }

        //     if let Some(data) = self.drag.as_mut() {
        //         data.current_pos = game.input_manager.mouse_pos.y
        //     }
        // }

        */
    }

    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut items: Vec<Box<dyn Renderable>> = Vec::new();
        // let mut counter: usize = 0;
        let depth: f64 = 5.0;
        // let font = get_font();

        // draw a bar on the top for the info
        let bar_rect = Rectangle::new(
            Color::WHITE,
            depth - 1.0,
            Vector2::zero(),
            Vector2::new(args.window_size[0], INFO_BAR_HEIGHT),
            Some(Border::new(Color::BLACK, 1.2))
        );
        items.push(Box::new(bar_rect));

        // // draw selected map info
        // if let Some(meta) = &mut BEATMAP_MANAGER.write().current_beatmap {
        //     // draw map name top-most left-most
        //     items.push(Box::new(Text::new(
        //         Color::BLACK,
        //         -10.0,
        //         Vector2::new(0.0, 5.0),
        //         25,
        //         meta.version_string(),
        //         font.clone()
        //     )));

        //     // diff string, under map string
        //     items.push(Box::new(Text::new(
        //         Color::BLACK,
        //         -10.0,
        //         Vector2::new(0.0, 35.0),
        //         15,
        //         meta.diff_string(self.mode.clone(), &ModManager::get()),
        //         font.clone()
        //     )));
        // }

        // beatmap scroll
        self.beatmap_scroll.draw(args, Vector2::zero(), 0.0, &mut items);

        // leaderboard scroll
        self.leaderboard_scroll.draw(args, Vector2::zero(), 0.0, &mut items);

        // back button
        self.back_button.draw(args, Vector2::zero(), 0.0, &mut items);

        // everything else
        for i in self.interactables() {
            i.draw(args, Vector2::zero(), 0.0, &mut items);
        }

        items
    }

    fn on_change(&mut self, into:bool) {
        if !into {return}

        OnlineManager::send_spec_frames(vec![(0.0, SpectatorFrameData::ChangingMap)], true);

        // play song if it exists
        if let Some(song) = Audio::get_song() {
            // reset any time mods

            #[cfg(feature="bass_audio")]
            song.set_rate(1.0).unwrap();
            #[cfg(feature="neb_audio")]
            song.set_playback_speed(1.0);
            // // play
            // song.play(true).unwrap();
        }

        // load maps
        self.refresh_maps(&mut BEATMAP_MANAGER.write());
        self.beatmap_scroll.refresh_layout();

        if BEATMAP_MANAGER.read().current_beatmap.is_some() {
            self.load_scores();
        }
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, mods: ayyeve_piston_ui::menu::KeyModifiers, game:&mut Game) {
        if self.back_button.on_click(pos, button, mods) {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }

        for i in self.interactables() {
            if i.on_click(pos, button, mods) {
                break;
            }
        }

        // check if selected mode changed
        let mut new_mode = None;
        if let Some(PlayModeDropdown::Mode(selected_mode)) = &self.playmode_dropdown.value {
            if selected_mode != &self.mode {
                new_mode = Some(selected_mode.clone())
            }
        }
        if let Some(new_mode) = new_mode {
            self.set_selected_mode(new_mode, Some(game))
        }

        let mut map_refresh = false;
        if let Some(sort_by) = &self.sort_by_dropdown.value {
            if sort_by != &self.sort_method {
                self.sort_method = sort_by.clone();
                map_refresh = true;
            }
        }
        if map_refresh {
            self.refresh_maps(&mut BEATMAP_MANAGER.write())
        }


        // check if leaderboard item was clicked
        if let Some(score_tag) = self.leaderboard_scroll.on_click_tagged(pos, button, mods) {
            // score display
            if let Some(score) = self.current_scores.get(&score_tag) {
                let score = score.lock().clone();

                if let Some(selected) = &BEATMAP_MANAGER.read().current_beatmap {
                    let menu = ScoreMenu::new(&score, selected.clone());
                    game.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(menu))));
                }
            }
            return;
        }


        // find the previously selected item
        let mut selected_index = None;
        for (i, item) in self.beatmap_scroll.items.iter().enumerate() {
            if item.get_selected() {
                selected_index = Some(i);
                break;
            }
        }

        // check if beatmap item was clicked
        if let Some(clicked_hash) = self.beatmap_scroll.on_click_tagged(pos, button, mods) {
            if button == MouseButton::Right {
                // clicked hash is the target
                let dialog = BeatmapDialog::new(clicked_hash.clone());
                game.add_dialog(Box::new(dialog));
            }

            self.select_map(game, clicked_hash, button == MouseButton::Left);
            return;
        }

        // if we got here, make sure a map is selected
        // TODO: can we do this a better way? probably not since individually each item wont know if it should deselect or not
        if let Some(i) = selected_index {
            if let Some(item) = self.beatmap_scroll.items.get_mut(i) {
                item.set_selected(true);
            }
            self.beatmap_scroll.refresh_layout();
        }
        
        // else {
        //     //TODO: hmm
        //     self.selected = None;
        //     self.beatmap_scroll.refresh_layout();
        //     self.leaderboard_scroll.clear();
        // }

        // self.beatmap_scroll.refresh_layout();
    }
    fn on_click_release(&mut self, pos:Vector2, button:MouseButton, _game:&mut Game) {
        for i in self.interactables() {
            i.on_click_release(pos, button) 
        }
    }
    
    fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        for i in self.interactables() {
            i.on_mouse_move(pos) 
        }
        self.back_button.on_mouse_move(pos);
        self.beatmap_scroll.on_mouse_move(pos);
        self.leaderboard_scroll.on_mouse_move(pos);
    }
    fn on_scroll(&mut self, delta:f64, _game:&mut Game) {
        self.beatmap_scroll.on_scroll(delta);
        self.leaderboard_scroll.on_scroll(delta);

        for i in self.interactables() {
            i.on_scroll(delta);
        }
    }

    fn on_key_press(&mut self, key:piston::Key, game:&mut Game, mods:KeyModifiers) {
        use piston::Key::*;

        if key == Left {
            if let Some(hash) = self.beatmap_scroll.select_previous_item() {
                self.select_map(game, hash, false);
            }
        }
        if key == Right {
            if let Some(hash) = self.beatmap_scroll.select_next_item() {
                self.select_map(game, hash, false);
            }
        }


        if key == Escape {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }
        if key == F5 {
            if mods.ctrl {
                NotificationManager::add_text_notification("doing a full refresh", 5000.0, Color::RED);
                BEATMAP_MANAGER.write().full_refresh();
            } else {
                self.refresh_maps(&mut BEATMAP_MANAGER.write());
            }
            return;
        }

        // mode change
        if mods.alt {
            let new_mode = match key {
                D1 => Some("osu".to_owned()),
                D2 => Some("taiko".to_owned()),
                // D3 => Some("catch".to_owned()),
                D4 => Some("mania".to_owned()),
                _ => None
            };

            if let Some(new_mode) = new_mode {
                self.set_selected_mode(new_mode.clone(), Some(game));
                NotificationManager::add_text_notification(&format!("Mode changed to {:?}", new_mode), 1000.0, Color::BLUE);
                self.mode = new_mode;
                self.load_scores();
            }
        }

        // mods and speed
        if mods.ctrl {
            let mut speed = ModManager::get().speed;
            let prev_speed = speed;
            const SPEED_DIFF:f32 = 0.1;

            match key {
                Equals => speed += SPEED_DIFF, // map speed up
                Minus => speed -= SPEED_DIFF, // map speed down
                 
                // autoplay enable/disable
                A => {
                    let mut manager = ModManager::get();
                    manager.autoplay = !manager.autoplay;

                    let state = if manager.autoplay {"on"} else {"off"};
                    NotificationManager::add_text_notification(&format!("Autoplay {}", state), 2000.0, Color::BLUE);
                }

                // nofail enable/disable
                N => {
                    let mut manager = ModManager::get();
                    manager.nofail = !manager.nofail;

                    let state = if manager.nofail {"on"} else {"off"};
                    NotificationManager::add_text_notification(&format!("Nofail {}", state), 2000.0, Color::BLUE);
                }

                _ => {}
            }

            speed = speed.clamp(SPEED_DIFF, 10.0);
            if speed != prev_speed {
                ModManager::get().speed = speed;

                // force diff recalc
                self.set_selected_mode(self.mode.clone(), Some(game));

                NotificationManager::add_text_notification(&format!("Map speed: {:.2}x", speed), 2000.0, Color::BLUE);
            }
        }

        // if enter was hit, or a beatmap item was updated
        if self.beatmap_scroll.on_key_press(key, mods) || key == Return {
            if let Some(selected_index) = self.beatmap_scroll.get_selected_index() {
                if let Some(item) = self.beatmap_scroll.items.get(selected_index) {
                    let hash = item.get_tag();
                    self.select_map(game, hash, key == Return);
                }
            }
        }
        

        // only refresh if the text changed
        let old_text = self.search_text.get_text();

        for i in self.interactables() {
            i.on_key_press(key, mods);
        }

        if self.search_text.get_text() != old_text {
            self.refresh_maps(&mut BEATMAP_MANAGER.write());
        }
    }

    fn on_key_release(&mut self, key:piston::Key, _game:&mut Game) {
        for i in self.interactables() {
            i.on_key_release(key);
        }
    }

    fn on_text(&mut self, text:String) {
        // DO NOT ACTIVAT FOR BEATMAP ITEMS!!
        // on_text is used to change the playmode lol
        for i in self.interactables() {
            i.on_text(text.clone());
        }
        self.refresh_maps(&mut BEATMAP_MANAGER.write());
    }
}
impl ControllerInputMenu<Game> for BeatmapSelectMenu {
    fn controller_down(&mut self, game:&mut Game, controller: &Box<dyn Controller>, button: u8) -> bool {
        if let Some(ControllerButton::DPad_Up) = controller.map_button(button) {
            self.on_key_press(Key::Up, game, KeyModifiers::default())
        }
        if let Some(ControllerButton::DPad_Down) = controller.map_button(button) {
            self.on_key_press(Key::Down, game, KeyModifiers::default())
        }
        if let Some(ControllerButton::DPad_Left) = controller.map_button(button) {
            self.on_key_press(Key::Left, game, KeyModifiers::default())
        }
        if let Some(ControllerButton::DPad_Right) = controller.map_button(button) {
            self.on_key_press(Key::Right, game, KeyModifiers::default())
        }

        if let Some(ControllerButton::A) = controller.map_button(button) {
            self.on_key_press(Key::Return, game, KeyModifiers::default())
        }

        if let Some(ControllerButton::B) = controller.map_button(button) {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
        }
        
        false
    }

    fn controller_axis(&mut self, _game:&mut Game, controller: &Box<dyn Controller>, axis_data: HashMap<u8, (bool, f64)>) -> bool {
        for (axis, (_, val)) in axis_data {
            if Some(ControllerAxis::Right_Y) == controller.map_axis(axis) && val.abs() > 0.1 {
                self.beatmap_scroll.set_hover(true);
                self.beatmap_scroll.on_scroll(-val / 16.0);
            }
        }

        false
    }
}


struct BeatmapsetItem {
    pos: Vector2,
    hover: bool,
    selected: bool,
    
    beatmaps: Vec<BeatmapMeta>,
    selected_index: usize,
    mouse_pos: Vector2,
    playmode: String
}
impl BeatmapsetItem {
    fn new(mut beatmaps: Vec<BeatmapMeta>, playmode: PlayMode) -> BeatmapsetItem {
        // ensure diff is calced for all maps
        let maps_clone = beatmaps.clone();
        let playmode2 = playmode.clone();
        // tokio::spawn(async move {
        let mods = ModManager::get();
        maps_clone.iter().for_each(|b| {
            if let None = get_diff(&b.beatmap_hash, &playmode2, &mods) {
                let diff = calc_diff(b, playmode2.clone(), &mods).unwrap_or_default();
                insert_diff(&b.beatmap_hash, &playmode2, &mods, diff);
            }
        });
        // });
        
        beatmaps.sort_by(|a, b| a.diff.partial_cmp(&b.diff).unwrap());

        let x = Settings::window_size().x - (BEATMAPSET_ITEM_SIZE.x + BEATMAPSET_PAD_RIGHT + LEADERBOARD_POS.x + LEADERBOARD_ITEM_SIZE.x);

        BeatmapsetItem {
            beatmaps: beatmaps.clone(), 
            pos: Vector2::new(x, 0.0),
            hover: false,
            selected: false,
            // pending_play: false,
            // tag,

            selected_index: 0,
            mouse_pos: Vector2::zero(),
            playmode
        }
    }

    /// set the currently selected map
    fn check_selected(&mut self, current_hash: &String) -> bool {
        for i in 0..self.beatmaps.len() {
            if &self.beatmaps[i].beatmap_hash == current_hash {
                self.selected = true;
                self.selected_index = i;
                return true;
            }
        }

        false
    }
}
impl ScrollableItemGettersSetters for BeatmapsetItem {
    fn size(&self) -> Vector2 {
        if !self.selected {
            BEATMAPSET_ITEM_SIZE
        } else {
            Vector2::new(BEATMAPSET_ITEM_SIZE.x, (BEATMAPSET_ITEM_SIZE.y + BEATMAP_ITEM_PADDING) * (self.beatmaps.len()+1) as f64)
        }
    }
    fn get_tag(&self) -> String {self.beatmaps[self.selected_index].beatmap_hash.clone()}
    // fn set_tag(&mut self, _tag:&str) {self.pending_play = false} // bit of a jank strat: when this is called, reset the pending_play property
    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, selected:bool) {self.selected = selected}
}
impl ScrollableItem for BeatmapsetItem {
    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.beatmaps[self.selected_index].clone())}

    fn on_click(&mut self, pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> bool {
        if self.selected && self.hover {
            // find the clicked item
            // we only care about y pos, because we know we were clicked
            let rel_y2 = (pos.y - self.pos.y).abs() - BEATMAPSET_ITEM_SIZE.y;
            let index = (((rel_y2 + BEATMAP_ITEM_PADDING/2.0) / (BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING)).floor() as usize).clamp(0, self.beatmaps.len() - 1);

            self.selected_index = index;

            return true;
        }

        self.selected = self.hover;
        self.hover
    }
    fn on_mouse_move(&mut self, pos:Vector2) {
        self.mouse_pos = pos;
        self.check_hover(pos);
    }

    fn on_key_press(&mut self, key:Key, _mods:KeyModifiers) -> bool {
        // press this key if you want to recalculate things
        if key == Key::Calculator {
            let previous_selected = self.beatmaps[self.selected_index].beatmap_hash.clone();

            // get the diff values from the beatmap manager
            for i in self.beatmaps.iter_mut() {
                i.diff = BEATMAP_MANAGER.read().get_by_hash(&i.beatmap_hash).unwrap().diff;
            }
            self.beatmaps.sort_by(|a, b| a.diff.partial_cmp(&b.diff).unwrap());

            // reselect the proper index
            for (i, map) in self.beatmaps.iter().enumerate() {
                if map.beatmap_hash == previous_selected {
                    self.selected_index = i;
                    break
                }
            }

            return false;
        }

        if !self.selected {return false}

        if key == Key::Down {
            self.selected_index += 1;
            if self.selected_index >= self.beatmaps.len() {
                self.selected_index = 0;
            }
            
            return true;
        }

        if key == Key::Up {
            if self.selected_index == 0 {
                self.selected_index = self.beatmaps.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            
            return true;
        }

        false
    }

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        let font = get_font();
        let meta = &self.beatmaps[0];

        // draw rectangle
        list.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            parent_depth + 5.0,
            self.pos + pos_offset,
            BEATMAPSET_ITEM_SIZE,
            if self.hover {
                Some(Border::new(Color::RED, 1.0))
            } else if self.selected {
                Some(Border::new(Color::BLUE, 1.0))
            } else {
                None
            }
        ).shape(RECT_ROUND)));

        // line 1
        list.push(Box::new(Text::new(
            Color::WHITE,
            parent_depth + 4.0,
            self.pos + pos_offset + Vector2::new(5.0, 5.0),
            15,
            format!("{} // {} - {}", meta.creator, meta.artist, meta.title),
            font.clone()
        )));


        // if selected, draw map items
        if self.selected {
            let mut pos = self.pos+pos_offset + Vector2::new(BEATMAPSET_ITEM_SIZE.x - BEATMAP_ITEM_SIZE.x, BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING);
            
            // try to find the clicked item
            // // we only care about y pos, because we know we were clicked
            let mut index:usize = 999;

            // if x is in correct area to hover over beatmap items
            if self.mouse_pos.x >= self.pos.x + (BEATMAPSET_ITEM_SIZE.x - BEATMAP_ITEM_SIZE.x) {
                let rel_y2 = (self.mouse_pos.y - self.pos.y).abs() - BEATMAPSET_ITEM_SIZE.y;
                index = ((rel_y2 + BEATMAP_ITEM_PADDING/2.0) / (BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING)).floor() as usize;
            }

            if self.mouse_pos.y < self.pos.y {
                index = 999;
            }

            for i in 0..self.beatmaps.len() {
                let meta = &mut self.beatmaps[i];

                // bounding rect
                list.push(Box::new(Rectangle::new(
                    [0.2, 0.2, 0.2, 1.0].into(),
                    parent_depth + 5.0,
                    pos,
                    BEATMAP_ITEM_SIZE,
                    if i == index {
                        Some(Border::new(Color::BLUE, 1.0))
                    } else if i == self.selected_index {
                        Some(Border::new(Color::RED, 1.0))
                    } else {
                        Some(Border::new(Color::BLACK, 1.0))
                    }
                ).shape(RECT_ROUND)));

                // version text
                list.push(Box::new(Text::new(
                    Color::WHITE,
                    parent_depth + 4.0,
                    pos + Vector2::new(5.0, 5.0),
                    12,
                    format!("{} - {}", gamemode_display_name(meta.mode.clone()), meta.version),
                    font.clone()
                )));

                // diff text
                list.push(Box::new(Text::new(
                    Color::WHITE,
                    parent_depth + 4.0,
                    pos + Vector2::new(5.0, 5.0 + 20.0),
                    12,
                    meta.diff_string(self.playmode.clone(), &ModManager::get()),
                    font.clone()
                )));

                pos.y += BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING;
            }
        }
    }

    fn on_text(&mut self, playmode:String) {
        self.playmode = playmode;
    }
}


#[derive(ScrollableGettersSetters)]
pub struct LeaderboardItem {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    selected: bool,
    tag: String,

    score: Score,
    font: Arc<Mutex<opengl_graphics::GlyphCache<'static>>>
}
impl LeaderboardItem {
    pub fn new(score:Score) -> LeaderboardItem {
        let tag = score.username.clone();
        let font = get_font();

        LeaderboardItem {
            pos: Vector2::zero(),
            size: LEADERBOARD_ITEM_SIZE,
            score,
            tag,
            hover: false,
            selected: false,
            font
        }
    }
}
impl ScrollableItem for LeaderboardItem {
    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        const PADDING:Vector2 = Vector2::new(5.0, 5.0);

        // bounding rect
        list.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            parent_depth + 5.0,
            self.pos + pos_offset,
            LEADERBOARD_ITEM_SIZE,
            if self.hover {Some(Border::new(Color::RED, 1.0))} else {None}
        )));

        // score text
        list.push(Box::new(Text::new(
            Color::WHITE,
            parent_depth + 4.0,
            self.pos + pos_offset + PADDING,
            15,
            format!("{}: {}", self.score.username, crate::format_number(self.score.score)),
            self.font.clone()
        )));

        // combo text
        list.push(Box::new(Text::new(
            Color::WHITE,
            parent_depth + 4.0,
            self.pos + pos_offset + PADDING + Vector2::new(0.0, PADDING.y + 15.0),
            12,
            format!("{}x, {:.2}%", crate::format_number(self.score.max_combo), calc_acc(&self.score) * 100.0),
            self.font.clone()
        )));
    }

    // fn on_click(&mut self, _pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> bool {self.hover}
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, ayyeve_piston_ui::prelude::Dropdown)]
pub enum SortBy {
    Title,
    Artist,
    Creator,
    Difficulty,
}


#[derive(Clone, Debug)]
pub enum PlayModeDropdown {
    Mode(PlayMode)
}
impl Dropdownable for PlayModeDropdown {
    fn variants() -> Vec<Self> {
        AVAILABLE_PLAYMODES.iter().map(|p|Self::Mode(p.to_owned().to_owned())).collect()
    }

    fn display_text(&self) -> String {
        let Self::Mode(s) = self;
        gamemode_display_name(s.clone()).to_owned()
    }

    fn from_string(s:String) -> Self {
        println!("str: {}", s);
        Self::Mode(s)
    }
}
