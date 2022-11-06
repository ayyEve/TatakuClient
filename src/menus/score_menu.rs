use crate::prelude::*;
use crate::REPLAY_EXPORTS_DIR;
use crate::gameplay::modes::manager_from_playmode;

use chrono::{ 
    NaiveDateTime,
    DateTime,
    Local,
    Utc
};

const GRAPH_SIZE:Vector2 = Vector2::new(400.0, 200.0);
const GRAPH_PADDING:Vector2 = Vector2::new(10.0,10.0);

const MENU_ITEM_COUNT:usize = 2;

pub struct ScoreMenu {
    score: IngameScore,
    pub replay: Option<Replay>,
    score_mods: String,

    beatmap: Arc<BeatmapMeta>,
    back_button: MenuButton<Font2, Text>,
    replay_button: MenuButton<Font2, Text>,
    graph: Graph<Font2, Text>,

    // cached
    hit_error: HitError,
    hit_counts: Vec<(String, u32, Color)>,


    pub dont_do_menu: bool,
    pub should_close: bool,

    selected_index: usize,
    window_size: Arc<WindowSize>,

    pub score_submit: Option<Arc<ScoreSubmitHelper>>,

    score_submit_response: Option<SubmitResponse>,
}
impl ScoreMenu {
    pub fn new(score:&IngameScore, beatmap: Arc<BeatmapMeta>) -> ScoreMenu {
        let window_size = WindowSize::get();
        let hit_error = score.hit_error();
        let font = get_font();
        let back_button = MenuButton::back_button(window_size.0, font.clone());

        let graph = Graph::new(
            Vector2::new(window_size.x * 2.0/3.0, window_size.y) - (GRAPH_SIZE + GRAPH_PADDING), //window_size() - (GRAPH_SIZE + GRAPH_PADDING),
            GRAPH_SIZE,
            score.hit_timings.iter().map(|e|*e as f32).collect(),
            -50.0,
            50.0,
            font.clone()
        );
        
        // map hit types to a display string
        let mut hit_counts = Vec::new();
        for judge in get_judgments(&score.playmode).variants().iter() {
            let txt = judge.as_str_display();
            if txt.is_empty() {continue}

            let count = score.judgments.get(judge.as_str_internal()).map(|n|*n).unwrap_or_default();

            let mut color = judge.color();
            if color.a == 0.0 { color = Color::BLACK }

            hit_counts.push((txt.to_owned(), count as u32, color));
        }

        // extract mods
        let mut score_mods = ModManager::short_mods_string(score.mods(), false); //(score.mods_string.as_ref()).map(|s| serde_json::from_str::<ModManager>(s).unwrap_or_default().mods_list_string_no_speed()).unwrap_or_default();
        if score_mods.len() > 0 { score_mods = format!("Mods: {score_mods}"); }

        ScoreMenu {
            score: score.clone(),
            score_mods,
            replay: None,
            beatmap,
            hit_error,
            graph,
            replay_button: MenuButton::new(back_button.get_pos() - Vector2::new(0.0, back_button.size().y+5.0), back_button.size(), "Replay", font.clone()),
            back_button,

            dont_do_menu: false,
            should_close: false,

            selected_index: 99,
            hit_counts,
            window_size,
            score_submit: None,
            score_submit_response: None,
        }
    }

    fn close(&mut self, game: &mut Game) {
        if self.dont_do_menu {
            self.should_close = true;
            return;
        }

        let menu = game.menus.get("beatmap").unwrap().clone();
        game.queue_state_change(GameState::InMenu(menu));
    }

    async fn replay(&mut self, game: &mut Game) {
        if let Some(replay) = self.replay.clone() {
            self.do_replay(game, replay).await;
        } else if let Some(replay) = self.score.get_replay().await {
            self.do_replay(game, replay).await;
        } else {
            println!("no replay")
        }
    }

    async fn do_replay(&mut self, game: &mut Game, mut replay: Replay) {

        // game.menus.get("beatmap").unwrap().lock().on_change(false);
        // game.queue_mode_change(GameMode::Replaying(self.beatmap.clone(), replay.clone(), 0));
        match manager_from_playmode(self.score.playmode.clone(), &self.beatmap).await {
            Ok(mut manager) => {
                if replay.score_data.is_none() {
                    replay.score_data = Some(self.score.score.clone());
                }
                manager.set_replay(replay);
                game.queue_state_change(GameState::Ingame(manager));
            },
            Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e).await
        }
    }





}

#[async_trait]
impl AsyncMenu<Game> for ScoreMenu {

    async fn update(&mut self, _game:&mut Game) {
        if self.score_submit_response.is_none() {
            if let Some(t) = &self.score_submit {
                if let Some(r) = t.response.read().await.as_ref() {
                    self.score_submit_response = Some(r.clone());
                }
            }
        }
    }

    async fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font();

        let depth = 0.0;
        
        // draw beatmap title string
        list.push(Box::new(Text::new(
            Color::BLACK,
            depth + 1.0,
            Vector2::new(10.0, 20.0),
            30,
            format!("{} ({}) (x{:.2})", self.beatmap.version_string(), gamemode_display_name(&self.score.playmode), self.score.speed),
            font.clone()
        )));

        let mut current_pos = Vector2::new(25.0, 80.0);
        let size = Vector2::new(0.0, 35.0);

        // draw score info
        list.push(Box::new(Text::new(
            Color::BLACK,
            depth + 1.0,
            current_pos,
            30,
            format!("Score: {}", format_number(self.score.score.score)),
            font.clone()
        )));
        current_pos += size;

        for (str, count, color) in self.hit_counts.iter() {
            list.push(Box::new(Text::new(
                *color,
                depth + 1.0,
                current_pos,
                30,
                format!("{str}: {}", format_number(*count)),
                font.clone()
            )));
            current_pos += size;
        }

        current_pos += size / 2.0;
        for str in [
            format!("Combo: {}x, {:.2}%", format_number(self.score.max_combo), calc_acc(&self.score) * 100.0),
            String::new(),
            format!("Mean: {:.2}ms", self.hit_error.mean),
            format!("Error: {:.2}ms - {:.2}ms avg", self.hit_error.early, self.hit_error.late),
            format!("Deviance: {:.2}ms", self.hit_error.deviance),
            if self.score.speed != 1.0 {format!("Speed: {:.2}x", self.score.speed)} else { String::new() },
            // format!("Expected Performance: {:.2}pr", self.score.score.performance),
            self.score_mods.clone(),
        ] {
            if !str.is_empty() {
                list.push(Box::new(Text::new(
                    Color::BLACK,
                    depth + 1.0,
                    current_pos,
                    30,
                    str,
                    font.clone()
                )));

                current_pos += size;
            } else {
                current_pos += size / 2.0;
            }
        }

        if let Some(sub) = &self.score_submit_response {
            current_pos += size / 2.0;

            match sub {
                SubmitResponse::NotSubmitted(_, str) => {
                    list.push(Box::new(Text::new(
                        Color::BLACK,
                        depth + 1.0,
                        current_pos,
                        30,
                        format!("Score not submitted: {str}"),
                        font.clone()
                    )));
                }

                SubmitResponse::Submitted { score_id:_, placing, performance_rating } => {
                    for str in [
                        format!("Map Ranking: #{}", format_number(*placing)),
                        format!("Performance: {}pr", format_float(performance_rating, 2)),
                    ] {
                        list.push(Box::new(Text::new(
                            Color::BLACK,
                            depth + 1.0,
                            current_pos,
                            30,
                            str,
                            font.clone()
                        )));
                        current_pos += size;
                    }
                }
            }
        }


        // draw buttons
        self.back_button.draw(args, Vector2::zero(), depth, &mut list);
        self.replay_button.draw(args, Vector2::zero(), depth, &mut list);


        // graph
        self.graph.draw(args, Vector2::zero(), depth, &mut list);
        
        // draw background so score info is readable
        list.push(visibility_bg(
            Vector2::one() * 5.0, 
            Vector2::new(self.window_size.x * 2.0/3.0, self.window_size.y - 5.0),
            depth + 10.0
        ));

        list
    }

    async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        if self.replay_button.on_click(pos, button, mods) {
            // self.beatmap.lock().reset();
            self.replay(game).await;
            return;
        }

        if self.back_button.on_click(pos, button, mods) {
            self.close(game)
        }
    }

    async fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        self.replay_button.on_mouse_move(pos);
        self.back_button.on_mouse_move(pos);
        self.graph.on_mouse_move(pos);
    }

    async fn on_key_press(&mut self, key:piston::Key, game: &mut Game, _mods:KeyModifiers) {
        if key == piston::Key::Escape {
            self.close(game)
        }

        if key == piston::Key::F2 {
            if let Some(replay) = &self.replay {
                // save the replay
                match save_replay(replay, &self.score) {
                    Ok(saved_path) => {
                        let saved_path = Path::new(&saved_path);

                        let BeatmapMeta { artist, title, version, .. } = &*self.beatmap;
                        let Score { playmode, username, time, .. } = &self.score.score;
                        let playmode = gamemode_display_name(playmode);

                        let datetime = NaiveDateTime::from_timestamp(*time as i64, 0);
                        let score_time = DateTime::<Utc>::from_utc(datetime, Utc).with_timezone(&Local);
                        let date = score_time.date().format("%d-%m-%Y").to_string();

                        let export_path = format!("{REPLAY_EXPORTS_DIR}/") + &sanitize_filename(format!("{username}[{playmode}] - {artist} - {title} [{version}] ({date}).ttkr"));
                        let export_path = Path::new(&export_path);

                        // ensure export dir exists
                        match std::fs::create_dir_all(&export_path.parent().unwrap()) {
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
        }
    }

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size;
        
    }
}

#[async_trait]
impl ControllerInputMenu<Game> for ScoreMenu {
    async fn controller_down(&mut self, game:&mut Game, controller: &Box<dyn Controller>, button: u8) -> bool {

        let mut changed = false;
        if let Some(ControllerButton::DPad_Down) = controller.map_button(button) {
            self.selected_index += 1;
            if self.selected_index >= MENU_ITEM_COUNT {
                self.selected_index = 0;
            }

            changed = true;
        }

        if let Some(ControllerButton::DPad_Up) = controller.map_button(button) {
            if self.selected_index == 0 {
                self.selected_index = 3;
            } else if self.selected_index >= MENU_ITEM_COUNT { // original value is 99
                self.selected_index = 0;
            } else {
                self.selected_index -= 1;
            }

            changed = true;
        }

        if changed {
            self.replay_button.set_selected(self.selected_index == 0);
            self.back_button.set_selected(self.selected_index == 1);
        }

        if let Some(ControllerButton::A) = controller.map_button(button) {
            match self.selected_index {
                0 => {
                    // replay
                    self.replay(game).await;
                },
                1 => {
                    // back
                    self.close(game);
                },
                _ => {}
            }
        }

        true
    }
}
