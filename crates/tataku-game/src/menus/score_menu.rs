use crate::prelude::*;
use crate::REPLAY_EXPORTS_DIR;
use chrono::{ NaiveDateTime, Local };

// const MENU_ITEM_COUNT:usize = 2;
// const TITLE_STRING_Y:f32 = 20.0;
// const TITLE_STRING_FONT_SIZE:f32 = 30.0;

pub struct ScoreMenu {
    actions: ActionQueue,
    infos: GamemodeInfos,
    // key_handler: KeyEventsHandlerGroup<ScoreMenuKeys>,

    score: IngameScore,
    beatmap: Arc<BeatmapMeta>,
    // pub replay: Option<Replay>,

    menu_type: Box<ScoreMenuType>,


    /// can the user retry?
    allow_retry: bool,

    // cached
    score_mods: String,
    hit_error: HitError,
    hit_counts: Vec<(String, u32, Color)>,
    stats: Vec<MenuStatsInfo>,

    // pub dont_close_on_back: bool,
    // pub should_close: bool,

    /// what stat is selected?
    selected_stat: usize,




    // score submit stuff
    pub score_submit: Option<Arc<ScoreSubmitHelper>>,
    score_submit_response: Option<SubmitResponse>,


    // lobby stuff
    // /// is this score menu being shown in a lobby?
    // is_lobby: bool,
    // lobby_helper: CurrentLobbyDataHelper,
    // lobby_items: Vec<LeaderboardComponent>,
    // close_sender: Option<AsyncSender<()>>,
}
impl ScoreMenu {
    pub fn new(
        score: &IngameScore, 
        beatmap: Arc<BeatmapMeta>, 
        allow_retry: bool,
        infos: GamemodeInfos,
    ) -> ScoreMenu {
        let hit_error = score.hit_error();

        let judgments = infos
            .get_info(&score.playmode)
            .map(|i| i.judgments)
            .unwrap_or_default();
        
        // map hit types to a display string
        let mut hit_counts = Vec::new();
        for judge in judgments.iter() {
            let txt = judge.display_name;
            if txt.is_empty() { continue }

            let count = score.judgments.get(judge.id).copied().unwrap_or_default();

            let mut color = judge.color;
            if color.a == 0.0 { color = Color::BLACK }

            hit_counts.push((txt.to_owned(), count as u32, color));
        }

        let mut score_mods = String::new();
        let mut stats = Vec::new();
        if let Ok(gamemode_info) = infos.get_info(&score.playmode) {

            // extract mods
            score_mods = ModManager::short_mods_string(&score.mods, false, gamemode_info);
            if !score_mods.is_empty() { score_mods = format!("Mods: {score_mods}"); }


            let mut groups = gamemode_info.stat_groups.to_vec();
            groups.extend(default_stat_groups());
            let data = score.stats_into_groups(&groups);

            stats = default_stats_from_groups(&data);
            stats.extend(gamemode_info.stats_from_groups(&data));
        }

        ScoreMenu {
            actions: ActionQueue::new(),
            infos,
            // key_handler: KeyEventsHandlerGroup::new(),
            menu_type: Box::new(ScoreMenuType::Normal),
                
            score: score.clone(),
            score_mods,
            // replay: None,
            beatmap,
            hit_error,

            // dont_close_on_back: false,
            // should_close: false,
            allow_retry,

            hit_counts,
            score_submit: None,
            score_submit_response: None,

            selected_stat: 0,
            stats,

            // is_lobby: false,
            // lobby_helper: CurrentLobbyDataHelper::new(),
            // lobby_items: Vec::new(),
            // close_sender: None,
        }
    }

    async fn close(&mut self) {
        self.actions.push(MenuAction::PreviousMenu(self.get_name()));

        // let menu: Box<dyn AsyncMenu>;
        // match &*self.menu_type {
        //     ScoreMenuType::Normal => menu = Box::new(BeatmapSelectMenu::new().await),
        //     ScoreMenuType::Multiplayer { .. } => menu = Box::new(LobbyMenu::new().await),
        //     ScoreMenuType::Spectator { .. } => menu = Box::new(SpectatorMenu::new()),
        // }
        // self.actions.push(MenuAction::SetMenu(menu));

        // if self.dont_close_on_back {
        //     self.should_close = true;
        //     return;
        // }
    }

    async fn replay(&mut self, settings: &Settings) {
        if self.score.replay.is_some() {
            self.do_replay((*self.score).clone()).await;
        } else {
            match self.score.get_replay(settings).await {
                Ok(score) => self.do_replay(score).await,
                Err(e) => self.actions.push(GameAction::AddNotification(Notification::new_error("Error loading replay", e))),
            }
        }
    }

    async fn do_replay(&mut self, score: Score) {
        // make sure the replay has score data
        // i dont think it should ever not, but just in case
        // if replay.score_data.is_none() {
        //     replay.score_data = Some(self.score.score.clone());
        // }

        self.actions.push(GameAction::WatchReplay(Box::new(score)));
        // match manager_from_playmode(self.score.playmode.clone(), &self.beatmap).await {
        //     Ok(mut manager) => {
        //         manager.set_replay(replay);
        //     }
        //     Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e).await
        // }
    }

    async fn retry(&mut self) {
        self.actions.push(BeatmapAction::PlaySelected);
        // self.actions.push(BeatmapAction::PlayMap(self.beatmap.clone(), self.score.playmode.clone()));
    }
    
    async fn change_score(&mut self, score: IngameScore) {
        self.hit_error = score.hit_error();

        let judgments = self.infos.get_info(&score.playmode).map(|i| i.judgments).unwrap_or_default();
        
        // map hit types to a display string
        self.hit_counts.clear();
        for judge in judgments.iter() {
            let txt = judge.display_name;
            if txt.is_empty() { continue }

            let count = score.judgments.get(judge.id).copied().unwrap_or_default();

            let mut color = judge.color;
            if color.a == 0.0 { color = Color::BLACK }

            self.hit_counts.push((txt.to_owned(), count as u32, color));
        }

        // extract mods
        // self.score_mods = ModManager::short_mods_string(&score.mods, false, &score.playmode);
        // if self.score_mods.len() > 0 { self.score_mods = format!("Mods: {}", self.score_mods); }

        if let Ok(gamemode_info) = self.infos.get_info(&score.playmode) {
            // mods
            self.score_mods = ModManager::short_mods_string(&score.mods, false, gamemode_info);
            if !self.score_mods.is_empty() { self.score_mods = format!("Mods: {}", self.score_mods); }
            
            // stats
            let mut groups = gamemode_info.stat_groups.to_vec();
            groups.extend(default_stat_groups().clone());
            let data = score.stats_into_groups(&groups);

            self.stats = default_stats_from_groups(&data);
            self.stats.extend(gamemode_info.stats_from_groups(&data));
        }

        self.score = score;
    }

    pub fn make_lobby(&mut self) {
        // self.is_lobby = true;
        // self.dont_close_on_back = true;
        // self.close_sender = Some(close_sender);
        self.menu_type = Box::new(ScoreMenuType::Multiplayer { 
            // lobby_helper: CurrentLobbyDataHelper::new(),
            lobby_items: Vec::new(),
        });

        // self.update_lobby();
    }
    fn update_lobby(&mut self, _values: &mut dyn Reflect) {
        let ScoreMenuType::Multiplayer { 
            // lobby_helper, 
            lobby_items 
        } = &mut *self.menu_type else { return };

        // TODO: read from values

        // lobby_items.clear();
        // let mut scores = lobby.player_scores.iter().collect::<Vec<_>>();
        // scores.sort_by(|(_,a), (_,b)| b.score.cmp(&a.score));

        // for (n, (user_id, score)) in scores.iter().enumerate() {
        //     let score = IngameScore::new((*score).clone(), user_id == &&lobby.our_user_id, false);
        //     lobby_items.push(LeaderboardComponent::new(n, score));
        //     // self.lobby_scrollable.add_item(Box::new(LeaderboardItem::new()))
        // }
        
    }
  
    async fn save_replay(&mut self) {
        let Some(replay) = &self.score.replay else { 
            NotificationManager::add_text_notification("There is no replay to save!", 5_000.0, Color::RED).await;
            return;
        };
        
        // save the replay
        match save_replay(&self.score) {
            Ok(saved_path) => {
                let saved_path = Path::new(&saved_path);

                let BeatmapMeta { artist, title, version, .. } = &*self.beatmap;
                let Score { playmode, username, time, .. } = &self.score.score;
                let playmode = self.infos.get_info(playmode).unwrap().display_name;

                let mut date = String::new();
                if let Some(datetime) = chrono::DateTime::from_timestamp(*time as i64, 0) {
                    let score_time = datetime.with_timezone(&Local);
                    date = score_time.date_naive().format("%d-%m-%Y").to_string();
                }

                let export_path = format!("{REPLAY_EXPORTS_DIR}/") + &Io::sanitize_filename(format!("{username}[{playmode}] - {artist} - {title} [{version}] ({date}).ttkr"));
                let export_path = Path::new(&export_path);

                // ensure export dir exists
                match std::fs::create_dir_all(export_path.parent().unwrap()) {
                    Ok(_) => {
                        // copy the file from the saved_path to the exports file
                        if let Err(e) = std::fs::copy(saved_path, export_path) {
                            NotificationManager::add_error_notification("Error exporting replay", e).await;
                        } else {
                            NotificationManager::add_text_notification("Replay exported!", 5000.0, Color::BLUE).await;
                        }
                    }
                    Err(e) => NotificationManager::add_error_notification("Error creating exports directory", e).await,
                }
            }
            Err(e) => NotificationManager::add_error_notification("Error saving replay", e).await,
        };
    }



    fn score_lines(&self) -> Vec<IcedElement> {
        use crate::prelude::iced_elements::*;

        let mut lines = Vec::with_capacity(20);
        let font_size = 30.0;

        macro_rules! add {
            ($s: expr, $color: expr) => {
                lines.push(
                    Text::new($s)
                        .color($color)
                        .size(font_size)
                        .width(Fill)
                        .into_element()
                )
            };

            ($s: expr) => {
                lines.push(Space::new(Fill, Fixed($s)).into_element());
            }
        }

        add!(format!("Score: {}", format_number(self.score.score.score)), Color::BLACK);

        // draw hit counts
        for (str, count, color) in self.hit_counts.iter() {
            add!(format!("{str}: {}", format_number(*count)), *color);
        }

        add!(font_size / 2.0);
        let info = self.infos.get_info(&self.score.playmode).unwrap();

        for str in [
            format!("Combo: {}x, {:.2}%", format_number(self.score.max_combo), info.calc_acc(&self.score) * 100.0),
            String::new(),
            format!("Mean: {:.2}ms", self.hit_error.mean),
            format!("Error: {:.2}ms - {:.2}ms avg", self.hit_error.early, self.hit_error.late),
            format!("Deviance: {:.2}ms", self.hit_error.deviance),
            if !self.score.speed.is_default() {format!("Speed: {:.2}x", self.score.speed)} else { String::new() },
            // format!("Expected Performance: {:.2}pr", self.score.score.performance),
            self.score_mods.clone(),
        ] {
            if !str.is_empty() {
                if !str.contains("NaN") {
                    add!(str, Color::BLACK);
                } else {
                    add!(font_size);
                }
            } else {
                add!(font_size / 2.0);
            }
        }

        if let Some(sub) = &self.score_submit_response {
            add!(font_size / 2.0);

            match sub {
                SubmitResponse::NotSubmitted(_, str) => {
                    add!(format!("Score not submitted: {str}"), Color::BLACK);
                }

                SubmitResponse::Submitted { score_id:_, placing, performance_rating } => {
                    for str in [
                        format!("Map Ranking: #{}", format_number(*placing)),
                        format!("Performance: {}pr", format_float(*performance_rating, 2)),
                    ] {
                        add!(str, Color::BLACK);
                        add!(font_size);
                    }
                }
            }
        }

        lines
    }

    fn get_stats_view(&self) -> IcedElement {
        use crate::prelude::iced_elements::*;

        // draw stats graphs
        if let Some(stat) = self.stats.get(self.selected_stat) {
            // const PAD:f32 = 20.0;
            // let pos = Vector2::new(self.window_size.x / 2.0, TITLE_STRING_Y + TITLE_STRING_FONT_SIZE + PAD);
            // let size = Vector2::new(self.window_size.x * 2.0/3.0 - pos.x, self.window_size.y - (pos.y + PAD * 2.0));

            // let bounds = Bounds::new(pos, size);
            // stat.draw(&bounds, list)
            stat.view()
        } else {
            Column::new().width(Fill).height(Fill).into_element()
        }
    }

    fn get_buttons(&self) -> Vec<IcedElement> {
        use iced::widget::Text;
        use iced::widget::Button;
        let mut buttons = Vec::with_capacity(2);
        
        // retry button
        if self.allow_retry {
            buttons.push(
                Button::new(Text::new("Retry"))
                    .on_press(Message::click(MessageOwner::Menu, "retry"))
                    .into_element()
            );
        }

        // replay button
        if !self.menu_type.is_lobby() {
            buttons.push(
                Button::new(Text::new("Replay"))
                    .on_press(Message::click(MessageOwner::Menu, "replay"))
                    .into_element()
            );
        }

        buttons.push(
            Button::new(Text::new("Back"))
                .on_press(Message::new(MessageOwner::Menu, "back", MessageType::Click))
                .into_element()
        );

        buttons
    }
}

#[async_trait]
impl AsyncMenu for ScoreMenu {
    fn get_name(&self) -> &'static str { "score" }

    async fn update(&mut self, values: &mut dyn Reflect) -> Vec<TatakuAction> {
        if self.score_submit_response.is_none() {
            if let Some(t) = &self.score_submit {
                if let Some(r) = t.response.read().await.as_ref() {
                    self.score_submit_response = Some(r.clone());
                }
            }
        }

        // update lobby scores
        self.update_lobby(values);
        
        // while let Some(event) = self.key_handler.check_events() {
        //     match event {
        //         KeyEvent::Press(ScoreMenuKeys::Back) => self.close().await,
        //         KeyEvent::Press(ScoreMenuKeys::SaveReplay) => self.save_replay().await,

        //         KeyEvent::Press(ScoreMenuKeys::PrevStat) if !self.stats.is_empty() => self.selected_stat = self.selected_stat.wrapping_sub_1(self.stats.len()),
        //         KeyEvent::Press(ScoreMenuKeys::NextStat) if !self.stats.is_empty() => self.selected_stat = self.selected_stat.wrapping_add_1(self.stats.len()),
        //         _ => {}
        //     }
        // }

        self.actions.take()
    }

    
    fn view(&self, _values: &mut dyn Reflect) -> IcedElement {
        use crate::prelude::iced_elements::*;

        // score info
        let beatmap_label = format!("{} ({}) (x{:.2})", self.beatmap.version_string(), self.infos.get_info(&self.score.playmode).unwrap().display_name, self.score.speed);
        col!(
            // beatmap label
            Text::new(beatmap_label).width(Fill),
            
            // data
            row!(
                // score info
                row!(
                    // score values
                    col!(
                        self.score_lines(),
                        width = FillPortion(2),
                        height = Fill
                    ),

                    // stats
                    self.get_stats_view();

                    width = FillPortion(2),
                    height = Fill
                ),

                // multi scores
                if let ScoreMenuType::Multiplayer {lobby_items, ..} = &*self.menu_type {
                    col!(
                        lobby_items.iter().map(|l| l.view()).collect::<Vec<_>>(),
                        width = FillPortion(1),
                        height = Fill,
                        align_x = Alignment::End
                    )
                } else {
                    EmptyElement.into_element()
                };

                width = Fill,
                height = Fill
            ),

            // buttons
            col!(
                self.get_buttons(),
                width = Fill,
                height = Shrink
            );

            // // key event helper
            // self.key_handler.handler();

            width = Fill,
            height = Fill
        )
    }
    
    async fn handle_message(&mut self, message: Message, values: &mut dyn Reflect) {
        let Some(tag) = message.tag.as_string() else { return };
        match &*tag {
            "retry" => self.retry().await,
            "replay" => self.replay(&values.reflect_get::<Settings>("settings").unwrap()).await,
            "back" => self.close().await,
            "score" => if let MessageType::Number(num) = message.message_type {
                if let ScoreMenuType::Multiplayer { lobby_items, .. } = &*self.menu_type {
                    if let Some(score) = lobby_items.get(num) {
                        self.change_score(score.score.clone()).await;
                    }
                }
            }
            _ => {}
        }
    }

    // async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
    //     #[cfg(feature="graphics")]
    //     if let Some(score_hash) = self.lobby_scrollable.on_click_tagged(pos, button, mods) {
    //         let Some(lobby) = &**self.lobby_helper else { return };
    //         let Some(score) = lobby.player_scores.values().find(|s|s.hash() == score_hash) else { return };
    //         self.change_score(IngameScore::new(score.clone(), false, false)).await;
    //     }
    // }

}



// pub enum ScoreMenuKeys {
//     Back,

//     SaveReplay,

//     NextStat,
//     PrevStat,
// }
// impl KeyMap for ScoreMenuKeys {
//     fn from_key(key: iced::keyboard::KeyCode, _mods: iced::keyboard::Modifiers) -> Option<Self> {
//         match key {
//             iced::keyboard::KeyCode::F2 => Some(Self::SaveReplay),
//             iced::keyboard::KeyCode::Escape => Some(Self::Back),

//             iced::keyboard::KeyCode::Left => Some(Self::PrevStat),
//             iced::keyboard::KeyCode::Right => Some(Self::NextStat),

//             _ => None,
//         }
//     }
// }



enum ScoreMenuType {
    Normal,
    Multiplayer {
        // lobby_helper: CurrentLobbyDataHelper,
        lobby_items: Vec<LeaderboardComponent>,
    },
}
impl ScoreMenuType {
    fn is_lobby(&self) -> bool {
        matches!(self, Self::Multiplayer { .. })
    }
}



#[cfg(feature="graphics")]
pub fn default_stats_from_groups(data: &HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<MenuStatsInfo> { 
    let mut info = Vec::new();

    if let Some(variance) = data.get(&VarianceStatGroup.name()) {
        if let Some(variance_values) = variance.get(&HitVarianceStat.name()) {
            let mut list = Vec::new();

            let mut late_total = 0.0;
            let mut early_total = 0.0;
            let mut total_all = 0.0;
            let mut late_count = 0;
            let mut early_count = 0;
            for i in variance_values {
                total_all += i;

                if *i > 0.0 {
                    late_total += i;
                    late_count += 1;
                } else {
                    early_total += i;
                    early_count += 1;
                }
            }

            let mean = total_all / variance_values.len() as f32;
            let early = early_total / early_count as f32;
            let late = late_total / late_count as f32;

            list.push(MenuStatsEntry::new_list("Variance", variance_values.clone(), Color::PURPLE, true, true, ConcatMethod::StandardDeviation));
            list.push(MenuStatsEntry::new_f32("Mean", mean, Color::WHITE, true, true));

            list.push(MenuStatsEntry::new_f32("Early", early, Color::BLUE, true, true));
            list.push(MenuStatsEntry::new_f32("Late", late, Color::RED, true, true));


            info.push(MenuStatsInfo::new("Hit Variance", GraphType::Scatter, list))
        }
    }

    info
}
