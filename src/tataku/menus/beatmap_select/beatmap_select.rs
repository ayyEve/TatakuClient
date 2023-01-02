use crate::prelude::*;

// constants
const INFO_BAR_HEIGHT:f64 = 60.0;
const DRAG_THRESHOLD:f64 = 50.0;
const DRAG_FACTOR:f64 = 10.0;


const DEFAULT_HEIGHT: f64 = 768.0;


pub struct BeatmapSelectMenu {
    current_scores: HashMap<String, IngameScore>,
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

    sort_method: SortBy,
    mode: PlayMode,

    sort_by_dropdown: Dropdown<SortBy, Font2, Text>,
    playmode_dropdown: Dropdown<PlayModeDropdown, Font2, Text>,

    leaderboard_method_dropdown: Dropdown<ScoreRetreivalMethod, Font2, Text>,

    /// drag_start, confirmed_drag, last_checked, mods_when_clicked
    /// drag_start is where the original click occurred
    /// confirmed_drag is if the drag as passed a certain threshhold. important if the drag returns to below the threshhold
    mouse_down: Option<(Vector2, bool, MouseButton, Vector2, KeyModifiers)>,

    window_size: Arc<WindowSize>,
    settings: SettingsHelper,
    mods: ModManagerHelper,
    current_skin: CurrentSkinHelper,

    menu_game: MenuGameHelper,
    cached_maps: Vec<Vec<Arc<BeatmapMeta>>>,

    diffcalc_complete: Option<Bomb<()>>,
    new_beatmap_helper: LatestBeatmapHelper
}
impl BeatmapSelectMenu {
    pub async fn new() -> BeatmapSelectMenu {
        let font = get_font();
        let window_size = WindowSize::get();
        let settings = SettingsHelper::new();
        let scale = window_size.y / DEFAULT_HEIGHT;

        let sort_by = settings.last_sort_by;
        let sort_by_dropdown = Dropdown::new(
            Vector2::new(0.0, 5.0),
            200.0,
            FontSize::new(15.0).unwrap(),
            "Sort",
            Some(sort_by),
            font.clone()
        );

        let mode = settings.last_played_mode.clone();
        GlobalValueManager::update(Arc::new(CurrentPlaymode(mode.clone())));
        
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
        beatmap_scroll.set_item_margin(7.0);
        beatmap_scroll.ui_scale_changed(Vector2::ONE * scale);

        let mut m = BeatmapSelectMenu {
            // pending_refresh: false,
            map_changing: (false, false, 0),
            current_scores: HashMap::new(),
            back_button: MenuButton::back_button(window_size.0, font.clone()),

            beatmap_scroll,
            leaderboard_scroll: ScrollableArea::new(LEADERBOARD_POS, Vector2::new(LEADERBOARD_ITEM_SIZE.x, window_size.y - (LEADERBOARD_PADDING + INFO_BAR_HEIGHT)), true),
            search_text: TextInput::new(Vector2::new(window_size.x - (window_size.x / 4.0), 0.0), Vector2::new(window_size.x / 4.0, INFO_BAR_HEIGHT), "Search", "", font.clone()),

            mode,
            mods: ModManagerHelper::new(),
            sort_method: sort_by,

            score_loader: None,

            playmode_dropdown,
            sort_by_dropdown,
            leaderboard_method_dropdown,

            mouse_down: None,
            // diff_calc_start_helper: MultiBomb::new()
            window_size: window_size.clone(),
            menu_game: MenuGameHelper::new(true, true, Box::new(|s|s.background_game_settings.beatmap_select_enabled)),
            settings,
            cached_maps: Vec::new(),

            diffcalc_complete: None,
            new_beatmap_helper: LatestBeatmapHelper::new(),
            current_skin: CurrentSkinHelper::new(),
        };

        // reposition things
        m.window_size_changed(window_size).await;
        m
    }

    async fn set_selected_mode(&mut self, new_mode: PlayMode) {
        // update values
        self.mode = new_mode.clone();
        GlobalValueManager::update(Arc::new(CurrentPlaymode(new_mode.clone())));

        self.playmode_dropdown.value = Some(PlayModeDropdown::Mode(new_mode.clone()));
        get_settings_mut!().last_played_mode = new_mode.clone();

        // // recalc diffs
        // self.apply_filter(&mut *BEATMAP_MANAGER.write().await).await;

        // // set modes and update diffs
        // self.beatmap_scroll.on_text(new_mode.clone());

        // change manager
        self.menu_game.setup().await;
    }


    pub async fn refresh_maps(&mut self) {
        //TODO: allow grouping by not just map set
        let sets = BEATMAP_MANAGER.read().await.all_by_sets(GroupBy::Title);
        // let diff_calc_helper = beatmap_manager.on_diffcalc_completed.1.clone();

        self.cached_maps = sets;
        self.apply_filter().await;

        // update diffs
        // let mode_clone = self.mode.clone();
        // tokio::spawn(async {
        //     BEATMAP_MANAGER.write().await.update_diffs(mode_clone, &*ModManager::get().await);
        // });
    }


    pub async fn apply_filter(&mut self) {
        self.menu_game.current_beatmap.update();
        let current_beatmap = self.menu_game.current_beatmap.0.clone();

        self.beatmap_scroll.clear();
        let filter_text = self.search_text.get_text().to_ascii_lowercase();
        let mods = self.mods.clone();
        let mode = Arc::new(self.mode.clone());
        let mut modes_needing_diffcalc = HashSet::new();

        // temp list which will need to be sorted before adding to the scrollable
        let mut full_list = Vec::new();
        
        // used to select the current map in the list
        let current_hash = if let Some(map) = current_beatmap {map.beatmap_hash.clone()} else {String::new()};

        for maps in self.cached_maps.iter() {
            let mut maps:Vec<BeatmapMetaWithDiff> = maps.iter().map(|m| {
                let mode = m.check_mode_override((*mode).clone());
                let diff = get_diff(&m, &mode, &mods);
                if diff.is_none() { modes_needing_diffcalc.insert(mode); }
                
                BeatmapMetaWithDiff::new(m.clone(), diff)
            }).collect();

            if !filter_text.is_empty() {
                let filters = filter_text.split(" ");

                for filter in filters {
                    maps.retain(|bm|bm.filter(&filter));
                }

                if maps.len() == 0 { continue }
            }

            let meta = &maps[0];
            let display_text = format!("{} // {} - {}", meta.creator, meta.artist, meta.title);
            let mut i = BeatmapsetItem::new(maps, display_text).await;
            i.check_selected(&current_hash);
            full_list.push(Box::new(i));
        }

        // sort
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

        for i in full_list { self.beatmap_scroll.add_item(i) }
        self.beatmap_scroll.scroll_to_selection();

        if modes_needing_diffcalc.len() > 0 && self.diffcalc_complete.is_none() {
            let (fuze, bomb) = Bomb::new();
            self.diffcalc_complete = Some(bomb);
            
            tokio::spawn(async move {
                for mode in modes_needing_diffcalc {
                    do_diffcalc(mode).await;
                }
                fuze.ignite(());
            });
        }

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
                lock.set_current_beatmap(game, &clicked, true).await;

                // self.setup_manager(clicked).await;
                self.menu_game.setup().await;
            }
            None => {
                trace!("no map?");
                // map was deleted?
                return
            }
        }
        drop(lock);

        // set any time mods
        if let Some(song) = AudioManager::get_song().await {
            song.set_rate(self.mods.get_speed());
        }

        self.beatmap_scroll.refresh_layout();
        self.load_scores().await;
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

        let mut dropdown_clicked = false;
        for i in self.interactables() {
            if i.on_click(pos, button, mods) {
                dropdown_clicked = true;
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
        if let Some(sort_by) = self.sort_by_dropdown.value {
            if sort_by != self.sort_method {
                self.sort_method = sort_by;
                map_refresh = true;
                get_settings_mut!().last_sort_by = sort_by;
            }
        }
        if map_refresh {
            self.refresh_maps().await
        }

        // check score dropdown
        let mut score_method_changed = false;
        if let Some(leaderboard_method) = self.leaderboard_method_dropdown.value {
            if SCORE_HELPER.read().await.current_method != leaderboard_method {
                SCORE_HELPER.write().await.current_method = leaderboard_method;
                score_method_changed = true;
                get_settings_mut!().last_score_retreival_method = leaderboard_method;
            }

        }
        if score_method_changed {
            self.load_scores().await
        }

        // dont continue if a dropdown click was handled
        // because the dropdown could be overlapping something below
        if dropdown_clicked {
            return;
        }

        
        // check if leaderboard item was clicked
        if let Some(score_tag) = self.leaderboard_scroll.on_click_tagged(pos, button, mods) {
            // score display
            if let Some(score) = self.current_scores.get(&score_tag) {
                let score = score.clone();

                if let Some(selected) = &BEATMAP_MANAGER.read().await.current_beatmap {
                    let menu = ScoreMenu::new(&score, selected.clone(), false);
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

    async fn reload_leaderboard(&mut self) {
        self.leaderboard_scroll.clear();
        
        for (_, s) in self.current_scores.iter() {
            self.leaderboard_scroll.add_item(Box::new(LeaderboardItem::new(s.clone()).load_image().await));
        }
    }
}
#[async_trait]
impl AsyncMenu<Game> for BeatmapSelectMenu {
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size.clone();
        let size = self.window_size.0;
        let scale = size.y / DEFAULT_HEIGHT;
        
        
        self.beatmap_scroll.set_pos(Vector2::new(self.window_size.x - BEATMAPSET_ITEM_SIZE.x * scale, INFO_BAR_HEIGHT));
        self.beatmap_scroll.set_size(Vector2::new(size.x - LEADERBOARD_ITEM_SIZE.x * scale, size.y - INFO_BAR_HEIGHT));
        self.beatmap_scroll.window_size_changed(size);
        self.beatmap_scroll.ui_scale_changed(Vector2::ONE * scale);
        self.beatmap_scroll.scroll_to_selection();

        
        self.leaderboard_scroll.set_size(Vector2::new(LEADERBOARD_ITEM_SIZE.x * scale, size.y - (LEADERBOARD_PADDING + INFO_BAR_HEIGHT)));
        self.leaderboard_scroll.window_size_changed(size);
        self.leaderboard_scroll.ui_scale_changed(Vector2::ONE * scale);


        self.search_text.set_pos(Vector2::new(size.x - (size.x / 4.0), 0.0));
        self.search_text.set_size(Vector2::new(size.x / 4.0, INFO_BAR_HEIGHT));

        self.back_button.set_pos(Vector2::new(10.0, size.y - (50.0 + 10.0)));


        self.menu_game.window_size_changed(window_size).await;
        self.menu_game.fit_to_area(
            Vector2::new(LEADERBOARD_ITEM_SIZE.x * scale, INFO_BAR_HEIGHT), 
            self.window_size.0 - Vector2::new(
                (LEADERBOARD_ITEM_SIZE.x + BEATMAPSET_ITEM_SIZE.x) * scale,
                INFO_BAR_HEIGHT
            )
        ).await;
    }

    async fn update(&mut self, game:&mut Game) {
        self.search_text.set_selected(true); // always have it selected
        let old_text = self.search_text.get_text();
        self.beatmap_scroll.update();
        self.leaderboard_scroll.update();
        self.settings.update();
        self.mods.update();

        for i in self.interactables() {
            i.update();
        }

        // update bg game
        self.menu_game.update().await;

        let mut filter_pending = false;
        if let Some(_) = self.diffcalc_complete.as_ref().and_then(|b| b.exploded()) {
            debug!("diffcalc done, reload maps");
            filter_pending = true;
            self.diffcalc_complete = None;
            debug!("reload maps done");
        }

        let mut refresh_pending = false;

        if self.current_skin.update() {
            filter_pending = true;
            self.reload_leaderboard().await;
        }

        if self.new_beatmap_helper.update() {
            BEATMAP_MANAGER.write().await.set_current_beatmap(game, &self.new_beatmap_helper.0, true).await;
            refresh_pending = true;
            self.menu_game.setup().await;
        }

        if old_text != self.search_text.get_text() {
            refresh_pending = true;
        }

        if refresh_pending {
            self.refresh_maps().await;
        } else if filter_pending {
            self.apply_filter().await;
        }

        {
            // if lock.force_beatmap_list_refresh {
            //     lock.force_beatmap_list_refresh = false;
            //     self.refresh_maps().await;
            // }
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
                    self.current_scores.insert(s.hash(), s.clone());
                    self.leaderboard_scroll.add_item(Box::new(LeaderboardItem::new(s.to_owned()).load_image().await));
                }
            }
        }
    
    }

    async fn draw(&mut self, args:RenderArgs, items: &mut RenderableCollection) {
        // let mut counter: usize = 0;
        let depth: f64 = 5.0;
        // let font = get_font();

        // draw a bar on the top for the info
        let bar_rect = Rectangle::new(
            Color::WHITE,
            depth - 1.0,
            Vector2::ZERO,
            Vector2::new(args.window_size[0], INFO_BAR_HEIGHT),
            Some(Border::new(Color::BLACK, 1.2))
        );
        items.push(bar_rect);

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
        self.beatmap_scroll.draw(args, Vector2::ZERO, 0.0, items);

        // leaderboard scroll
        self.leaderboard_scroll.draw(args, Vector2::ZERO, 0.0, items);

        // back button
        self.back_button.draw(args, Vector2::ZERO, 0.0, items);

        // everything else
        for i in self.interactables() {
            i.draw(args, Vector2::ZERO, 0.0, items);
        }


        // draw bg game
        self.menu_game.draw(args, items).await;
    }

    async fn on_change(&mut self, into:bool) {
        if !into { return }
        
        self.new_beatmap_helper.update();
        self.menu_game.setup().await;
        
        // update our window size
        self.window_size_changed(WindowSize::get()).await;

        OnlineManager::send_spec_frames(vec![(0.0, SpectatorFrameData::ChangingMap)], true);

        // play song if it exists
        if let Some(song) = AudioManager::get_song().await {
            // set any time mods
            song.set_rate(self.mods.get_speed());
        }

        // load maps
        self.refresh_maps().await;
        self.beatmap_scroll.refresh_layout();

        // if let Some(map) = &BEATMAP_MANAGER.read().await.current_beatmap {
            self.load_scores().await;
        // }

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

        if key == Key::M && mods.ctrl { 
            let mut found = false;
            for d in game.dialogs.iter_mut() {
                if d.name() == "mod_menu" {
                    found = true;
                    break;
                }
            }

            if !found {
                let mut groups = Vec::new();
                let mut playmode = self.mode.clone();

                if let Some(man) = &self.menu_game.manager {
                    playmode = man.gamemode.playmode();
                } else if let Some(map) = BEATMAP_MANAGER.read().await.current_beatmap.clone() {
                    playmode = map.check_mode_override(self.mode.clone());
                }

                if let Some(info) = get_gamemode_info(&playmode) {
                    groups = info.get_mods();
                }

                game.add_dialog(Box::new(ModDialog::new(groups).await)) 
            }
        }

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

        if key == F7 && mods.ctrl {
            let playmode = self.mode.clone();
            tokio::spawn(async move {
                do_diffcalc(playmode).await;
            });
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
                self.refresh_maps().await;
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
            let mut speed = self.mods.get_speed();
            let prev_speed = speed;
            const SPEED_DIFF:f32 = 0.05;

            match key {
                Equals => speed += SPEED_DIFF, // map speed up
                Minus => speed -= SPEED_DIFF, // map speed down
                 
                // autoplay enable/disable
                A => {
                    let mut manager = ModManager::get_mut();
                    let state = if manager.toggle_mod("autoplay") {"on"} else {"off"};
                    NotificationManager::add_text_notification(&format!("Autoplay {}", state), 2000.0, Color::BLUE).await;
                }

                // nofail enable/disable
                N => {
                    let mut manager = ModManager::get_mut();

                    let state = if manager.toggle_mod("no_fail") {"on"} else {"off"};
                    NotificationManager::add_text_notification(&format!("Nofail {}", state), 2000.0, Color::BLUE).await;
                }

                _ => {}
            }

            speed = speed.clamp(SPEED_DIFF, 10.0);
            if speed != prev_speed {
                ModManager::get_mut().set_speed(speed);

                // update audio speed
                if let Some(song) = AudioManager::get_song().await {
                    song.set_rate(speed);
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
            self.apply_filter().await;
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

        self.apply_filter().await;
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

