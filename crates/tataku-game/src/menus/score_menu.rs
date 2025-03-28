use chrono::Local;
use crate::prelude::*;
use crate::prelude::ui::*;
use crate::REPLAY_EXPORTS_DIR;

pub struct ScoreMenu {
    actions: ActionQueue,
    infos: GamemodeInfos,

    score: IngameScore,
    beatmap: Arc<BeatmapMeta>,

    menu_type: Box<ScoreMenuType>,

    /// can the user retry?
    allow_retry: bool,

    // cached
    score_mods: String,
    hit_error: HitError,
    hit_counts: Vec<(String, u32, Color)>,
    stats: Vec<StatsInfo>,

    /// what stat is selected?
    selected_stat: usize,

    pub score_submit: Option<String>,

    node: Box<dyn Widget>,
    node_id: NodeId,
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
            menu_type: Box::new(ScoreMenuType::Normal),
                
            score: score.clone(),
            score_mods,
            beatmap,
            hit_error,
            allow_retry,

            hit_counts,
            score_submit: None,

            selected_stat: 0,
            stats,

            node: EmptyWidget::new_boxed(),
            node_id: EMPTY_NODE
        }
    }


    fn build_view(&mut self) -> Box<dyn Widget> {
        // score info
        let beatmap_label = format!("{} ({}) (x{:.2})", self.beatmap.version_string(), self.infos.get_info(&self.score.playmode).unwrap().display_name, self.score.speed);
        col!(
            // beatmap label
            TextWidget::new(beatmap_label).width(FILL).boxed(),
            
            // data
            row!(
                // score info
                row!(
                    // score values
                    col!(
                        self.score_lines(),
                        width = Dimension::Percent(0.5),
                        height = FILL
                    ),

                    // stats
                    self.get_stats_view();

                    width = Dimension::Percent(0.5),
                    height = FILL
                ),

                // multi scores
                if let ScoreMenuType::Multiplayer {lobby_items, ..} = &*self.menu_type {
                    col!(
                        lobby_items.iter().map(|l| l.view()).collect::<Vec<_>>(),
                        width = FILL, // FillPortion(1)
                        height = FILL,
                        horizontal_align = AlignContent::End
                    )
                } else {
                    EmptyWidget::new_boxed()
                };

                width = FILL,
                height = FILL
            ),

            // buttons
            col!(
                self.get_buttons(),
                width = FILL,
                height = SHRINK
            )

            // // key event helper
            // self.key_handler.handler();
            ;

            width = FILL,
            height = FILL
        )
    }

    async fn close(&mut self) {
        self.actions.push(MenuAction::PreviousMenu(self.name()));

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
        if self.score.replay.is_none() { 
            self.actions.push(
                Notification::default()
                .text("There is no replay to save!")
                .duration(5_000.0)
                .color(Color::RED)
            );
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
                            self.actions.push(Notification::new_error("Error exporting replay", e));
                        } else {
                            self.actions.push(
                                Notification::default()
                                .text("Replay exported!")
                                .duration(5000.0)
                                .color(Color::BLUE)
                            );
                        }
                    }
                    Err(e) => self.actions.push(Notification::new_error("Error creating exports directory", e)),
                }
            }
            Err(e) => self.actions.push(Notification::new_error("Error saving replay", e)),
        };
    }



    fn score_lines(&self) -> Vec<Box<dyn Widget>> {
        let mut lines = Vec::with_capacity(20);
        let font_size = 30.0;

        macro_rules! add {
            ($s: expr, $color: expr) => {
                lines.push(
                    TextWidget::new($s)
                    .text_color($color)
                    .font_size(font_size)
                    .width(FILL)
                    .boxed()
                )
            };

            ($s: expr) => {
                lines.push(Space::new(FILL, Dimension::Length($s)).boxed());
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

        if let Some(path) = &self.score_submit {
            lines.push(
                ScoreSubmitWidget::default()
                .score_submit_path(path.clone())
                .boxed()
            )
        }
        // if let Some(sub) = &self.score_submit_response {
        //     add!(font_size / 2.0);

        //     match sub {
        //         SubmitResponse::NotSubmitted(_, str) => {
        //             add!(format!("Score not submitted: {str}"), Color::BLACK);
        //         }

        //         SubmitResponse::Submitted { score_id:_, placing, performance_rating } => {
        //             for str in [
        //                 format!("Map Ranking: #{}", format_number(*placing)),
        //                 format!("Performance: {}pr", format_float(*performance_rating, 2)),
        //             ] {
        //                 add!(str, Color::BLACK);
        //                 add!(font_size);
        //             }
        //         }
        //     }
        // }

        lines
    }

    fn get_stats_view(&self) -> Box<dyn Widget> {
        EmptyWidget::new_boxed()

        // TODO!!!!
        // // draw stats graphs
        // if let Some(stat) = self.stats.get(self.selected_stat) {
        //     // const PAD:f32 = 20.0;
        //     // let pos = Vector2::new(self.window_size.x / 2.0, TITLE_STRING_Y + TITLE_STRING_FONT_SIZE + PAD);
        //     // let size = Vector2::new(self.window_size.x * 2.0/3.0 - pos.x, self.window_size.y - (pos.y + PAD * 2.0));

        //     // let bounds = Bounds::new(pos, size);
        //     // stat.draw(&bounds, list)
        //     stat.view()
        // } else {
        //     Box::new(Column::new()
        //         .width(Dimension::Percent(1.0))
        //         .height(Dimension::Percent(1.0))
        //     )
        // }
    }

    fn get_buttons(&self) -> Vec<Box<dyn Widget>> {
        let mut buttons = Vec::with_capacity(2);
        
        // retry button
        if self.allow_retry {
            buttons.push(
                Button::new(TextWidget::new("Retry").boxed())
                    .on_press(Message::click(MessageOwner::Menu, "retry"))
                    .boxed()
            );
        }

        // replay button
        if !self.menu_type.is_lobby() {
            buttons.push(
                Button::new(TextWidget::new("Replay").boxed())
                    .on_press(Message::click(MessageOwner::Menu, "replay"))
                    .boxed()
            );
        }

        buttons.push(
            Button::new(TextWidget::new("Back").boxed())
                .on_press(Message::new(MessageOwner::Menu, "back", MessageValue::Click))
                .boxed()
        );

        buttons
    }
}

#[async_trait]
impl Widget for ScoreMenu {
    fn name(&self) -> Cow<'static, str> { "score_menu".into() }
    fn node_id(&self) -> NodeId { self.node_id }


    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.node = self.build_view();

        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            Style::DEFAULT, 
            &[child]
        )?;

        Ok(self.node_id)
    }

    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>, 
        actions: &mut ActionQueue
    ) {
        // FIXME: !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        // if self.score_submit_response.is_none() {
        //     if let Some(t) = &self.score_submit {
        //         if let Some(r) = t.response.read().await.as_ref() {
        //             self.score_submit_response = Some(r.clone());
        //         }
        //     }
        // }

        // update lobby scores
        self.update_lobby(shell.values);
        
        // while let Some(event) = self.key_handler.check_events() {
        //     match event {
        //         KeyEvent::Press(ScoreMenuKeys::Back) => self.close().await,
        //         KeyEvent::Press(ScoreMenuKeys::SaveReplay) => self.save_replay().await,

        //         KeyEvent::Press(ScoreMenuKeys::PrevStat) if !self.stats.is_empty() => self.selected_stat = self.selected_stat.wrapping_sub_1(self.stats.len()),
        //         KeyEvent::Press(ScoreMenuKeys::NextStat) if !self.stats.is_empty() => self.selected_stat = self.selected_stat.wrapping_add_1(self.stats.len()),
        //         _ => {}
        //     }
        // }

        self.node.update(shell, actions);
        actions.extend(self.actions.take());
    }

    
    async fn handle_message(
        &mut self, 
        message: &Message, 
        values: &mut dyn Reflect,
        actions: &mut ActionQueue
    ) {
        self.node.handle_message(message, values, actions).await;

        let Some(tag) = message.tag.as_string() else { return };
        match &**tag {
            "retry" => self.retry().await,
            "replay" => self.replay(&values.reflect_get::<Settings>("settings").unwrap()).await,
            "back" => self.close().await,
            "score" => if let MessageValue::Number(num) = message.value {
                if let ScoreMenuType::Multiplayer { lobby_items, .. } = &*self.menu_type {
                    if let Some(score) = lobby_items.get(num) {
                        self.change_score(score.score.clone()).await;
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        shell: &mut DrawShell<'_>,
    ) {
        self.node.draw(shell)
    }


    async fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        values: &mut dyn Reflect
    ) {
        self.node.handle_event(event, event_value, values).await
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<'_>
    ) {
        self.node.input(event, shell);
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
pub fn default_stats_from_groups(data: &HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<StatsInfo> { 
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

            list.push(StatsEntry::new_list("Variance", variance_values.clone(), Color::PURPLE, true, true, ConcatMethod::StandardDeviation));
            list.push(StatsEntry::new_f32("Mean", mean, Color::WHITE, true, true));

            list.push(StatsEntry::new_f32("Early", early, Color::BLUE, true, true));
            list.push(StatsEntry::new_f32("Late", late, Color::RED, true, true));


            info.push(StatsInfo::new("Hit Variance", GraphType::Scatter, list))
        }
    }

    info
}



#[derive(Clone)]
pub struct LeaderboardComponent {
    pub num: usize,
    pub score: IngameScore,
    score_mods: String,
    acc: f32,
}
impl LeaderboardComponent {
    pub fn new(
        num: usize, 
        score: IngameScore,
        infos: &GamemodeInfos,
    ) -> Self {

        let info = infos.get_info(&score.playmode).unwrap();
        let score_mods = ModManager::short_mods_string(
            &score.mods, 
            false, 
            info
        );
        let acc = info.calc_acc(&score) * 100.0;


        Self {
            num,
            score,
            score_mods,
            acc
        }
    }
    pub fn view(&self) -> Box<dyn Widget> {
        use crate::prelude::ui::*;
        
        let score_mods = &self.score_mods;
        let acc = self.acc;

        let now = chrono::Utc::now().timestamp() as u64;
        let time_diff = now as i64 - self.score.time as i64;
        let time_diff_str = if time_diff < 60 * 5 {
            format!(" | {time_diff}s")
        } else {
            String::new()
        };

        // TODO: cache this ??
        Button::new(col!(
            TextWidget::new(format!("{}: {}", self.score.username, format_number(self.score.score.score)))
                .width(FILL)
                .font_size(16.0)
                .boxed(),
            TextWidget::new(format!("{}x, {acc:.2}%, {score_mods}{time_diff_str}", format_number(self.score.max_combo)))
                .width(FILL)
                .font_size(16.0)
                .boxed();
        ))
        .width(FILL)
        .on_press(Message::new(MessageOwner::Menu, "score", MessageValue::Number(self.num)))
        .boxed()
    }
}


#[derive(Default)]
#[derive(ChainableInitializer)]
struct ScoreSubmitWidget {
    #[chain] score_submit_path: String,
    style: Style,
    text_style: TextStyle,
    data: ScoreSubmitResponse,

    node_id: NodeId,
}
impl Widget for ScoreSubmitWidget {
    fn name(&self) -> Cow<'static, str> { "score_submit_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf(self.style.clone())?;
        Ok(self.node_id)
    }

    fn update(
        &mut self,
        shell: &mut UpdateShell<'_>,
        _actions: &mut ActionQueue,
    ) {
        let Ok(data) = shell.values.reflect_get::<ScoreSubmitResponse>(&self.score_submit_path) else { return };
        if data.completed {
            self.data = data.cloned();
        }
    }

    fn draw(&self, shell: &mut DrawShell<'_>) {
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id) else { return };
        
        let text = if self.data.completed {
            let text = format!("Placing: {}\nPerformance: {:.2}pr", self.data.placing, self.data.performance_rating);
            self.text_style.create_text(text, bounds)
        } else {
            self.text_style.create_text("Score uploading...".to_owned(), bounds)
        };

        shell.list.push(text);
    }

}