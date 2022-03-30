use crate::prelude::*;

// constants
const INFO_BAR_HEIGHT:f64 = 60.0;
const DRAG_THRESHOLD:f64 = 50.0;
const DRAG_FACTOR:f64 = 10.0;


pub struct BeatmapSelectMenu {
    current_scores: HashMap<String, Arc<Mutex<Score>>>,
    beatmap_scroll: ScrollableArea,
    leaderboard_scroll: ScrollableArea,
    back_button: MenuButton,
    // pending_refresh: bool,

    score_loader: Option<Arc<RwLock<ScoreLoaderHelper>>>,

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

    leaderboard_method_dropdown: Dropdown<ScoreRetreivalMethod>,

    /// drag_start, confirmed_drag, last_checked, mods_when_clicked
    /// drag_start is where the original click occurred
    /// confirmed_drag is if the drag as passed a certain threshhold. important if the drag returns to below the threshhold
    mouse_down: Option<(Vector2, bool, MouseButton, Vector2, KeyModifiers)>,


    diff_calc_start_helper: (MultiFuse<()>, MultiBomb<()>),
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
            Vector2::new(205.0, 5.0),
            200.0,
            15,
            "Mode",
            Some(PlayModeDropdown::Mode(mode.clone())),
            font.clone()
        );
        
        
        let leaderboard_method = SCORE_HELPER.read().current_method;
        let leaderboard_method_dropdown = Dropdown::new(
            Vector2::new(410.0, 5.0),
            200.0, 
            15,
            "Leaderboard",
            Some(leaderboard_method),
            font.clone()
        );

        let mut beatmap_scroll = ScrollableArea::new(Vector2::new(LEADERBOARD_POS.x + LEADERBOARD_ITEM_SIZE.x, INFO_BAR_HEIGHT), Vector2::new(window_size.x - LEADERBOARD_ITEM_SIZE.x, window_size.y - INFO_BAR_HEIGHT), true);
        beatmap_scroll.dragger = DraggerSide::Right(10.0, true);

        BeatmapSelectMenu {
            no_maps_notif_sent: false,

            // mouse_down: false,
            // drag: None,

            // pending_refresh: false,
            map_changing: (false, false, 0),
            current_scores: HashMap::new(),
            back_button: MenuButton::back_button(window_size, font.clone()),

            beatmap_scroll,
            leaderboard_scroll: ScrollableArea::new(LEADERBOARD_POS, Vector2::new(LEADERBOARD_ITEM_SIZE.x, window_size.y - (LEADERBOARD_PADDING + INFO_BAR_HEIGHT)), true),
            search_text: TextInput::new(Vector2::new(window_size.x - (window_size.x / 4.0), 0.0), Vector2::new(window_size.x / 4.0, INFO_BAR_HEIGHT), "Search", "", font.clone()),

            mode: mode.clone(),
            sort_method: sort_by,

            score_loader: None,

            playmode_dropdown,
            sort_by_dropdown,
            leaderboard_method_dropdown,

            mouse_down: None,
            diff_calc_start_helper: MultiBomb::new()
        }
    }

    fn set_selected_mode(&mut self, new_mode: PlayMode, game: Option<&mut Game>) {
        // update values
        self.mode = new_mode.clone();
        self.playmode_dropdown.value = Some(PlayModeDropdown::Mode(new_mode.clone()));
        get_settings_mut!().last_played_mode = new_mode.clone();

        // recalc diffs
        let mod_manager = ModManager::get().clone();
        BEATMAP_MANAGER.write().update_diffs(new_mode.clone(), &mod_manager);
        self.diff_calc_start_helper.0.ignite(());
        
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

        //TODO: allow grouping by not just map set
        let sets = beatmap_manager.all_by_sets();
        let mut full_list = Vec::new();
        let diff_calc_helper = beatmap_manager.on_diffcalc_complete.1.clone();

        for mut maps in sets {
            if !filter_text.is_empty() {
                let filters = filter_text.split(" ");

                for filter in filters {
                    maps.retain(|bm|bm.filter(&filter));
                }

                if maps.len() == 0 {continue}
            }

            let meta = &maps[0];
            let display_text = format!("{} // {} - {}", meta.creator, meta.artist, meta.title);
            let mut i = BeatmapsetItem::new(maps, self.mode.clone(), diff_calc_helper.clone(), self.diff_calc_start_helper.1.clone(), display_text);
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

        // update diffs
        let mode_clone = self.mode.clone();
        tokio::spawn(async {
            BEATMAP_MANAGER.write().update_diffs(mode_clone, &ModManager::get());
        });
    }

    pub fn load_scores(&mut self) {
        // if nothing is selected, leave
        if let Some(map) = &BEATMAP_MANAGER.read().current_beatmap {
            self.score_loader = Some(SCORE_HELPER.read().get_scores(&map.beatmap_hash, &map.check_mode_override(self.mode.clone())));

            // clear lists
            self.leaderboard_scroll.clear();
            self.current_scores.clear();
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
                let current = current.clone();
                drop(lock);
                self.play_map(game, &current);
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
                trace!("no map?");
                // map was deleted?
                return
            }
        }
        drop(lock);


        // set any time mods
        if let Some(song) = Audio::get_song() {
            #[cfg(feature="bass_audio")]
            song.set_rate(ModManager::get().speed).unwrap();
            #[cfg(feature="neb_audio")]
            song.set_playback_speed(ModManager::get().speed as f64);
        }

        self.beatmap_scroll.refresh_layout();
        self.load_scores();
    }

    fn interactables(&mut self) -> Vec<&mut dyn ScrollableItem> {
        vec![
            &mut self.leaderboard_method_dropdown,
            &mut self.sort_by_dropdown,
            &mut self.playmode_dropdown,
            &mut self.search_text,
        ]
    }

    fn actual_on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
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

        // check sort by dropdown
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

        // check score dropdown
        let mut score_refresh = false;
        if let Some(leaderboard_method) = &self.leaderboard_method_dropdown.value {
            if &SCORE_HELPER.read().current_method != leaderboard_method {
                SCORE_HELPER.write().current_method = *leaderboard_method;
                score_refresh = true;
            }
        }
        if score_refresh {
            self.load_scores()
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
        
        for i in self.interactables() {
            i.on_click_release(pos, button) 
        }
    }
}
impl Menu<Game> for BeatmapSelectMenu {
    fn update(&mut self, game:&mut Game) {
        self.search_text.set_selected(true); // always have it selected
        let old_text = self.search_text.get_text();
        self.beatmap_scroll.update();
        self.leaderboard_scroll.update();

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

        // check load score 
        if let Some(helper) = self.score_loader.clone() {
            let helper = helper.read();
            
            if helper.done {
                self.score_loader = None;

                // load scores
                let mut scores = helper.scores.clone();
                scores.sort_by(|a, b| b.score.cmp(&a.score));

                // add scores to list
                for s in scores.iter() {
                    self.current_scores.insert(s.username.clone(), Arc::new(Mutex::new(s.clone())));
                    self.leaderboard_scroll.add_item(Box::new(LeaderboardItem::new(s.to_owned())));
                }
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
                        let _ = song.set_position(map.audio_preview as f64);
                        song.set_rate(ModManager::get().speed).unwrap();
                        song.play(false).unwrap();
                    },
                }
            }

            // no value, set it to something
            _ => {
                let lock = BEATMAP_MANAGER.read();
                match &lock.current_beatmap {
                    Some(map) => {
                        let audio = Audio::play_song(map.audio_filename.clone(), true, map.audio_preview).unwrap();
                        audio.set_rate(ModManager::get().speed).unwrap();
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
                        // trace!("song loading");
                        self.map_changing = (true, true, 0);
                    }
                }
                // we know its changing, and the previous song has ended
                (true, true, _) => {
                    if Audio::get_song().is_some() {
                        // trace!("song loaded");
                        self.map_changing = (false, false, 0);
                    }
                }
    
                // the song hasnt ended and we arent changing
                (false, false, _) | (false, true, _) => {
                    if Audio::get_song().is_none() {
                        // trace!("song done");
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
        //             // trace!("song loading");
        //             self.map_changing = (true, true, 0);
        //         }
        //     }
        //     // we know its changing, and the previous song has ended
        //     (true, true, _) => {
        //         if !song_done {
        //             // trace!("song loaded");
        //             self.map_changing = (false, false, 0);
        //         }
        //     }

        //     // the song hasnt ended and we arent changing
        //     (false, false, _) | (false, true, _) => {
        //         if song_done {
        //             // trace!("song done");
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
            // set any time mods
            #[cfg(feature="bass_audio")]
            song.set_rate(ModManager::get().speed).unwrap();
            #[cfg(feature="neb_audio")]
            song.set_playback_speed(ModManager::get().speed as f64);
        }

        // load maps
        self.refresh_maps(&mut BEATMAP_MANAGER.write());
        self.beatmap_scroll.refresh_layout();

        if BEATMAP_MANAGER.read().current_beatmap.is_some() {
            self.load_scores();
        }
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, _game:&mut Game) {
        // search text relies on this event, so if it consumed the event, ignore drag
        self.search_text.check_hover(pos);
        if self.search_text.get_hover() {
            if self.search_text.on_click(pos, button, mods) {
                // note: we shouldnt need to store mods, as search text in this instance doesnt care about it
                return;
            }
        }

        self.mouse_down = Some((pos, false, button, pos, mods));
    }
    fn on_click_release(&mut self, pos:Vector2, button:MouseButton, game:&mut Game) {
        let mut was_hold = false;
        let mut mods = None;

        // if mouse_down is none, it means we got here from the special condition where
        // the search input absorbed the on_click.
        // therefor, perform the on_release only for the search input
        if self.mouse_down.is_none() {
            self.search_text.on_click_release(pos, button);
            return
        }


        if let Some((_, was_drag, button, _, click_mods)) = self.mouse_down {
            if was_drag {
                mods = Some((click_mods, button));
                was_hold = true;
            }
        }
        self.mouse_down = None;


        // perform actual on_click
        // this is here because on_click is now only used for dragging
        let (mods, button) = mods.unwrap_or((KeyModifiers::default(), MouseButton::Left));
        if !was_hold {
            self.actual_on_click(pos, button, mods, game)
        }

    }
    
    fn on_mouse_move(&mut self, pos:Vector2, game:&mut Game) {
        let mut scroll_pos = 0.0;
        if let Some((drag_pos, confirmed_drag, button_pressed, last_checked, _)) = &mut self.mouse_down {
            if *confirmed_drag || (pos.y - drag_pos.y).abs() >= DRAG_THRESHOLD  {
                *confirmed_drag |= true;

                if *button_pressed == MouseButton::Right {
                    let offset_pos = self.beatmap_scroll.get_pos();
                    let comp_size = self.beatmap_scroll.size();
                    let y_percent = ((pos.y - offset_pos.y) / comp_size.y).clamp(0.0, 1.0);

                    let items_height = self.beatmap_scroll.get_elements_height();
                    self.beatmap_scroll.scroll_pos = -items_height * y_percent;
                } else {
                    let dist = (pos.y - last_checked.y) / DRAG_FACTOR;
                    scroll_pos = dist;
                }
            }

            *last_checked = pos;
        }
        // drag acts like scroll
        if scroll_pos != 0.0 {
            self.on_scroll(scroll_pos, game)
        }

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

        if key == Left && !mods.alt {
            if let Some(hash) = self.beatmap_scroll.select_previous_item() {
                self.select_map(game, hash, false);
                self.beatmap_scroll.scroll_to_selection();
            }
        }
        if key == Right && !mods.alt  {
            if let Some(hash) = self.beatmap_scroll.select_next_item() {
                self.select_map(game, hash, false);
                self.beatmap_scroll.scroll_to_selection();
            }
        }


        if key == Escape {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }
        if key == F5 {
            if mods.ctrl {
                NotificationManager::add_text_notification("Doing a full refresh", 5000.0, Color::RED);
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
                let display = gamemode_display_name(&new_mode);
                NotificationManager::add_text_notification(&format!("Mode changed to {}", display), 1000.0, Color::BLUE);
                self.mode = new_mode;
                self.load_scores();
            }
        }

        // mods and speed
        if mods.ctrl {
            let mut speed = ModManager::get().speed;
            let prev_speed = speed;
            const SPEED_DIFF:f32 = 0.05;

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

                // update audio speed
                if let Some(song) = Audio::get_song() {
                    #[cfg(feature="bass_audio")]
                    song.set_rate(speed).unwrap();
                    #[cfg(feature="neb_audio")]
                    song.set_playback_speed(speed as f64);
                }

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
