use std::collections::HashMap;

use piston::{Key, MouseButton, RenderArgs};
use ayyeve_piston_ui::menu::menu_elements::TextInput;

use crate::render::*;
use taiko_rs_common::types::{Score, PlayMode};
use crate::game::managers::NotificationManager;
use crate::gameplay::{BeatmapMeta, modes::manager_from_playmode};
use crate::{window_size, Vector2, databases::get_scores, sync::*};
use crate::menu::{Menu, ScoreMenu, ScrollableArea, ScrollableItem, MenuButton};
use crate::game::{Game, GameState, KeyModifiers, get_font, Audio, managers::BEATMAP_MANAGER};


// constants
const INFO_BAR_HEIGHT: f64 = 60.0;
const BEATMAPSET_ITEM_SIZE: Vector2 = Vector2::new(550.0, 50.0);
const BEATMAPSET_PAD_RIGHT: f64 = 5.0;

const BEATMAP_ITEM_PADDING: f64 = 5.0;
const BEATMAP_ITEM_SIZE: Vector2 = Vector2::new(450.0, 50.0);

const LEADERBOARD_PADDING: f64 = 100.0;
const LEADERBOARD_POS: Vector2 = Vector2::new(10.0, LEADERBOARD_PADDING);
const LEADERBOARD_ITEM_SIZE: Vector2 = Vector2::new(200.0, 50.0);


pub struct BeatmapSelectMenu {
    mode: PlayMode,
    
    current_scores: HashMap<String, Arc<Mutex<Score>>>,
    beatmap_scroll: ScrollableArea,
    leaderboard_scroll: ScrollableArea,
    back_button: MenuButton,
    // pending_refresh: bool,

    /// is changing, update loop detected that it was changing
    map_changing: (bool, bool, u32),

    // drag: Option<DragData>,
    // mouse_down: bool

    /// internal search box
    search_text: TextInput
}
impl BeatmapSelectMenu {
    pub fn new() -> BeatmapSelectMenu {
        let window_size = window_size();
        BeatmapSelectMenu {
            mode: PlayMode::Standard,

            // mouse_down: false,
            // drag: None,

            // pending_refresh: false,
            map_changing: (false, false, 0),
            current_scores: HashMap::new(),
            back_button: MenuButton::back_button(window_size),

            // beatmap_scroll: ScrollableArea::new(Vector2::new(window_size().x - (BEATMAPSET_ITEM_SIZE.x + BEATMAPSET_PAD_RIGHT), INFO_BAR_HEIGHT), Vector2::new(window_size().x - LEADERBOARD_ITEM_SIZE.x, window_size().y - INFO_BAR_HEIGHT), true),
            beatmap_scroll: ScrollableArea::new(Vector2::new(LEADERBOARD_POS.x + LEADERBOARD_ITEM_SIZE.x, INFO_BAR_HEIGHT), Vector2::new(window_size.x - LEADERBOARD_ITEM_SIZE.x, window_size.y - INFO_BAR_HEIGHT), true),
            leaderboard_scroll: ScrollableArea::new(LEADERBOARD_POS, Vector2::new(LEADERBOARD_ITEM_SIZE.x, window_size.y - (LEADERBOARD_PADDING + INFO_BAR_HEIGHT)), true),
            search_text: TextInput::new(Vector2::new(window_size.x - (window_size.x / 4.0), 0.0), Vector2::new(window_size.x / 4.0, INFO_BAR_HEIGHT), "Search", "")
        }
    }

    pub fn refresh_maps(&mut self) {
        let filter_text = self.search_text.get_text().to_ascii_lowercase();
        self.beatmap_scroll.clear();

        // used to select the current map in the list
        let current_hash = if let Some(map) = &BEATMAP_MANAGER.lock().current_beatmap {map.beatmap_hash.clone()} else {String::new()};

        let sets = BEATMAP_MANAGER.lock().all_by_sets();
        let mut full_list = Vec::new();

        for mut maps in sets {
            if !filter_text.is_empty() {
                maps.retain(|bm|bm.filter(&filter_text));
                if maps.len() == 0 {continue}
            }

            let mut i = BeatmapsetItem::new(maps);
            i.check_selected(&current_hash);
            full_list.push(Box::new(i));
        }

        // sort by artist
        full_list.sort_by(|a, b| a.beatmaps[0].artist.to_lowercase().cmp(&b.beatmaps[0].artist.to_lowercase()));
        for i in full_list {self.beatmap_scroll.add_item(i)}

        self.beatmap_scroll.scroll_to_selection();
    }

    pub fn load_scores(&mut self) {
        // if nothing is selected, leave
        if let Some(map) = &BEATMAP_MANAGER.lock().current_beatmap {

            // clear lists
            self.leaderboard_scroll.clear();
            self.current_scores.clear();

            // load scores
            let mut scores = get_scores(&map.beatmap_hash);
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
        let manager = manager_from_playmode(self.mode, map);
        game.queue_state_change(GameState::Ingame(Arc::new(Mutex::new(manager))));
    }
}
impl Menu<Game> for BeatmapSelectMenu {
    fn update(&mut self, game:&mut Game) {
        self.search_text.set_selected(true); // always have it selected
        self.search_text.update();

        let maps = BEATMAP_MANAGER.lock().get_new_maps();
        if maps.len() > 0 {
            BEATMAP_MANAGER.lock().set_current_beatmap(game, &maps[maps.len() - 1], false, true);
            self.refresh_maps();
        }
    
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
    }

    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut items: Vec<Box<dyn Renderable>> = Vec::new();
        // let mut counter: usize = 0;
        let depth: f64 = 5.0;
        let font = get_font("main");

        // draw a bar on the top for the info
        let bar_rect = Rectangle::new(
            Color::WHITE,
            depth - 1.0,
            Vector2::zero(),
            Vector2::new(args.window_size[0], INFO_BAR_HEIGHT),
            Some(Border::new(Color::BLACK, 1.2))
        );
        items.push(Box::new(bar_rect));

        // draw selected map info
        if let Some(meta) = &BEATMAP_MANAGER.lock().current_beatmap {
            // draw map name top-most left-most
            items.push(Box::new(Text::new(
                Color::BLACK,
                -10.0,
                Vector2::new(0.0, 30.0),
                25,
                meta.version_string(),
                font.clone()
            )));

            // diff string, under map string
            items.push(Box::new(Text::new(
                Color::BLACK,
                -10.0,
                Vector2::new(0.0, 55.0),
                15,
                meta.diff_string(),
                font.clone()
            )));
        }

        // beatmap scroll
        items.extend(self.beatmap_scroll.draw(args, Vector2::zero(), 0.0));

        // leaderboard scroll
        items.extend(self.leaderboard_scroll.draw(args, Vector2::zero(), 0.0));

        // back button
        items.extend(self.back_button.draw(args, Vector2::zero(), 0.0));

        // filter text
        items.extend(self.search_text.draw(args, Vector2::zero(), 0.0));

        items
    }

    fn on_change(&mut self, into:bool) {
        if !into {return}

        // load maps
        self.refresh_maps();
        self.beatmap_scroll.refresh_layout();

        if BEATMAP_MANAGER.lock().current_beatmap.is_some() {
            self.load_scores();
        }
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, mods: ayyeve_piston_ui::menu::KeyModifiers, game:&mut Game) {
        if self.back_button.on_click(pos, button, mods) {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }

        // check if leaderboard item was clicked
        if let Some(score_tag) = self.leaderboard_scroll.on_click_tagged(pos, button, mods) {
            // score display
            if let Some(score) = self.current_scores.get(&score_tag) {
                let score = score.lock().clone();

                if let Some(selected) = &BEATMAP_MANAGER.lock().current_beatmap {
                    let menu = ScoreMenu::new(&score, selected.clone());
                    game.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(menu))));
                }
            }
            return;
        }

        // check if beatmap item was clicked
        if let Some(clicked_hash) = self.beatmap_scroll.on_click_tagged(pos, button, mods) {
            let mut lock = BEATMAP_MANAGER.lock();
            println!("clicked: {}", clicked_hash);

            // compare last clicked map hash with the new hash.
            // if the hashes are the same, the same map was clicked twice in a row.
            // play it
            if let Some(current) = &lock.current_beatmap {
                if current.beatmap_hash == clicked_hash {
                    self.play_map(game, current);
                    self.map_changing = (true, false, 0);
                    return;
                }
            }

            // set the current map to the clicked
            self.map_changing = (true, false, 0);
            let clicked = lock.get_by_hash(&clicked_hash).unwrap();
            lock.set_current_beatmap(game, &clicked, true, true);
            drop(lock);

            self.beatmap_scroll.refresh_layout();
            self.load_scores();
            return;
        }
        
        // else {
        //     //TODO: hmm
        //     self.selected = None;
        //     self.beatmap_scroll.refresh_layout();
        //     self.leaderboard_scroll.clear();
        // }

        // self.beatmap_scroll.refresh_layout();
    }
    fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        self.back_button.on_mouse_move(pos);
        self.beatmap_scroll.on_mouse_move(pos);
        self.leaderboard_scroll.on_mouse_move(pos);
    }
    fn on_scroll(&mut self, delta:f64, _game:&mut Game) {
        self.beatmap_scroll.on_scroll(delta);
        self.leaderboard_scroll.on_scroll(delta);
    }

    fn on_key_press(&mut self, key:piston::Key, game:&mut Game, mods:KeyModifiers) {
        if key == Key::Escape {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }
        if key == Key::F5 {
            self.refresh_maps();
            return;
        }

        if mods.alt {
            let new_mode = match key {
                piston::Key::D1 => Some(PlayMode::Standard),
                piston::Key::D2 => Some(PlayMode::Taiko),
                piston::Key::D3 => Some(PlayMode::Catch),
                piston::Key::D4 => Some(PlayMode::Mania),
                _ => None
            };

            if let Some(new_mode) = new_mode {
                self.mode = new_mode;
                NotificationManager::add_text_notification(&format!("Mode changed to {:?}", new_mode), 1000.0, Color::BLUE);
            }
        }

        // only refresh if the text changed
        let old_text = self.search_text.get_text();
        self.search_text.on_key_press(key, mods);
        if self.search_text.get_text() != old_text {
            self.refresh_maps();
        }
    }

    //TODO: implement search (oh god)
    fn on_text(&mut self, text:String) {
        self.search_text.on_text(text);
        self.refresh_maps();
    }
}


struct BeatmapsetItem {
    pos: Vector2,
    hover: bool,
    selected: bool,
    
    beatmaps: Vec<BeatmapMeta>,
    selected_index: usize,
    mouse_pos: Vector2
}
impl BeatmapsetItem {
    fn new(beatmaps: Vec<BeatmapMeta>) -> BeatmapsetItem {
        // sort beatmaps by sr
        // let mut beatmaps = beatmaps.clone();
        // todo once mode diff calcs get re-implemented
        // beatmaps.sort_by(|a, b| {
        //     let a = a.lock().metadata.sr;
        //     let b = b.lock().metadata.sr;
        //     a.partial_cmp(&b).unwrap()
        // });

        let x = window_size().x - (BEATMAPSET_ITEM_SIZE.x + BEATMAPSET_PAD_RIGHT + LEADERBOARD_POS.x + LEADERBOARD_ITEM_SIZE.x);

        BeatmapsetItem {
            beatmaps: beatmaps.clone(), 
            pos: Vector2::new(x, 0.0),
            hover: false,
            selected: false,
            // pending_play: false,
            // tag,

            selected_index: 0,
            mouse_pos: Vector2::zero()
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
impl ScrollableItem for BeatmapsetItem {
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
    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.beatmaps[self.selected_index].clone())}

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, selected:bool) {self.selected = selected}

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64) -> Vec<Box<dyn Renderable>> {
        let mut items: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");
        let meta = &self.beatmaps[0];

        // draw rectangle
        items.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            parent_depth + 5.0,
            self.pos+pos_offset,
            BEATMAPSET_ITEM_SIZE,
            if self.hover {
                Some(Border::new(Color::RED, 1.0))
            } else if self.selected {
                Some(Border::new(Color::BLUE, 1.0))
            } else {
                None
            }
        )));

        // line 1
        items.push(Box::new(Text::new(
            Color::WHITE,
            parent_depth + 4.0,
            self.pos+pos_offset + Vector2::new(5.0, 20.0),
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

            for i in 0..self.beatmaps.len() {
                let meta = &self.beatmaps[i];

                // bounding rect
                items.push(Box::new(Rectangle::new(
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
                )));
                // version text
                items.push(Box::new(Text::new(
                    Color::WHITE,
                    parent_depth + 4.0,
                    pos + Vector2::new(5.0, 20.0),
                    12,
                    format!("({:?}) - {}", meta.mode, meta.version),
                    font.clone()
                )));

                pos.y += BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING;
            }
        }
        
        items
    }

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
}



struct LeaderboardItem {
    pos: Vector2,
    hover: bool,
    selected: bool,
    tag: String,

    score: Score,
    acc: f64,
}
impl LeaderboardItem {
    pub fn new(score:Score) -> LeaderboardItem {
        let tag = score.username.clone();
        let acc = score.acc() * 100.0;

        LeaderboardItem {
            pos: Vector2::zero(),
            score,
            tag,
            acc,
            hover: false,
            selected: false
        }
    }
}
impl ScrollableItem for LeaderboardItem {
    fn size(&self) -> Vector2 {LEADERBOARD_ITEM_SIZE}
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, tag:&str) {self.tag = tag.to_owned()}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, selected:bool) {self.selected = selected}

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64) -> Vec<Box<dyn Renderable>> {
        let mut items: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");

        // bounding rect
        items.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            parent_depth + 5.0,
            self.pos+pos_offset,
            LEADERBOARD_ITEM_SIZE,
            if self.hover {Some(Border::new(Color::RED, 1.0))} else {None}
        )));

        // score text
        items.push(Box::new(Text::new(
            Color::WHITE,
            parent_depth + 4.0,
            self.pos+pos_offset + Vector2::new(5.0, 20.0),
            15,
            format!("{}: {}", self.score.username, crate::format(self.score.score)),
            font.clone()
        )));

        // combo text
        items.push(Box::new(Text::new(
            Color::WHITE,
            parent_depth + 4.0,
            self.pos+pos_offset+Vector2::new(5.0, 40.0),
            12,
            format!("{}x, {:.2}%", crate::format(self.score.max_combo), self.acc),
            font.clone()
        )));

        items
    }

    // fn on_click(&mut self, _pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> bool {self.hover}
}


// struct DragData {
//     start_pos: f64,
//     current_pos: f64,
//     start_time: Instant
// }