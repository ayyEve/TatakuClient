use crate::prelude::*;

// // constants
// const INFO_BAR_HEIGHT:f32 = 60.0;
// const DRAG_THRESHOLD:f32 = 50.0;
// const DRAG_FACTOR:f32 = 10.0;

// const DEFAULT_WIDTH: f32 = 1270.0;
// const DEFAULT_HEIGHT: f32 = 768.0;

const SPEED_DIFF:f32 = 0.05;

pub struct BeatmapSelectMenu {
    action_queue: ActionQueue,
    
    current_scores: HashMap<String, IngameScore>,

    score_loader: Option<Arc<AsyncRwLock<ScoreLoaderHelper>>>,

    // /// is changing, update loop detected that it was changing, how long since it changed
    // map_changing: (bool, bool, u32),

    // drag: Option<DragData>,
    // mouse_down: bool

    // /// internal search box
    // search_text: TextInput,

    sort_method: SortBy,
    filter_text: String,

    // sort_by_dropdown: Dropdown<SortBy>,
    // playmode_dropdown: Dropdown<PlayModeDropdown>,
    // leaderboard_method_dropdown: Dropdown<ScoreRetreivalMethod>,

    // /// drag_start, confirmed_drag, last_checked, mods_when_clicked
    // /// drag_start is where the original click occurred
    // /// confirmed_drag is if the drag as passed a certain threshhold. important if the drag returns to below the threshhold
    // mouse_down: Option<(Vector2, bool, MouseButton, Vector2, KeyModifiers)>,

    // window_size: Arc<WindowSize>,
    settings: SettingsHelper,
    mods: ModManagerHelper,
    current_skin: CurrentSkinHelper,
    new_beatmap_helper: LatestBeatmapHelper,
    diffcalc_complete: Option<Bomb<()>>,
    // key_events: KeyEventsHandlerGroup<BeatmapSelectKeyEvent>,

    menu_game: GameplayPreview,
    cached_maps: Vec<Vec<Arc<BeatmapMeta>>>,
    visible_sets: Vec<BeatmapSetComponent>,
    updates: usize,

    selected_set: usize,
    selected_map: usize,

    pub select_action: BeatmapSelectAction
}
impl BeatmapSelectMenu {
    pub async fn new() -> BeatmapSelectMenu {
        let settings = SettingsHelper::new();
        let sort_by = settings.last_sort_by;

        GlobalValueManager::update(Arc::new(CurrentPlaymode(settings.last_played_mode.clone())));

        // let leaderboard_method = SCORE_HELPER.read().await.current_method;
        // let leaderboard_method_dropdown = Dropdown::new(
        //     Vector2::new(410.0, 5.0),
        //     200.0,
        //     15.0,
        //     "Leaderboard",
        //     Some(leaderboard_method),
        //     Font::Main
        // );


        // let mut beatmap_scroll = ScrollableArea::new(
        //     Vector2::new(window_size.x - BEATMAPSET_ITEM_SIZE.x, INFO_BAR_HEIGHT),
        //     Vector2::new(window_size.x - LEADERBOARD_ITEM_SIZE.x, window_size.y - INFO_BAR_HEIGHT),
        //     ListMode::VerticalList
        // );
        // beatmap_scroll.dragger = DraggerSide::Right(10.0, true);
        // beatmap_scroll.set_item_margin(7.0);
        // beatmap_scroll.ui_scale_changed(Vector2::ONE * scale);

        let mut b = BeatmapSelectMenu {
            action_queue: ActionQueue::new(),

            // pending_refresh: false,
            // map_changing: (false, false, 0),
            current_scores: HashMap::new(),
            // back_button: MenuButton::back_button(window_size.0, Font::Main),

            // beatmap_scroll,
            // leaderboard_scroll: ScrollableArea::new(LEADERBOARD_POS, Vector2::new(LEADERBOARD_ITEM_SIZE.x, window_size.y - (LEADERBOARD_PADDING + INFO_BAR_HEIGHT)), ListMode::VerticalList),
            // search_text: TextInput::new(Vector2::new(window_size.x - (window_size.x / 4.0), 0.0), Vector2::new(window_size.x / 4.0, INFO_BAR_HEIGHT), "Search", "", Font::Main),

            mods: ModManagerHelper::new(),
            sort_method: sort_by,
            filter_text: String::new(),

            score_loader: None,

            visible_sets: Vec::new(),

            new_beatmap_helper: LatestBeatmapHelper::new(),
            current_skin: CurrentSkinHelper::new(),
            select_action: BeatmapSelectAction::PlayMap,
            // key_events: KeyEventsHandlerGroup::new(),

            // mouse_down: None,
            menu_game: GameplayPreview::new(true, true, Arc::new(|s|s.background_game_settings.beatmap_select_enabled)),
            settings,
            cached_maps: Vec::new(),
            updates: 0,
            selected_set: 0,
            selected_map: 0,
            
            diffcalc_complete: None,
        };
        b.refresh_maps().await;

        b
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
        self.visible_sets.clear();
        self.menu_game.current_beatmap.update();
        let current_beatmap = self.get_beatmap();

        // self.beatmap_scroll.clear();
        let filter_text = self.filter_text.to_ascii_lowercase();

        let mods = self.mods.clone();
        let mode = self.get_playmode(); //Arc::new(self.mode.clone());
        let mut modes_needing_diffcalc = HashSet::new();

        // used to select the current map in the list
        let current_hash = current_beatmap.map(|m|m.beatmap_hash.clone()).unwrap_or_default();
        // let mut n = 0;

        let mut selected_set = 0;
        let mut selected_map = 0;

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

            let n = self.visible_sets.len();
            let meta = &maps[0];
            let display_text = format!("{} // {} - {}", meta.creator, meta.artist, meta.title);
            // let mut i = BeatmapsetItem::new(maps, display_text).await;
            let mut set_item = BeatmapSetComponent::new(display_text, n, maps).await;
            if let Some(map_num) = set_item.check_selected(current_hash) {
                selected_set = n;
                selected_map = map_num;
            }
            self.visible_sets.push(set_item);
        }

        // make sure the correct set and map are selected
        // this will also scroll to the selected set
        self.select_set(selected_set);
        self.select_map(selected_map);

        // sort
        macro_rules! sort {
            ($property:tt, String) => {
                self.visible_sets.sort_by(|a, b| a.maps[0].$property.to_lowercase().cmp(&b.maps[0].$property.to_lowercase()))
            };
            ($property:ident, Float) => {
                self.visible_sets.sort_by(|a, b| a.maps[0].$property.partial_cmp(&b.maps[0].$property).unwrap())
            }
        }
        match self.sort_method {
            SortBy::Title => sort!(title, String),
            SortBy::Artist => sort!(artist, String),
            SortBy::Creator => sort!(creator, String),
            SortBy::Difficulty => sort!(diff, Float),
        }
        // we need to renumber because the sort changes the numbers
        self.visible_sets
            .iter_mut()
            .enumerate()
            .for_each(|(n, s)|s.num = n);

        // self.visible_sets = full_list;
        // for i in full_list { 
        //     self.show_maps.push(i);
        //     // self.beatmap_scroll.add_item(i) 
        // }
        // self.beatmap_scroll.scroll_to_selection();

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
        let mode = CurrentPlaymodeHelper::new().0.clone();
        // if nothing is selected, leave
        if let Some(map) = &BEATMAP_MANAGER.read().await.current_beatmap {
            self.score_loader = Some(SCORE_HELPER.read().await.get_scores(map.beatmap_hash, &map.check_mode_override(mode)).await);

            // clear lists
            // self.leaderboard_scroll.clear();
            self.current_scores.clear();
        }
    }


    // async fn actual_on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
    //     if self.back_button.on_click(pos, button, mods) {
    //         match &self.select_action {
    //             BeatmapSelectAction::PlayMap => game.queue_state_change(GameState::InMenu(Box::new(MainMenu::new().await))),
    //             BeatmapSelectAction::OnComplete(sender) => sender.send(None).await.unwrap(),
    //         }
    //         return;
    //     }

    //     let mut dropdown_clicked = false;
    //     for i in self.interactables() {
    //         if i.on_click(pos, button, mods) {
    //             dropdown_clicked = true;
    //             break;
    //         }
    //     }


    //     // check if selected mode changed
    //     let mut new_mode = None;
    //     if let Some(PlayModeDropdown::Mode(selected_mode)) = &self.playmode_dropdown.value {
    //         if selected_mode != &self.mode {
    //             new_mode = Some(selected_mode.clone());
    //         }
    //     }
    //     if let Some(new_mode) = new_mode {
    //         self.set_selected_mode(new_mode).await;
    //     }

    //     // check sort by dropdown
    //     let mut map_refresh = false;
    //     if let Some(sort_by) = self.sort_by_dropdown.value {
    //         if sort_by != self.sort_method {
    //             self.sort_method = sort_by;
    //             map_refresh = true;
    //             Settings::get_mut().last_sort_by = sort_by;
    //         }
    //     }
    //     if map_refresh {
    //         self.refresh_maps().await
    //     }

    //     // check score dropdown
    //     let mut score_method_changed = false;
    //     if let Some(leaderboard_method) = self.leaderboard_method_dropdown.value {
    //         if SCORE_HELPER.read().await.current_method != leaderboard_method {
    //             SCORE_HELPER.write().await.current_method = leaderboard_method;
    //             score_method_changed = true;
    //             Settings::get_mut().last_score_retreival_method = leaderboard_method;
    //         }

    //     }
    //     if score_method_changed {
    //         self.load_scores().await
    //     }

    //     // dont continue if a dropdown click was handled
    //     // because the dropdown could be overlapping something below
    //     if dropdown_clicked {
    //         return;
    //     }


    //     // check if leaderboard item was clicked
    //     if let Some(score_tag) = self.leaderboard_scroll.on_click_tagged(pos, button, mods) {
    //         // score display
    //         if let Some(score) = self.current_scores.get(&score_tag) {
    //             let score = score.clone();

    //             if let Some(selected) = &BEATMAP_MANAGER.read().await.current_beatmap {
    //                 let menu = ScoreMenu::new(&score, selected.clone(), false);
    //                 game.queue_state_change(GameState::InMenu(Box::new(menu)));
    //             }
    //         }
    //         return;
    //     }


    //     // find the previously selected item
    //     let mut selected_index = None;
    //     for (i, item) in self.beatmap_scroll.items.iter().enumerate() {
    //         if item.get_selected() {
    //             selected_index = Some(i);
    //             break;
    //         }
    //     }

    //     // check if beatmap item was clicked
    //     if let Some(clicked_hash) = self.beatmap_scroll.on_click_tagged(pos, button, mods).and_then(|h|Md5Hash::try_from(h).ok()) {
    //         if button == MouseButton::Right {
    //             // clicked hash is the target
    //             let dialog = BeatmapDialog::new(clicked_hash.clone());
    //             game.add_dialog(Box::new(dialog), false);
    //         }

    //         self.select_map(game, clicked_hash, button == MouseButton::Left).await;
    //         return;
    //     }

    //     // if we got here, make sure a map is selected
    //     // TODO: can we do this a better way? probably not since individually each item wont know if it should deselect or not
    //     if let Some(i) = selected_index {
    //         if let Some(item) = self.beatmap_scroll.items.get_mut(i) {
    //             item.set_selected(true);
    //         }
    //         self.beatmap_scroll.refresh_layout();
    //     }

    //     for i in self.interactables() {
    //         i.on_click_release(pos, button)
    //     }
    // }

    fn get_beatmap_list(&self) -> IcedElement {
        use iced::Length;

        let helper = Helper {
            maps: self.visible_sets.clone(),
            updates: self.updates,
        };

        let name = self.get_name();
        iced::widget::Lazy::new(helper, move |helper| {
            make_scrollable(helper.maps.iter().map(|a|a.view(name)).collect(), "beatmap_scroll")
                .width(Length::FillPortion(2))
                .height(Length::Fill)
                .into_element()
        })
        .into_element()
    }

    async fn speed_step(&mut self, step: f32) {
        let old_speed = self.mods.get_speed();
        let speed = (old_speed + step).clamp(-10.0, 10.0);

        if speed != old_speed {
            ModManager::get_mut().set_speed(speed);
            self.action_queue.push(SongMenuAction::SetRate(speed));

            // AudioManager::get_song().await.ok_do(|song|song.set_rate(speed));
            NotificationManager::add_text_notification(format!("Map speed: {speed:.2}x"), 2000.0, Color::BLUE).await;

            // force diff recalc
            // TODO: is this needed?
        }
    }

    // menu event helpers
    fn select_set(&mut self, set_num: usize) {
        self.visible_sets.get_mut(self.selected_set).ok_do_mut(|set|set.selected = false);
        self.visible_sets.get_mut(set_num).ok_do_mut(|set|set.selected = true);
        
        self.selected_set = set_num;
        self.select_map(0);

        self.action_queue.push(MenuAction::PerformOperation(
            snap_to_id(
            "beatmap_scroll", 
            iced::widget::scrollable::RelativeOffset { 
                x: 0.0,
                y: set_num as f32 / self.visible_sets.len() as f32
            })
        ))
    }
    fn next_set(&mut self) {
        self.select_set(self.selected_set.wrapping_add_1(self.visible_sets.len()))
    }
    fn prev_set(&mut self) {
        self.select_set(self.selected_set.wrapping_sub_1(self.visible_sets.len()))
    }

    fn select_map(&mut self, map_num: usize)  {
        self.updates += 1;
        self.selected_map = map_num;

        let Some(set) = self.visible_sets.get(self.selected_set) else { return };
        if let Some(map) = set.maps.get(self.selected_map) {
            self.action_queue.push(BeatmapMenuAction::Set(map.meta.clone(), true))
        }
    }
    fn next_map(&mut self) {
        let Some(set) = self.visible_sets.get(self.selected_set) else { return };
        self.select_map(self.selected_map.wrapping_add_1(set.maps.len()))
    }
    fn prev_map(&mut self) {
        let Some(set) = self.visible_sets.get(self.selected_set) else { return };
        self.select_map(self.selected_map.wrapping_sub_1(set.maps.len()))
    }

    async fn play_map(&mut self) {
        let mode = self.get_playmode();
        let Some(map) = self.get_beatmap() else { return };

        match &self.select_action {
            BeatmapSelectAction::PlayMap => self.action_queue.push(BeatmapMenuAction::PlayMap(map, mode)),
            BeatmapSelectAction::Back => self.action_queue.push(MenuMenuAction::PreviousMenu(self.get_name())),
        }
    }
    
    async fn make_mods_dialog(&self) -> Box<dyn Dialog> {
        let mut groups = Vec::new();
        let mut playmode = self.get_playmode();

        if let Some(man) = &self.menu_game.manager {
            playmode = man.gamemode.playmode();
        } else if let Some(map) = self.get_beatmap() {
            playmode = map.check_mode_override(self.get_playmode());
        }

        if let Some(info) = get_gamemode_info(&playmode) {
            groups = info.get_mods();
        }

        Box::new(ModDialog::new(groups).await)
    }

    async fn toggle_mod(&self, m: GameplayMod) {
        let state = if ModManager::get_mut().toggle_mod(m) {"enabled"} else {"disabled"};
        NotificationManager::add_text_notification(format!("{} {state}", m.display_name), 2000.0, Color::BLUE).await
    }

    fn get_playmode(&self) -> String {
        self.menu_game.current_playmode.0.clone()
    }
    fn get_beatmap(&self) -> Option<Arc<BeatmapMeta>> {
        self.menu_game.current_beatmap.0.clone()
    }
}

#[async_trait]
impl AsyncMenu for BeatmapSelectMenu {
    fn get_name(&self) -> &'static str {"beatmap_select"}

    fn view(&self, _values: &mut ShuntingYardValues) -> IcedElement {
        use iced_elements::*;
        let gamemodes = AVAILABLE_PLAYMODES.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        let sort_bys = SortBy::list().iter().map(SortBy::to_string).collect::<Vec<_>>();

        let owner = MessageOwner::new_menu(self);
        col!(
            // top bar
            row!(
                // game mode
                Dropdown::new(gamemodes, Some(self.get_playmode()), move|value|Message::new(owner, "playmode", MessageType::Dropdown(value))).width(FillPortion(1)).text_size(20.0).into_element(),
                
                // sort by
                Dropdown::new(sort_bys, Some(self.settings.last_sort_by.to_string()), move|value|Message::new(owner, "sort_by", MessageType::Dropdown(value))).width(FillPortion(1)).text_size(20.0).into_element(),
                
                // search
                TextInput::new("Search", &self.filter_text).on_input(move|text|Message::new(owner, "search", MessageType::Text(text))).width(FillPortion(2));

                width = Fill,
                spacing = 5.0
            ),

            // main view
            row!(
                // leaderboard
                col!(
                    self.current_scores.values().enumerate().map(|(i, s)|LeaderboardComponent::new(i, s.clone()).view(self.get_name())).collect(),
                    width = Fill,
                    height = Fill
                ),
    
                // gameplay preview and beatmap list
                PanelScroll::with_children(vec![ 
                    // gameplay preview
                    col!(
                        self.menu_game.widget();
                        width = Fill,
                        height = Fill
                    ), 
    
                    // beatmaps
                    self.get_beatmap_list(),
                ]).width(FillPortion(5));
            );

            // // key events
            // self.key_events.handler();

            width = Fill,
            height = Fill
        )
    }

    async fn handle_message(&mut self, message: Message, _values: &mut ShuntingYardValues) {
        self.updates += 1;

        match (message.tag, message.message_type) {
            (MessageTag::Beatmap(b), _) => {
                if self.menu_game.current_beatmap.0.as_ref().unwrap().beatmap_hash == b.beatmap_hash {
                    self.play_map().await;
                } else {
                    self.action_queue.push(BeatmapMenuAction::Set(b, true));
                }
            }

            // set number changed
            (MessageTag::Number(n), _) => self.select_set(n),
            
            
            // dropdowns
            (MessageTag::String(tag), MessageType::Dropdown(value)) => {
                match &*tag {
                    "playmode" => GlobalValueManager::update(Arc::new(CurrentPlaymode(value))),
                    _ => {}
                }

            }
            
            // filter text
            (MessageTag::String(tag), MessageType::Text(text)) => {
                match &*tag {
                    "search" => {
                        self.filter_text = text;
                        self.apply_filter().await;
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }



    async fn update(&mut self, values: &mut ShuntingYardValues) -> Vec<MenuAction> {
        self.settings.update();
        self.mods.update();

        // update bg game
        self.menu_game.update(values, &mut self.action_queue).await;

        // // check for key events
        // while let Some(event) = self.key_events.check_events() {
        //     match event {
        //         KeyEvent::Press(BeatmapSelectKeyEvent::NextMap) => self.next_map(),
        //         KeyEvent::Press(BeatmapSelectKeyEvent::PrevMap) => self.prev_map(),

        //         KeyEvent::Press(BeatmapSelectKeyEvent::NextSet) => self.next_set(),
        //         KeyEvent::Press(BeatmapSelectKeyEvent::PrevSet) => self.prev_set(),

        //         KeyEvent::Press(BeatmapSelectKeyEvent::Enter) => self.play_map().await,
        //         KeyEvent::Press(BeatmapSelectKeyEvent::Back) => self.action_queue.push(MenuMenuAction::PreviousMenu(self.get_name())),

        //         KeyEvent::Press(BeatmapSelectKeyEvent::ModsDialog) => self.action_queue.push(MenuMenuAction::AddDialog(self.make_mods_dialog().await, false)),
                
        //         KeyEvent::Press(BeatmapSelectKeyEvent::SpeedUp) => self.speed_step(SPEED_DIFF).await,
        //         KeyEvent::Press(BeatmapSelectKeyEvent::SpeedDown) => self.speed_step(-SPEED_DIFF).await,
        //         KeyEvent::Press(BeatmapSelectKeyEvent::NoFailMod) => self.toggle_mod(NoFail).await,
        //         KeyEvent::Press(BeatmapSelectKeyEvent::AutoplayMod) => self.toggle_mod(Autoplay).await,

        //         #[allow(unreachable_patterns)]
        //         KeyEvent::Press(other) => error!("unhandled BeatmapSelectKeyEvent: {other:?}"),
                
        //         _ => {}
        //     };
        // }

        // check for diffcalc update
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
        }

        if self.new_beatmap_helper.update() {
            self.action_queue.push(BeatmapMenuAction::Set(self.new_beatmap_helper.0.clone(), true));
            // BEATMAP_MANAGER.write().await.set_current_beatmap(game, &self.new_beatmap_helper.0, true).await;
            refresh_pending = true;
            self.menu_game.setup().await;
        }

        // if old_text != self.search_text.get_text() {
        //     refresh_pending = true;
        // }

        if refresh_pending {
            self.refresh_maps().await;
        } else if filter_pending {
            self.apply_filter().await;
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
                    // self.leaderboard_scroll.add_item(Box::new(LeaderboardItem::new(s.to_owned()).load_image().await));
                }
            }
        }

        self.action_queue.take()
    }

    // async fn draw(&mut self, items: &mut RenderableCollection) {
    //     // draw a bar on the top for the info
    //     let bar_rect = Rectangle::new(
    //         Vector2::ZERO,
    //         Vector2::new(self.window_size.x, INFO_BAR_HEIGHT),
    //         Color::WHITE,
    //         Some(Border::new(Color::BLACK, 1.2))
    //     );
    //     items.push(bar_rect);

    //     // draw bg game
    //     self.menu_game.draw(items).await;

    //     // // draw selected map info
    //     // if let Some(meta) = &mut BEATMAP_MANAGER.write().current_beatmap {
    //     //     // draw map name top-most left-most
    //     //     items.push(Box::new(Text::new(
    //     //         Color::BLACK,
    //     //         -10.0,
    //     //         Vector2::new(0.0, 5.0),
    //     //         25,
    //     //         meta.version_string(),
    //     //         Font::Main
    //     //     )));

    //     //     // diff string, under map string
    //     //     items.push(Box::new(Text::new(
    //     //         Color::BLACK,
    //     //         -10.0,
    //     //         Vector2::new(0.0, 35.0),
    //     //         15,
    //     //         meta.diff_string(self.mode.clone(), &ModManager::get()),
    //     //         Font::Main
    //     //     )));
    //     // }

    //     // beatmap scroll
    //     self.beatmap_scroll.draw(Vector2::ZERO, items);

    //     // leaderboard scroll
    //     self.leaderboard_scroll.draw(Vector2::ZERO, items);

    //     // back button
    //     self.back_button.draw(Vector2::ZERO, items);

    //     // everything else
    //     for i in self.interactables() {
    //         i.draw(Vector2::ZERO, items);
    //     }

    // }

    // async fn on_change(&mut self, into:bool) {
    //     if !into { return }

    //     self.new_beatmap_helper.update();
    //     self.menu_game.setup().await;

    //     // update our window size
    //     self.window_size_changed(WindowSize::get()).await;

    //     OnlineManager::send_spec_frames(vec![SpectatorFrame::new(0.0, SpectatorAction::ChangingMap)], true);

    //     // play song if it exists
    //     if let Some(song) = AudioManager::get_song().await {
    //         // set any time mods
    //         song.set_rate(self.mods.get_speed());

    //         // ensure song is playing
    //         song.play(false);
    //     }

    //     // load maps
    //     self.refresh_maps().await;
    //     self.beatmap_scroll.refresh_layout();

    //     // if let Some(map) = &BEATMAP_MANAGER.read().await.current_beatmap {
    //         self.load_scores().await;
    //     // }

    // }

    // async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, _game:&mut Game) {
    //     // search text relies on this event, so if it consumed the event, ignore drag
    //     self.search_text.check_hover(pos);
    //     if self.search_text.get_hover() {
    //         if self.search_text.on_click(pos, button, mods) {
    //             // note: we shouldnt need to store mods, as search text in this instance doesnt care about it
    //             return;
    //         }
    //     }

    //     self.mouse_down = Some((pos, false, button, pos, mods));
    // }
    // async fn on_click_release(&mut self, pos:Vector2, button:MouseButton, game:&mut Game) {
    //     let mut was_hold = false;
    //     let mut mods = None;

    //     // if mouse_down is none, it means we got here from the special condition where
    //     // the search input absorbed the on_click.
    //     // therefor, perform the on_release only for the search input
    //     if self.mouse_down.is_none() {
    //         self.search_text.on_click_release(pos, button);
    //         return
    //     }


    //     if let Some((_, was_drag, button, _, click_mods)) = self.mouse_down {
    //         if was_drag {
    //             mods = Some((click_mods, button));
    //             was_hold = true;
    //         }
    //     }
    //     self.mouse_down = None;


    //     // perform actual on_click
    //     // this is here because on_click is now only used for dragging
    //     let (mods, button) = mods.unwrap_or((KeyModifiers::default(), button));
    //     if !was_hold {
    //         self.actual_on_click(pos, button, mods, game).await;
    //     }

    // }

    // async fn on_mouse_move(&mut self, pos:Vector2, game:&mut Game) {
    //     let mut scroll_pos = 0.0;
    //     if let Some((drag_pos, confirmed_drag, button_pressed, last_checked, _)) = &mut self.mouse_down {
    //         if *confirmed_drag || (pos.y - drag_pos.y).abs() >= DRAG_THRESHOLD  {
    //             *confirmed_drag |= true;

    //             if *button_pressed == MouseButton::Right {
    //                 let offset_pos = self.beatmap_scroll.get_pos();
    //                 let comp_size = self.beatmap_scroll.size();
    //                 let y_percent = ((pos.y - offset_pos.y) / comp_size.y).clamp(0.0, 1.0);

    //                 let items_height = self.beatmap_scroll.get_elements_height();
    //                 self.beatmap_scroll.scroll_pos = -items_height * y_percent;
    //             } else {
    //                 let dist = (pos.y - last_checked.y) / DRAG_FACTOR;
    //                 scroll_pos = dist;
    //             }
    //         }

    //         *last_checked = pos;
    //     }
    //     // drag acts like scroll
    //     if scroll_pos != 0.0 {
    //         self.on_scroll(scroll_pos, game).await
    //     }

    //     for i in self.interactables() {
    //         i.on_mouse_move(pos)
    //     }
    //     self.back_button.on_mouse_move(pos);
    //     self.beatmap_scroll.on_mouse_move(pos);
    //     self.leaderboard_scroll.on_mouse_move(pos);
    // }
    // async fn on_scroll(&mut self, delta:f32, _game:&mut Game) {
    //     let mut h = false;

    //     h |= self.beatmap_scroll.on_scroll(delta);
    //     h |= self.leaderboard_scroll.on_scroll(delta);

    //     for i in self.interactables() {
    //         h |= i.on_scroll(delta);
    //     }

    //     if !h {
    //         // make the scroll think its hovered
    //         self.beatmap_scroll.set_hover(true);
    //         // try again
    //         self.beatmap_scroll.on_scroll(delta);
    //     }
    // }

    // async fn on_key_press(&mut self, key:Key, game:&mut Game, mods:KeyModifiers) {
    //     use Key::*;

    //     if key == Key::M && mods.ctrl {
    //         let mut found = false;
    //         for d in game.dialogs.iter_mut() {
    //             if d.name() == "mod_menu" {
    //                 found = true;
    //                 break;
    //             }
    //         }

    //         if !found {
    //             let mut groups = Vec::new();
    //             let mut playmode = self.mode.clone();

    //             if let Some(man) = &self.menu_game.manager {
    //                 playmode = man.gamemode.playmode();
    //             } else if let Some(map) = BEATMAP_MANAGER.read().await.current_beatmap.clone() {
    //                 playmode = map.check_mode_override(self.mode.clone());
    //             }

    //             if let Some(info) = get_gamemode_info(&playmode) {
    //                 groups = info.get_mods();
    //             }

    //             game.add_dialog(Box::new(ModDialog::new(groups).await), false)
    //         }
    //     }

    //     if key == Left && !mods.alt {
    //         if let Some(hash) = self.beatmap_scroll.select_previous_item().and_then(|h|h.try_into().ok()) {
    //             self.select_map(game, hash, false).await;
    //             self.beatmap_scroll.scroll_to_selection();
    //         }
    //     }
    //     if key == Right && !mods.alt  {
    //         if let Some(hash) = self.beatmap_scroll.select_next_item().and_then(|h|h.try_into().ok()) {
    //             self.select_map(game, hash, false).await;
    //             self.beatmap_scroll.scroll_to_selection();
    //         }
    //     }

    //     if key == F7 && mods.ctrl {
    //         self.run_diffcalc(vec![self.mode.clone()].into_iter(), true);
    //     }

    //     if key == Escape {
    //         // let menu = game.menus.get("main").unwrap().clone();
    //         match &self.select_action {
    //             BeatmapSelectAction::PlayMap => game.queue_state_change(GameState::InMenu(Box::new(MainMenu::new().await))),
    //             BeatmapSelectAction::OnComplete(sender) => sender.send(None).await.unwrap(),
    //         }
    //         return;
    //     }
    //     if key == F5 {
    //         if mods.ctrl {
    //             NotificationManager::add_text_notification("Doing a full refresh, the game will freeze for a bit", 5000.0, Color::RED).await;
    //             tokio::spawn(async {
    //                 BEATMAP_MANAGER.write().await.full_refresh().await;
    //             });
    //         } else {
    //             self.refresh_maps().await;
    //         }
    //         return;
    //     }

    //     // mode change
    //     if mods.alt {
    //         let new_mode = match key {
    //             Key1 => Some("osu".to_owned()),
    //             Key2 => Some("taiko".to_owned()),
    //             // Key3 => Some("catch".to_owned()),
    //             Key4 => Some("mania".to_owned()),
    //             _ => None
    //         };

    //         if let Some(new_mode) = new_mode {
    //             self.set_selected_mode(new_mode.clone()).await;
    //             let display = gamemode_display_name(&new_mode);
    //             NotificationManager::add_text_notification(&format!("Mode changed to {}", display), 1000.0, Color::BLUE).await;
    //             self.mode = new_mode;
    //             self.load_scores().await;
    //         }
    //     }
    // }


}


/// what to do once a beatmap has been selected
pub enum BeatmapSelectAction {
    PlayMap,
    Back
}


/// lazy helper, should probably rename this
struct Helper {
    maps: Vec<BeatmapSetComponent>,
    updates: usize,
}
impl std::hash::Hash for Helper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.updates.hash(state);
    }
}


// #[derive(Debug)]
// enum BeatmapSelectKeyEvent {
//     PrevMap,
//     NextMap,

//     NextSet,
//     PrevSet,

//     Enter,
//     Back,

//     SpeedUp,
//     SpeedDown,

//     ModsDialog,
//     NoFailMod,
//     AutoplayMod,
// }
// impl KeyMap for BeatmapSelectKeyEvent {
//     fn from_key(key: iced::keyboard::KeyCode, mods: iced::keyboard::Modifiers) -> Option<Self> {
//         use iced::keyboard::KeyCode;
//         match key {
//             KeyCode::Enter => Some(Self::Enter),
//             KeyCode::Escape => Some(Self::Back),

//             KeyCode::Up if mods.alt() => Some(Self::SpeedUp),
//             KeyCode::Down if mods.alt() => Some(Self::SpeedDown),

//             KeyCode::Left => Some(Self::PrevSet),
//             KeyCode::Right => Some(Self::NextSet),

//             KeyCode::Up => Some(Self::PrevMap),
//             KeyCode::Down => Some(Self::NextMap),

//             KeyCode::M if mods.control() => Some(Self::ModsDialog),
//             KeyCode::A if mods.control() => Some(Self::AutoplayMod),
//             KeyCode::N if mods.control() => Some(Self::NoFailMod),

//             _ => None,
//         }
//     }
// }
