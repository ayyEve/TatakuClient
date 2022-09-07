use crate::prelude::*;

// constants
const INFO_BAR_HEIGHT:f64 = 60.0;
const DRAG_THRESHOLD:f64 = 50.0;
const DRAG_FACTOR:f64 = 10.0;


pub struct BeatmapSelectMenu {
    current_scores: HashMap<String, Arc<Mutex<IngameScore>>>,
    beatmap_scroll: ScrollableArea,
    leaderboard_scroll: ScrollableArea,
    back_button: MenuButton<Font2, Text>,
    // pending_refresh: bool,

    score_loader: Option<Arc<RwLock<ScoreLoaderHelper>>>,

    /// is changing, update loop detected that it was changing, how long since it changed
    map_changing: (bool, bool, u32),

    // drag: Option<DragData>,
    // mouse_down: bool

    /// internal search box
    search_text: TextInput<Font2, Text>,
    no_maps_notif_sent: bool,

    sort_method: SortBy,
    mode: PlayMode,

    sort_by_dropdown: Dropdown<SortBy, Font2, Text>,
    playmode_dropdown: Dropdown<PlayModeDropdown, Font2, Text>,

    leaderboard_method_dropdown: Dropdown<ScoreRetreivalMethod, Font2, Text>,

    /// drag_start, confirmed_drag, last_checked, mods_when_clicked
    /// drag_start is where the original click occurred
    /// confirmed_drag is if the drag as passed a certain threshhold. important if the drag returns to below the threshhold
    mouse_down: Option<(Vector2, bool, MouseButton, Vector2, KeyModifiers)>,
    
    // info_changed: (MultiFuse<CalcInfo>, MultiBomb<CalcInfo>),
    // diff_calc_started: (MultiFuse<()>, MultiBomb<()>),

    window_size: Arc<WindowSize>,
    settings: SettingsHelper,

    background_game: Option<IngameManager>,

    cached_maps: Vec<Vec<Arc<BeatmapMeta>>>
}
impl BeatmapSelectMenu {
    pub async fn new() -> BeatmapSelectMenu {
        let font = get_font();
        let window_size = WindowSize::get();

        let sort_by = SortBy::Title;
        let sort_by_dropdown = Dropdown::new(
            Vector2::new(0.0, 5.0),
            200.0,
            FontSize::new(15.0).unwrap(),
            "Sort",
            Some(sort_by),
            font.clone()
        );

        let mode = get_settings!().last_played_mode.clone();
        let playmode_dropdown = Dropdown::new(
            Vector2::new(205.0, 5.0),
            200.0,
            FontSize::new(15.0).unwrap(),
            "Mode",
            Some(PlayModeDropdown::Mode(mode.clone())),
            font.clone()
        );
        
        
        let leaderboard_method = SCORE_HELPER.read().await.current_method;
        let leaderboard_method_dropdown = Dropdown::new(
            Vector2::new(410.0, 5.0),
            200.0, 
            FontSize::new(15.0).unwrap(),
            "Leaderboard",
            Some(leaderboard_method),
            font.clone()
        );


        let mut beatmap_scroll = ScrollableArea::new(
            Vector2::new(window_size.x - BEATMAPSET_ITEM_SIZE.x, INFO_BAR_HEIGHT), 
            Vector2::new(window_size.x - LEADERBOARD_ITEM_SIZE.x, window_size.y - INFO_BAR_HEIGHT), 
            true
        );
        beatmap_scroll.dragger = DraggerSide::Right(10.0, true);

        let mut m = BeatmapSelectMenu {
            no_maps_notif_sent: false,

            // mouse_down: false,
            // drag: None,

            // pending_refresh: false,
            map_changing: (false, false, 0),
            current_scores: HashMap::new(),
            back_button: MenuButton::back_button(window_size.0, font.clone()),

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
            // diff_calc_start_helper: MultiBomb::new()
            window_size: window_size.clone(),
            background_game: None,
            settings: SettingsHelper::new().await,
            cached_maps: Vec::new(),
        };

        // reposition things
        m.window_size_changed(window_size).await;
        m
    }

    async fn set_selected_mode(&mut self, new_mode: PlayMode) {
        // update values
        self.mode = new_mode.clone();
        self.playmode_dropdown.value = Some(PlayModeDropdown::Mode(new_mode.clone()));
        get_settings_mut!().last_played_mode = new_mode.clone();

        // recalc diffs
        let mod_manager = ModManager::get().await.clone();
        // self.diff_calc_start_helper.0.ignite(());
        BEATMAP_MANAGER.write().await.update_diffs(new_mode.clone(), &mod_manager);

        // set modes and update diffs
        self.beatmap_scroll.on_text(new_mode.clone());

        // change manager
        if let Some(map) = &BEATMAP_MANAGER.read().await.current_beatmap {
            self.setup_manager(map.clone()).await;
        }
    }

    async fn setup_manager(&mut self, map: Arc<BeatmapMeta>) {
        if !self.settings.background_game_settings.beatmap_select_enabled { return }

        let pos = Vector2::new(LEADERBOARD_ITEM_SIZE.x, INFO_BAR_HEIGHT);
        let window_size = self.window_size.0;
        let size = Vector2::new(
            window_size.x - (LEADERBOARD_ITEM_SIZE.x + BEATMAPSET_ITEM_SIZE.x),
            window_size.y - INFO_BAR_HEIGHT
        );

        match manager_from_playmode(self.mode.clone(), &map).await {
            Ok(mut manager) => {
                manager.current_mods = Arc::new(ModManager {
                    autoplay: true,
                    ..Default::default()
                });
                manager.menu_background = true;
                manager.gamemode.fit_to_area(pos, size).await;
                manager.start().await;
                trace!("beatmapselect manager started");

                self.background_game = Some(manager);
            },
            Err(e) => {
                NotificationManager::add_error_notification("Error loading beatmap", e).await;
            }
        }

    }

    pub async fn refresh_maps(&mut self, beatmap_manager:&mut BeatmapManager) {
        //TODO: allow grouping by not just map set
        let sets = beatmap_manager.all_by_sets(GroupBy::Title);
        // let diff_calc_helper = beatmap_manager.on_diffcalc_completed.1.clone();

        self.cached_maps = sets;
        self.apply_filter(beatmap_manager).await;

        // update diffs
        let mode_clone = self.mode.clone();
        tokio::spawn(async {
            BEATMAP_MANAGER.write().await.update_diffs(mode_clone, &*ModManager::get().await);
        });
    }


    pub async fn apply_filter(&mut self, beatmap_manager:&mut BeatmapManager) {
        self.beatmap_scroll.clear();
        let filter_text = self.search_text.get_text().to_ascii_lowercase();
        let mut full_list = Vec::new();
        let mods = Arc::new(ModManager::get().await.clone());
        let mode = Arc::new(self.mode.clone());

        // used to select the current map in the list
        let current_hash = if let Some(map) = &beatmap_manager.current_beatmap {map.beatmap_hash.clone()} else {String::new()};
        
        for maps in self.cached_maps.iter() {
            let mut maps:Vec<BeatmapMetaWithDiff> = maps.iter().map(|m| 
                BeatmapMetaWithDiff::new(
                    m.clone(),
                    mods.clone(),
                    mode.clone(),
                    Arc::new(HashMap::new()),
                    beatmap_manager.on_diffcalc_started.1.clone(),
                    beatmap_manager.on_diffcalc_completed.1.clone(),
                )
            ).collect();

            if !filter_text.is_empty() {
                let filters = filter_text.split(" ");

                for filter in filters {
                    maps.retain(|bm|bm.filter(&filter));
                }

                if maps.len() == 0 { continue }
            }

            let meta = &maps[0];
            let display_text = format!("{} // {} - {}", meta.creator, meta.artist, meta.title);
            let mut i = BeatmapsetItem::new(maps, display_text, mods.clone()).await;
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


    pub async fn load_scores(&mut self) {
        // if nothing is selected, leave
        if let Some(map) = &BEATMAP_MANAGER.read().await.current_beatmap {
            self.score_loader = Some(SCORE_HELPER.read().await.get_scores(&map.beatmap_hash, &map.check_mode_override(self.mode.clone())).await);

            // clear lists
            self.leaderboard_scroll.clear();
            self.current_scores.clear();
        }
    }

    async fn play_map(&self, game: &mut Game, map: &BeatmapMeta) {
        // Audio::stop_song();
        match manager_from_playmode(self.mode.clone(), map).await {
            Ok(manager) => game.queue_state_change(GameState::Ingame(manager)),
            Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e).await
        }
    }

    async fn select_map(&mut self, game: &mut Game, map: String, can_start: bool) {
        self.background_game = None;

        let mut lock = BEATMAP_MANAGER.write().await;

        // compare last clicked map hash with the new hash.
        // if the hashes are the same, the same map was clicked twice in a row.
        // play it
        if let Some(current) = &lock.current_beatmap {
            if current.beatmap_hash == map && can_start {
                let current = current.clone();
                drop(lock);
                self.play_map(game, &current).await;
                self.map_changing = (true, false, 0);
                return;
            }
        }

        // set the current map to the clicked
        self.map_changing = (true, false, 0);
        match lock.get_by_hash(&map) {
            Some(clicked) => {
                lock.set_current_beatmap(game, &clicked, true, true).await;

                self.setup_manager(clicked).await;
            }
            None => {
                trace!("no map?");
                // map was deleted?
                return
            }
        }
        drop(lock);

        // set any time mods
        if let Some(song) = Audio::get_song().await {
            #[cfg(feature="bass_audio")]
            song.set_rate(ModManager::get().await.get_speed()).unwrap();
            #[cfg(feature="neb_audio")]
            song.set_playback_speed(ModManager::get().get_speed() as f64);
        }

        self.beatmap_scroll.refresh_layout();
        self.load_scores().await;

        // load gamemode maybe

    }

    fn interactables(&mut self) -> Vec<&mut dyn ScrollableItem> {
        vec![
            &mut self.leaderboard_method_dropdown,
            &mut self.sort_by_dropdown,
            &mut self.playmode_dropdown,
            &mut self.search_text,
        ]
    }

    async fn actual_on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
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
                new_mode = Some(selected_mode.clone());
            }
        }
        if let Some(new_mode) = new_mode {
            self.set_selected_mode(new_mode).await;
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
            self.refresh_maps(&mut *BEATMAP_MANAGER.write().await).await
        }

        // check score dropdown
        let mut score_refresh = false;
        if let Some(leaderboard_method) = &self.leaderboard_method_dropdown.value {
            if &SCORE_HELPER.read().await.current_method != leaderboard_method {
                SCORE_HELPER.write().await.current_method = *leaderboard_method;
                score_refresh = true;
            }
        }
        if score_refresh {
            self.load_scores().await
        }

        
        // check if leaderboard item was clicked
        if let Some(score_tag) = self.leaderboard_scroll.on_click_tagged(pos, button, mods) {
            // score display
            if let Some(score) = self.current_scores.get(&score_tag) {
                let score = score.lock().await.clone();

                if let Some(selected) = &BEATMAP_MANAGER.read().await.current_beatmap {
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

            self.select_map(game, clicked_hash, button == MouseButton::Left).await;
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
#[async_trait]
impl AsyncMenu<Game> for BeatmapSelectMenu {
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size.clone();
        let size = self.window_size.0;

        
        self.beatmap_scroll.set_pos(Vector2::new(size.x - (BEATMAPSET_ITEM_SIZE.x + BEATMAPSET_PAD_RIGHT), INFO_BAR_HEIGHT));
        self.beatmap_scroll.set_size(Vector2::new(size.x - LEADERBOARD_ITEM_SIZE.x, size.y - INFO_BAR_HEIGHT));
        self.beatmap_scroll.window_size_changed(size);

        
        self.leaderboard_scroll.set_size(Vector2::new(LEADERBOARD_ITEM_SIZE.x, size.y - (LEADERBOARD_PADDING + INFO_BAR_HEIGHT)));
        self.leaderboard_scroll.window_size_changed(size);


        self.search_text.set_pos(Vector2::new(size.x - (size.x / 4.0), 0.0));
        self.search_text.set_size(Vector2::new(size.x / 4.0, INFO_BAR_HEIGHT));


        self.back_button.set_pos(Vector2::new(10.0, size.y - (50.0 + 10.0)));


        if let Some(m) = &mut self.background_game {
            m.window_size_changed(window_size).await;

            let pos = Vector2::new(LEADERBOARD_ITEM_SIZE.x, INFO_BAR_HEIGHT);
            let window_size = self.window_size.0;
            let size = Vector2::new(
                window_size.x - (LEADERBOARD_ITEM_SIZE.x + BEATMAPSET_ITEM_SIZE.x),
                window_size.y - INFO_BAR_HEIGHT
            );
            m.gamemode.fit_to_area(pos, size).await;
        }
    }

    async fn update(&mut self, game:&mut Game) {
        self.search_text.set_selected(true); // always have it selected
        let old_text = self.search_text.get_text();
        self.beatmap_scroll.update();
        self.leaderboard_scroll.update();
        self.settings.update();

        for i in self.interactables() {
            i.update();
        }

        // update bg game
        if let Some(manager) = &mut self.background_game {
            manager.update().await;
        }


        {
            let mut lock = BEATMAP_MANAGER.write().await;

            if old_text != self.search_text.get_text() {
                self.refresh_maps(&mut lock).await;
            }

            let maps = lock.get_new_maps();
            if maps.len() > 0  {
                lock.set_current_beatmap(game, &maps[maps.len() - 1], false, true).await;
                self.refresh_maps(&mut lock).await;
            }
            if lock.force_beatmap_list_refresh {
                lock.force_beatmap_list_refresh = false;
                self.refresh_maps(&mut lock).await;
            }
        }

        // check load score 
        if let Some(helper) = self.score_loader.clone() {
            let helper = helper.read().await;
            
            if helper.done {
                self.score_loader = None;

                // load scores
                let mut scores = helper.scores.clone();
                scores.sort_by(|a, b| b.score.score.cmp(&a.score.score));

                // add scores to list
                for s in scores.iter() {
                    self.current_scores.insert(s.hash(), Arc::new(Mutex::new(s.clone())));
                    self.leaderboard_scroll.add_item(Box::new(LeaderboardItem::new(s.to_owned())));
                }
            }
        }
    
        #[cfg(feature="bass_audio")]
        match Audio::get_song().await {
            Some(song) => {
                match song.get_playback_state() {
                    Ok(PlaybackState::Playing) => {},
                    _ => {
                        // restart the song at the preview point
                        let lock = BEATMAP_MANAGER.read().await;
                        let map = lock.current_beatmap.as_ref().unwrap();
                        let _ = song.set_position(map.audio_preview as f64);
                        song.set_rate(ModManager::get().await.get_speed()).unwrap();
                        
                        song.play(false).unwrap();
                        self.setup_manager(map.clone()).await;
                    },
                }
            }

            // no value, set it to something
            _ => {
                let lock = BEATMAP_MANAGER.read().await;
                match &lock.current_beatmap {
                    Some(map) => {
                        let audio = Audio::play_song(map.audio_filename.clone(), true, map.audio_preview).await.unwrap();
                        audio.set_rate(ModManager::get().await.get_speed()).unwrap();
                    }
                    None => if !self.no_maps_notif_sent {
                        NotificationManager::add_text_notification("No beatmaps\nHold on...", 5000.0, Color::GREEN).await;
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
                            let lock = BEATMAP_MANAGER.read();
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

    async fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
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


        // update bg game
        if let Some(manager) = &mut self.background_game {
            manager.draw(args, &mut items).await;
        }

        items
    }

    async fn on_change(&mut self, into:bool) {
        if !into { return }
        
        // update our window size
        self.window_size_changed(WindowSize::get()).await;

        OnlineManager::send_spec_frames(vec![(0.0, SpectatorFrameData::ChangingMap)], true);

        // play song if it exists
        if let Some(song) = Audio::get_song().await {
            // set any time mods
            #[cfg(feature="bass_audio")]
            song.set_rate(ModManager::get().await.get_speed()).unwrap();
            #[cfg(feature="neb_audio")]
            song.set_playback_speed(ModManager::get().speed as f64);
        }

        // load maps
        self.refresh_maps(&mut *BEATMAP_MANAGER.write().await).await;
        self.beatmap_scroll.refresh_layout();

        if let Some(map) = &BEATMAP_MANAGER.read().await.current_beatmap {
            self.load_scores().await;
            self.setup_manager(map.clone()).await;
        }

    }

    async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, _game:&mut Game) {
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
    async fn on_click_release(&mut self, pos:Vector2, button:MouseButton, game:&mut Game) {
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
            self.actual_on_click(pos, button, mods, game).await;
        }

    }
    
    async fn on_mouse_move(&mut self, pos:Vector2, game:&mut Game) {
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
            self.on_scroll(scroll_pos, game).await
        }

        for i in self.interactables() {
            i.on_mouse_move(pos) 
        }
        self.back_button.on_mouse_move(pos);
        self.beatmap_scroll.on_mouse_move(pos);
        self.leaderboard_scroll.on_mouse_move(pos);
    }
    async fn on_scroll(&mut self, delta:f64, _game:&mut Game) {
        let mut h = false;

        h |= self.beatmap_scroll.on_scroll(delta);
        h |= self.leaderboard_scroll.on_scroll(delta);

        for i in self.interactables() {
            h |= i.on_scroll(delta);
        }

        if !h {
            // make the scroll think its hovered
            self.beatmap_scroll.set_hover(true);
            // try again
            self.beatmap_scroll.on_scroll(delta);
        }
    }

    async fn on_key_press(&mut self, key:piston::Key, game:&mut Game, mods:KeyModifiers) {
        use piston::Key::*;

        if key == Left && !mods.alt {
            if let Some(hash) = self.beatmap_scroll.select_previous_item() {
                self.select_map(game, hash, false).await;
                self.beatmap_scroll.scroll_to_selection();
            }
        }
        if key == Right && !mods.alt  {
            if let Some(hash) = self.beatmap_scroll.select_next_item() {
                self.select_map(game, hash, false).await;
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
                NotificationManager::add_text_notification("Doing a full refresh", 5000.0, Color::RED).await;
                BEATMAP_MANAGER.write().await.full_refresh().await;
            } else {
                self.refresh_maps(&mut *BEATMAP_MANAGER.write().await).await;
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
                self.set_selected_mode(new_mode.clone()).await;
                let display = gamemode_display_name(&new_mode);
                NotificationManager::add_text_notification(&format!("Mode changed to {}", display), 1000.0, Color::BLUE).await;
                self.mode = new_mode;
                self.load_scores().await;
            }
        }

        // mods and speed
        if mods.ctrl {
            let mut speed = ModManager::get().await.get_speed();
            let prev_speed = speed;
            const SPEED_DIFF:f32 = 0.05;

            match key {
                Equals => speed += SPEED_DIFF, // map speed up
                Minus => speed -= SPEED_DIFF, // map speed down
                 
                // autoplay enable/disable
                A => {
                    let mut manager = ModManager::get().await;
                    manager.autoplay = !manager.autoplay;

                    let state = if manager.autoplay {"on"} else {"off"};
                    NotificationManager::add_text_notification(&format!("Autoplay {}", state), 2000.0, Color::BLUE).await;
                }

                // nofail enable/disable
                N => {
                    let mut manager = ModManager::get().await;
                    manager.nofail = !manager.nofail;

                    let state = if manager.nofail {"on"} else {"off"};
                    NotificationManager::add_text_notification(&format!("Nofail {}", state), 2000.0, Color::BLUE).await;
                }

                _ => {}
            }

            speed = speed.clamp(SPEED_DIFF, 10.0);
            if speed != prev_speed {
                ModManager::get().await.set_speed(speed);

                // update audio speed
                if let Some(song) = Audio::get_song().await {
                    #[cfg(feature="bass_audio")]
                    song.set_rate(speed).unwrap();
                    #[cfg(feature="neb_audio")]
                    song.set_playback_speed(speed as f64);
                }

                // force diff recalc
                self.set_selected_mode(self.mode.clone()).await;

                NotificationManager::add_text_notification(&format!("Map speed: {:.2}x", speed), 2000.0, Color::BLUE).await;
            }
        }

        // if enter was hit, or a beatmap item was updated
        if self.beatmap_scroll.on_key_press(key, mods) || key == Return {
            if let Some(selected_index) = self.beatmap_scroll.get_selected_index() {
                if let Some(item) = self.beatmap_scroll.items.get(selected_index) {
                    let hash = item.get_tag();
                    self.select_map(game, hash, key == Return).await;
                }
            }
        }
        

        // only refresh if the text changed
        let old_text = self.search_text.get_text();

        for i in self.interactables() {
            i.on_key_press(key, mods);
        }

        if self.search_text.get_text() != old_text {
            // self.refresh_maps(&mut *BEATMAP_MANAGER.write().await).await;
            self.apply_filter(&mut *BEATMAP_MANAGER.write().await).await;
        }
    }

    async fn on_key_release(&mut self, key:piston::Key, _game:&mut Game) {
        for i in self.interactables() {
            i.on_key_release(key);
        }
    }

    async fn on_text(&mut self, text:String) {
        // DO NOT ACTIVATE FOR BEATMAP ITEMS!!
        // on_text is used to change the playmode lol
        for i in self.interactables() {
            i.on_text(text.clone());
        }

        self.apply_filter(&mut *BEATMAP_MANAGER.write().await).await;
    }
}
#[async_trait]
impl ControllerInputMenu<Game> for BeatmapSelectMenu {
    async fn controller_down(&mut self, game:&mut Game, controller: &Box<dyn Controller>, button: u8) -> bool {
        if let Some(ControllerButton::DPad_Up) = controller.map_button(button) {
            self.on_key_press(Key::Up, game, KeyModifiers::default()).await
        }
        if let Some(ControllerButton::DPad_Down) = controller.map_button(button) {
            self.on_key_press(Key::Down, game, KeyModifiers::default()).await
        }
        if let Some(ControllerButton::DPad_Left) = controller.map_button(button) {
            self.on_key_press(Key::Left, game, KeyModifiers::default()).await
        }
        if let Some(ControllerButton::DPad_Right) = controller.map_button(button) {
            self.on_key_press(Key::Right, game, KeyModifiers::default()).await
        }

        if let Some(ControllerButton::A) = controller.map_button(button) {
            self.on_key_press(Key::Return, game, KeyModifiers::default()).await
        }

        if let Some(ControllerButton::B) = controller.map_button(button) {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
        }
        
        false
    }

    async fn controller_axis(&mut self, _game:&mut Game, controller: &Box<dyn Controller>, axis_data: HashMap<u8, (bool, f64)>) -> bool {
        for (axis, (_, val)) in axis_data {
            if Some(ControllerAxis::Right_Y) == controller.map_axis(axis) && val.abs() > 0.1 {
                self.beatmap_scroll.set_hover(true);
                self.beatmap_scroll.on_scroll(-val / 16.0);
            }
        }

        false
    }
}

