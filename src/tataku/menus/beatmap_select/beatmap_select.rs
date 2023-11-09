use crate::{prelude::*, create_value_helper};

// constants
// const INFO_BAR_HEIGHT:f32 = 60.0;
const DRAG_THRESHOLD:f32 = 50.0;
const DRAG_FACTOR:f32 = 10.0;

// const DEFAULT_WIDTH: f32 = 1270.0;
// const DEFAULT_HEIGHT: f32 = 768.0;

create_value_helper!(CurrentSortBy, SortBy, SortByHelper);
create_value_helper!(CurrentScoreMethod, ScoreRetreivalMethod, ScoreMethodHelper);
create_value_helper!(FilterText, String, FilterTextHelper);

create_value_helper!(PlayRequest, (), PlayRequestHelper);



pub struct BeatmapSelectMenu {
    current_scores: HashMap<String, IngameScore>,

    top_bar: ScrollableArea,
    beatmap_scroll: ScrollableArea,
    leaderboard_scroll: ScrollableArea,
    back_button: MenuButton,

    layout_manager: LayoutManager,
    // pending_refresh: bool,

    score_loader: Option<Arc<AsyncRwLock<ScoreLoaderHelper>>>,

    /// is changing, update loop detected that it was changing, how long since it changed
    map_changing: (bool, bool, u32),

    // drag: Option<DragData>,
    // mouse_down: bool

    // /// internal search box
    // search_text: TextInput,

    sort_method: SortByHelper,
    mode: CurrentPlaymodeHelper,
    leaderboard_method: ScoreMethodHelper,
    filter_text: FilterTextHelper,
    play_request: PlayRequestHelper,

    // sort_by_dropdown: Dropdown<SortBy>,
    // playmode_dropdown: Dropdown<PlayModeDropdown>,
    // leaderboard_method_dropdown: Dropdown<ScoreRetreivalMethod>,

    /// drag_start, confirmed_drag, last_checked, mods_when_clicked
    /// drag_start is where the original click occurred
    /// confirmed_drag is if the drag as passed a certain threshhold. important if the drag returns to below the threshhold
    mouse_down: Option<(Vector2, bool, MouseButton, Vector2, KeyModifiers)>,

    // window_size: Arc<WindowSize>,
    settings: SettingsHelper,
    mods: ModManagerHelper,
    current_skin: CurrentSkinHelper,

    menu_game: MenuGameHelper,
    cached_maps: Vec<Vec<Arc<BeatmapMeta>>>,

    diffcalc_complete: Option<Bomb<()>>,
    new_beatmap_helper: LatestBeatmapHelper,

    pub select_action: BeatmapSelectAction
}
impl BeatmapSelectMenu {
    pub async fn new() -> Self {
        // let window_size = WindowSize::get();
        let settings = SettingsHelper::new();
        // let scale = window_size.y / DEFAULT_HEIGHT;

    
        let mode = settings.last_played_mode.clone();
        let sort_by = settings.last_sort_by;

        let layout_manager = LayoutManager::new();
        layout_manager.set_style(Style {
            size: LayoutManager::full_size(),
            display: taffy::style::Display::Flex,
            // flex_direction: taffy::style::FlexDirection::Column,
            flex_wrap: taffy::style::FlexWrap::Wrap,
            
            ..Default::default()
        });

        let mut top_bar = ScrollableArea::new(
            Style {
                size: Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Percent(0.1),
                },
                display: taffy::style::Display::Flex,
                flex_direction: taffy::style::FlexDirection::Row,
                
                ..Default::default()
            }, 
            ListMode::None, 
            &layout_manager
        );
        top_bar.set_tag("top_bar");
        top_bar.draw_rect = Some((Color::WHITE, Some(Border::new(Color::BLACK, 1.2))));

        // add items to the top bar
        {
            let dropdown_style = Style {
                size: Size {
                    width: Dimension::Percent(0.5 * 0.33),
                    height: Dimension::Auto,
                },
                ..Default::default()
            };

            GlobalValueManager::update(Arc::new(CurrentSortBy(sort_by)));
            let sort_by_dropdown = Dropdown::new(
                dropdown_style.clone(),
                15.0,
                "Sort",
                Some(sort_by),
                &top_bar.layout_manager,
                Font::Main
            ).with_on_change(move|_, b| if let Some(sortby) = b {GlobalValueManager::update(Arc::new(CurrentSortBy(sortby))); });
            top_bar.add_item(Box::new(sort_by_dropdown));

            
            GlobalValueManager::update(Arc::new(CurrentPlaymode(mode.clone())));
            let playmode_dropdown = Dropdown::new(
                dropdown_style.clone(),
                15.0,
                "Mode",
                Some(PlayModeDropdown::Mode(mode.clone())),
                &top_bar.layout_manager,
                Font::Main
            );
            //TODO: add playmode dropdown sender
            top_bar.add_item(Box::new(playmode_dropdown));

            
            let leaderboard_method = SCORE_HELPER.read().await.current_method;
            GlobalValueManager::update(Arc::new(CurrentScoreMethod(leaderboard_method)));
            let leaderboard_method_dropdown = Dropdown::new(
                dropdown_style.clone(),
                15.0,
                "Leaderboard",
                Some(leaderboard_method),
                &top_bar.layout_manager,
                Font::Main
            )
            .with_on_change(|_, b|if let Some(sortby) = b {GlobalValueManager::update(Arc::new(CurrentScoreMethod(sortby))); });
            top_bar.add_item(Box::new(leaderboard_method_dropdown));

            // search input
            GlobalValueManager::update(Arc::new(FilterText(String::new())));
            let mut search_text = TextInput::new(Style {
                size: Size {
                    width: Dimension::Percent(0.5 * 0.33),
                    height: Dimension::Auto,
                },
                ..Default::default()
            }, "Search", "", &top_bar.layout_manager, Font::Main);
            search_text.on_change = Arc::new(|_,s|GlobalValueManager::update(Arc::new(FilterText(s))));
            top_bar.add_item(Box::new(search_text));
        }
        

        // [leaderboard] [gameplay preview] [beatmap list]
        // [10%] [40%] [50%]

        let mut leaderboard_scroll = ScrollableArea::new(
            Style {
                size: Size {
                    width: Dimension::Percent(0.1),
                    height: Dimension::Percent(1.0),
                },
                display: taffy::style::Display::Flex,
                flex_direction: taffy::style::FlexDirection::Column,
                ..Default::default()
            }, 
            ListMode::VerticalList,
            &layout_manager
        );
        leaderboard_scroll.set_tag("leaderboard_scroll");
        leaderboard_scroll.draw_rect = Some((Color::LIME.alpha(0.7), None));

        let gameplay_node = GenericNode::new(
            Style {
                size: Size {
                    width: Dimension::Percent(0.4),
                    height: Dimension::Percent(1.0),
                },
                ..Default::default()
            }, 
            &layout_manager
        );

        let mut beatmap_scroll = ScrollableArea::new(
            Style {
                size: Size {
                    width: Dimension::Percent(0.5),
                    height: Dimension::Percent(1.0),
                },
                display: taffy::style::Display::Flex,
                flex_direction: taffy::style::FlexDirection::Column,
                ..Default::default()
            },
            ListMode::VerticalList,
            &layout_manager,
        );
        beatmap_scroll.set_tag("beatmap_scroll");
        beatmap_scroll.dragger = DraggerSide::Right(10.0, true);
        beatmap_scroll.draw_rect = Some((Color::RED.alpha(0.7), None));
        // beatmap_scroll.set_item_margin(7.0);
        // beatmap_scroll.ui_scale_changed(Vector2::ONE * scale);

        GlobalValueManager::update(Arc::new(PlayRequest(())));
        Self {
            // pending_refresh: false,
            map_changing: (false, false, 0),
            current_scores: HashMap::new(),
            back_button: MenuButton::back_button(Font::Main, &layout_manager),
            layout_manager,

            top_bar,
            beatmap_scroll,
            leaderboard_scroll,
            // search_text,

            mode: GlobalValue::new(),
            mods: ModManagerHelper::new(),
            sort_method: SortByHelper::new(),
            leaderboard_method: ScoreMethodHelper::new(),
            filter_text: FilterTextHelper::new(),
            play_request: PlayRequestHelper::new(),

            score_loader: None,

            // playmode_dropdown,
            // sort_by_dropdown,
            // leaderboard_method_dropdown,

            mouse_down: None,
            // diff_calc_start_helper: MultiBomb::new()
            // window_size: window_size.clone(),
            menu_game: MenuGameHelper::new(true, true, Box::new(|s|s.background_game_settings.beatmap_select_enabled)),
            settings,
            cached_maps: Vec::new(),

            diffcalc_complete: None,
            new_beatmap_helper: LatestBeatmapHelper::new(),
            current_skin: CurrentSkinHelper::new(),
            select_action: BeatmapSelectAction::PlayMap
        }
    }


    pub async fn refresh_maps(&mut self) {
        //TODO: allow grouping by not just map set
        let sets = BEATMAP_MANAGER.read().await.all_by_sets(GroupBy::Title);

        self.cached_maps = sets;
        self.apply_filter().await;
    }


    pub async fn apply_filter(&mut self) {
        self.menu_game.current_beatmap.update();
        let current_beatmap = self.menu_game.current_beatmap.0.clone();

        self.beatmap_scroll.clear();
        let filter_text = self.filter_text.0.to_ascii_lowercase();
        let mods = self.mods.clone();
        let mode = self.mode.0.clone();
        let mut modes_needing_diffcalc = HashSet::new();

        // temp list which will need to be sorted before adding to the scrollable
        let mut full_list = Vec::new();
        
        // used to select the current map in the list
        let current_hash = current_beatmap.map(|m|m.beatmap_hash).unwrap_or_default();
        for maps in self.cached_maps.iter() {
            let mut maps:Vec<BeatmapMetaWithDiff> = maps.iter().map(|m| {
                let mode = m.check_mode_override(mode.clone());
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
            let mut i = BeatmapsetItem::new(maps, display_text, &self.beatmap_scroll.layout_manager).await;
            i.check_selected(current_hash);
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
        match self.sort_method.0 {
            SortBy::Title => sort!(title, String),
            SortBy::Artist => sort!(artist, String),
            SortBy::Creator => sort!(creator, String),
            SortBy::Difficulty => sort!(diff, Float),
        }

        for i in full_list { self.beatmap_scroll.add_item(i) }
        self.beatmap_scroll.scroll_to_selection();

        if modes_needing_diffcalc.len() > 0 && self.diffcalc_complete.is_none() {
            self.run_diffcalc(modes_needing_diffcalc.into_iter(), false);
        }

    }

    fn run_diffcalc(&mut self, modes: impl Iterator<Item=String>+Send+'static, manual: bool) {
        // if diffcalc is not enabled, and this wasnt manually triggered
        if !self.settings.enable_diffcalc && !manual { return }

        let (fuze, bomb) = Bomb::new();
        self.diffcalc_complete = Some(bomb);
        
        tokio::spawn(async move {
            for mode in modes {
                if manual {
                    NotificationManager::add_text_notification(format!("Diffcalc started for mode {mode}"), 5_000.0, Color::BLUE).await;
                    BEATMAP_DIFFICULTIES.get(&mode).unwrap().write().unwrap().clear();
                }

                do_diffcalc(mode.clone()).await;

                if manual {
                    NotificationManager::add_text_notification(format!("Diffcalc complete for mode {mode}"), 5_000.0, Color::BLUE).await;
                }
            }
            fuze.ignite(());
        });
    }


    pub async fn load_scores(&mut self) {
        // if nothing is selected, leave
        if let Some(map) = &BEATMAP_MANAGER.read().await.current_beatmap {
            self.score_loader = Some(SCORE_HELPER.read().await.get_scores(map.beatmap_hash, &map.check_mode_override(self.mode.0.clone())).await);

            // clear lists
            self.leaderboard_scroll.clear();
            self.current_scores.clear();
        }
    }

    async fn play_map(&self, game: &mut Game, map: &Arc<BeatmapMeta>) {
        match &self.select_action {
            BeatmapSelectAction::PlayMap => {
                // Audio::stop_song();
                match manager_from_playmode(self.mode.0.clone(), map).await {
                    Ok(mut manager) => {
                        let mods = ModManager::get();
                        manager.apply_mods(mods.deref().clone()).await;
                        game.queue_state_change(GameState::Ingame(Box::new(manager)))
                    }
                    Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e).await
                }
            }

            BeatmapSelectAction::OnComplete(sender) => {
                sender.send(Some((map.clone(), self.mode.0.clone()))).await.unwrap();
            }
        }
    }

    async fn select_map(&mut self, game: &mut Game, map: Md5Hash, can_start: bool) {
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

    async fn actual_on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        if self.back_button.on_click(pos, button, mods) {
            match &self.select_action {
                BeatmapSelectAction::PlayMap => game.queue_state_change(GameState::InMenu(Box::new(MainMenu::new().await))),
                BeatmapSelectAction::OnComplete(sender) => sender.send(None).await.unwrap(),
            }
            return;
        }

        let dropdown_clicked = self.top_bar.on_click(pos, button, mods);
        // for i in self.interactables() {
        //     if i.on_click(pos, button, mods) {
        //         dropdown_clicked = true;
        //         break;
        //     }
        // }
        

        // check sort by dropdown
        if self.sort_method.update() {
            Settings::get_mut().last_sort_by = self.sort_method.0;
            self.refresh_maps().await
        }

        // check score dropdown
        if self.leaderboard_method.update() {
            SCORE_HELPER.write().await.current_method = self.leaderboard_method.0;
            Settings::get_mut().last_score_retreival_method = self.leaderboard_method.0;
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
                    game.queue_state_change(GameState::InMenu(Box::new(menu)));
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
        if let Some(clicked_hash) = self.beatmap_scroll.on_click_tagged(pos, button, mods).and_then(|h|Md5Hash::try_from(h).ok()) {
            if button == MouseButton::Right {
                // clicked hash is the target
                let dialog = BeatmapDialog::new(clicked_hash.clone());
                game.add_dialog(Box::new(dialog), false);
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
        
        self.top_bar.on_click_release(pos, button);
    }

    async fn reload_leaderboard(&mut self) {
        self.leaderboard_scroll.clear();
        let layout_manager = self.leaderboard_scroll.layout_manager.clone();
        
        for (_, s) in self.current_scores.iter() {
            self.leaderboard_scroll.add_item(Box::new(LeaderboardItem::new(Style::default(), s.clone(), &layout_manager).load_image().await));
        }
    }
}

#[async_trait]
impl AsyncMenu<Game> for BeatmapSelectMenu {
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.layout_manager.apply_layout(window_size.0);

        self.top_bar.apply_layout(&self.layout_manager, Vector2::ZERO);
        self.beatmap_scroll.apply_layout(&self.layout_manager, Vector2::ZERO);
        self.leaderboard_scroll.apply_layout(&self.layout_manager, Vector2::ZERO);

        // self.window_size = window_size.clone();
        // let size = self.window_size.0;
        // let scale = size.x / DEFAULT_WIDTH;
        // let scale2 = size.y / DEFAULT_HEIGHT;
        // let scale = scale.min(scale2);
        
        // self.beatmap_scroll.set_pos(Vector2::new(self.window_size.x - BEATMAPSET_ITEM_SIZE.x * scale, INFO_BAR_HEIGHT));
        // self.beatmap_scroll.set_size(Vector2::new(self.window_size.x - LEADERBOARD_ITEM_SIZE.x * scale, size.y - INFO_BAR_HEIGHT));
        // self.beatmap_scroll.window_size_changed(size);
        // self.beatmap_scroll.ui_scale_changed(Vector2::ONE * scale);
        // self.beatmap_scroll.scroll_to_selection();

        
        // self.leaderboard_scroll.set_size(Vector2::new(LEADERBOARD_ITEM_SIZE.x * scale, size.y - (LEADERBOARD_PADDING + INFO_BAR_HEIGHT)));
        // self.leaderboard_scroll.window_size_changed(size);
        // self.leaderboard_scroll.ui_scale_changed(Vector2::ONE * scale);


        // self.search_text.set_pos(Vector2::new(size.x - (size.x / 4.0), 0.0));
        // self.search_text.set_size(Vector2::new(size.x / 4.0, INFO_BAR_HEIGHT));

        // self.back_button.set_pos(Vector2::new(10.0, size.y - (50.0 + 10.0)));


        // self.menu_game.window_size_changed(window_size).await;
        // self.menu_game.fit_to_area(Bounds::new(
        //     Vector2::new(LEADERBOARD_ITEM_SIZE.x * scale, INFO_BAR_HEIGHT), 
        //     self.window_size.0 - Vector2::new(
        //         (LEADERBOARD_ITEM_SIZE.x + BEATMAPSET_ITEM_SIZE.x) * scale,
        //         INFO_BAR_HEIGHT
        //     )
        // )).await;
    }

    async fn update(&mut self, game:&mut Game) {
        // self.search_text.set_selected(true); // always have it selected
        // let old_text = self.search_text.get_text();
        self.top_bar.update();
        self.beatmap_scroll.update();
        self.leaderboard_scroll.update();
        self.settings.update();
        self.mods.update();

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

        if self.filter_text.update() {
            refresh_pending = true;
        }

        if refresh_pending {
            self.refresh_maps().await;
        } else if filter_pending {
            self.apply_filter().await;
        } else {
            // check if selected mode changed
            if self.mode.update() {
                GlobalValueManager::update(Arc::new(CurrentPlaymode(self.mode.0.clone())));
                Settings::get_mut().last_played_mode = self.mode.0.clone();

                // change manager
                self.menu_game.setup().await;
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
                
                let layout_manager = self.leaderboard_scroll.layout_manager.clone();

                // add scores to list
                for s in scores.iter() {
                    self.current_scores.insert(s.hash(), s.clone());
                    self.leaderboard_scroll.add_item(Box::new(LeaderboardItem::new(Style::default(), s.to_owned(), &layout_manager).load_image().await));
                }
            }
        }
    
    }

    async fn draw(&mut self, items: &mut RenderableCollection) {

        // draw bg game
        self.menu_game.draw(items).await;

        // // draw selected map info
        // if let Some(meta) = &mut BEATMAP_MANAGER.write().current_beatmap {
        //     // draw map name top-most left-most
        //     items.push(Box::new(Text::new(
        //         Color::BLACK,
        //         -10.0,
        //         Vector2::new(0.0, 5.0),
        //         25,
        //         meta.version_string(),
        //         Font::Main
        //     )));

        //     // diff string, under map string
        //     items.push(Box::new(Text::new(
        //         Color::BLACK,
        //         -10.0,
        //         Vector2::new(0.0, 35.0),
        //         15,
        //         meta.diff_string(self.mode.clone(), &ModManager::get()),
        //         Font::Main
        //     )));
        // }

        // top bar
        self.top_bar.draw(Vector2::ZERO, items);

        // beatmap scroll
        self.beatmap_scroll.draw(Vector2::ZERO, items);

        // leaderboard scroll
        self.leaderboard_scroll.draw(Vector2::ZERO, items);

        // back button
        self.back_button.draw(Vector2::ZERO, items);

    }

    async fn on_change(&mut self, into:bool) {
        if !into { return }
        
        self.new_beatmap_helper.update();
        self.menu_game.setup().await;
        
        // update our window size
        self.window_size_changed(WindowSize::get()).await;

        OnlineManager::send_spec_frames(vec![SpectatorFrame::new(0.0, SpectatorAction::ChangingMap)], true);

        // play song if it exists
        if let Some(song) = AudioManager::get_song().await {
            // set any time mods
            song.set_rate(self.mods.get_speed());

            // ensure song is playing
            song.play(false);
        }

        // load maps
        self.refresh_maps().await;
        self.beatmap_scroll.refresh_layout();

        // if let Some(map) = &BEATMAP_MANAGER.read().await.current_beatmap {
            self.load_scores().await;
        // }

    }

    async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, _game:&mut Game) {
        // // search text relies on this event, so if it consumed the event, ignore drag
        // self.search_text.check_hover(pos);
        // if self.search_text.get_hover() {
        //     if self.search_text.on_click(pos, button, mods) {
        //         // note: we shouldnt need to store mods, as search text in this instance doesnt care about it
        //         return;
        //     }
        // }
        if self.top_bar.get_hover() {
            return;
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
            self.top_bar.on_click_release(pos, button);
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
        let (mods, button) = mods.unwrap_or((KeyModifiers::default(), button));
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

        self.top_bar.on_mouse_move(pos);
        self.back_button.on_mouse_move(pos);
        self.beatmap_scroll.on_mouse_move(pos);
        self.leaderboard_scroll.on_mouse_move(pos);
    }
    async fn on_scroll(&mut self, delta:f32, _game:&mut Game) {
        let mut h = false;

        h |= self.top_bar.on_scroll(delta);
        h |= self.beatmap_scroll.on_scroll(delta);
        h |= self.leaderboard_scroll.on_scroll(delta);

        if !h {
            // make the scroll think its hovered
            self.beatmap_scroll.set_hover(true);
            // try again
            self.beatmap_scroll.on_scroll(delta);
        }
    }

    async fn on_key_press(&mut self, key:Key, game:&mut Game, mods:KeyModifiers) {
        use Key::*;

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
                let mut playmode = self.mode.0.clone();

                if let Some(man) = &self.menu_game.manager {
                    playmode = man.gamemode.playmode();
                } else if let Some(map) = BEATMAP_MANAGER.read().await.current_beatmap.clone() {
                    playmode = map.check_mode_override(self.mode.0.clone());
                }

                if let Some(info) = get_gamemode_info(&playmode) {
                    groups = info.get_mods();
                }

                game.add_dialog(Box::new(ModDialog::new(groups).await), false) 
            }
        }

        if key == Left && !mods.alt {
            if let Some(hash) = self.beatmap_scroll.select_previous_item().and_then(|h|h.try_into().ok()) {
                self.select_map(game, hash, false).await;
                self.beatmap_scroll.scroll_to_selection();
            }
        }
        if key == Right && !mods.alt  {
            if let Some(hash) = self.beatmap_scroll.select_next_item().and_then(|h|h.try_into().ok()) {
                self.select_map(game, hash, false).await;
                self.beatmap_scroll.scroll_to_selection();
            }
        }

        if key == F7 && mods.ctrl {
            self.run_diffcalc(vec![self.mode.0.clone()].into_iter(), true);
        }

        if key == Escape {
            // let menu = game.menus.get("main").unwrap().clone();
            match &self.select_action {
                BeatmapSelectAction::PlayMap => game.queue_state_change(GameState::InMenu(Box::new(MainMenu::new().await))),
                BeatmapSelectAction::OnComplete(sender) => sender.send(None).await.unwrap(),
            }
            return;
        }
        if key == F5 {
            if mods.ctrl {
                NotificationManager::add_text_notification("Doing a full refresh, the game will freeze for a bit", 5000.0, Color::RED).await;
                tokio::spawn(async {
                    BEATMAP_MANAGER.write().await.full_refresh().await;
                });
            } else {
                self.refresh_maps().await;
            }
            return;
        }

        // mode change
        if mods.alt {
            match key {
                Key1 => GlobalValueManager::update(Arc::new(CurrentPlaymode("osu".to_owned()))),
                Key2 => GlobalValueManager::update(Arc::new(CurrentPlaymode("taiko".to_owned()))),
                // Key3 => Some("catch".to_owned()),
                Key4 => GlobalValueManager::update(Arc::new(CurrentPlaymode("mania".to_owned()))),
                _ => {}
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
                self.refresh_maps().await;
                // self.set_selected_mode(self.mode.clone()).await;

                NotificationManager::add_text_notification(&format!("Map speed: {:.2}x", speed), 2000.0, Color::BLUE).await;
            }
        }

        // if enter was hit, or a beatmap item was updated
        if self.beatmap_scroll.on_key_press(key, mods) || key == Return {
            if let Some(selected_index) = self.beatmap_scroll.get_selected_index() {
                if let Some(item) = self.beatmap_scroll.items.get(selected_index) {
                    let hash = item.get_tag().try_into().unwrap();
                    self.select_map(game, hash, key == Return).await;
                }
            }
        }
        

        self.top_bar.on_key_press(key, mods);
    }

    async fn on_key_release(&mut self, key:Key, _game:&mut Game) {
        self.top_bar.on_key_release(key);
    }

    async fn on_text(&mut self, text:String) {
        self.top_bar.on_text(text.clone());
        // self.apply_filter().await;
    }


    async fn controller_down(&mut self, game:&mut Game, _controller: &GamepadInfo, button: ControllerButton) -> bool {
        match button {
            ControllerButton::DPadUp => self.on_key_press(Key::Up, game, KeyModifiers::default()).await,
            ControllerButton::DPadDown => self.on_key_press(Key::Down, game, KeyModifiers::default()).await,
            ControllerButton::DPadLeft|ControllerButton::LeftTrigger => self.on_key_press(Key::Left, game, KeyModifiers::default()).await,
            ControllerButton::DPadRight|ControllerButton::RightTrigger => self.on_key_press(Key::Right, game, KeyModifiers::default()).await,
            ControllerButton::South => self.on_key_press(Key::Return, game, KeyModifiers::default()).await,
            ControllerButton::East => game.queue_state_change(GameState::InMenu(Box::new(MainMenu::new().await))),
            _ => {}
        }

        false
    }

    async fn controller_axis(&mut self, _game:&mut Game, _controller: &GamepadInfo, axis_data: HashMap<Axis, (bool, f32)>) -> bool {
        for (axis, (_, val)) in axis_data {
            if axis == Axis::RightStickY && val.abs() > 0.1 {
                self.beatmap_scroll.set_hover(true);
                self.beatmap_scroll.on_scroll(-val / 16.0);
            }
        }

        false
    }

}


pub enum BeatmapSelectAction {
    PlayMap,
    OnComplete(tokio::sync::mpsc::Sender<Option<(Arc<BeatmapMeta>, String)>>)
}

