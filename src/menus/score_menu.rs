use crate::prelude::*;
use crate::{databases, format_number};
use crate::gameplay::modes::manager_from_playmode;

const GRAPH_SIZE:Vector2 = Vector2::new(400.0, 200.0);
const GRAPH_PADDING:Vector2 = Vector2::new(10.0,10.0);

const MENU_ITEM_COUNT:usize = 2;

pub struct ScoreMenu {
    score: Score,
    beatmap: BeatmapMeta,
    back_button: MenuButton<Font2, Text>,
    replay_button: MenuButton<Font2, Text>,
    graph: Graph<Font2, Text>,

    // cached
    hit_error: HitError,

    hit_counts: Vec<(String, u32)>,


    pub dont_do_menu: bool,
    pub should_close: bool,

    selected_index: usize
}
impl ScoreMenu {
    pub fn new(score:&Score, beatmap: BeatmapMeta) -> ScoreMenu {
        let window_size = Settings::window_size();
        let hit_error = score.hit_error();
        let font = get_font();
        let back_button = MenuButton::back_button(window_size, font.clone());

        let graph = Graph::new(
            Vector2::new(window_size.x * 2.0/3.0, window_size.y) - (GRAPH_SIZE + GRAPH_PADDING), //window_size() - (GRAPH_SIZE + GRAPH_PADDING),
            GRAPH_SIZE,
            score.hit_timings.iter().map(|e|*e as f32).collect(),
            -50.0,
            50.0,
            font.clone()
        );
        // let playmode = &score.playmode;

        
        // // map hit types to a display string
        let mut hit_counts = Vec::new();
        // TODO: get the judgment variants somehow, so we can order them correctly
        // for judge in manager.judgment_type.variants().iter() {
        //     let txt = judge.as_str();
        //     if txt.is_empty() {continue}

        //     let count = score.judgments.get(txt).map(|n|*n).unwrap_or_default();
        //     hit_counts.push((txt.to_owned(), count as u32));
        // }

        for (txt, count) in score.judgments.iter() {
            if txt.is_empty() { continue }

            hit_counts.push((txt.clone(), *count as u32));
        }

        // for (hit_type, count) in [
        //     (ScoreHit::Miss, score.xmiss),
        //     (ScoreHit::X50, score.x50),
        //     (ScoreHit::X100, score.x100),
        //     (ScoreHit::Xkatu, score.xkatu),
        //     (ScoreHit::X300, score.x300),
        //     (ScoreHit::Xgeki, score.xgeki),
        // ] {
        //     let txt = get_score_hit_string(playmode, &hit_type);
        //     if txt.is_empty() {continue}

        //     hit_counts.push((txt, count as u32));
        // }

        ScoreMenu {
            score: score.clone(),
            beatmap,
            hit_error,
            graph,
            replay_button: MenuButton::new(back_button.get_pos() - Vector2::new(0.0, back_button.size().y+5.0), back_button.size(), "Replay", font.clone()),
            back_button,

            dont_do_menu: false,
            should_close: false,

            selected_index: 99,
            hit_counts
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
        match databases::get_local_replay_for_score(&self.score) {
            Ok(replay) => {
                // game.menus.get("beatmap").unwrap().lock().on_change(false);
                // game.queue_mode_change(GameMode::Replaying(self.beatmap.clone(), replay.clone(), 0));
                match manager_from_playmode(self.score.playmode.clone(), &self.beatmap).await {
                    Ok(mut manager) => {
                        manager.replaying = true;
                        manager.replay = replay.clone();
                        manager.replay.speed = self.score.speed;
                        game.queue_state_change(GameState::Ingame(manager));
                    },
                    Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e).await
                }
            },
            Err(e) => NotificationManager::add_error_notification("Error loading replay", e).await,
        }
    }
}

#[async_trait]
impl AsyncMenu<Game> for ScoreMenu {
    async fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font();

        let window_size = Settings::window_size();

        let depth = 0.0;
        
        // draw beatmap title string
        list.push(Box::new(Text::new(
            Color::BLACK,
            depth + 1.0,
            Vector2::new(10.0, 20.0),
            30,
            format!("{} ({})", self.beatmap.version_string(), gamemode_display_name(&self.score.playmode)),
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
            format!("Score: {}", format_number(self.score.score)),
            font.clone()
        )));
        current_pos += size;

        for (str, count) in self.hit_counts.iter() {
            list.push(Box::new(Text::new(
                Color::BLACK,
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


        // draw buttons
        self.back_button.draw(args, Vector2::zero(), depth, &mut list);
        self.replay_button.draw(args, Vector2::zero(), depth, &mut list);


        // graph
        self.graph.draw(args, Vector2::zero(), depth, &mut list);
        
        // draw background so score info is readable
        list.push(visibility_bg(
            Vector2::one() * 5.0, 
            Vector2::new(window_size.x * 2.0/3.0, window_size.y - 5.0),
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