use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::{path::Path, fs::read_dir};

use cgmath::Vector2;
use piston::{Key, MouseButton, RenderArgs};

use crate::gameplay::{Beatmap, BeatmapMeta, Score};
use crate::menu::{Menu, ScoreMenu, ScrollableArea, ScrollableItem};
use crate::game::{Game, GameMode, KeyModifiers, Settings, get_font};
use crate::{WINDOW_SIZE, DOWNLOADS_DIR, SONGS_DIR, render::*, databases::get_scores};

// constants
const INFO_BAR_HEIGHT:f64 = 60.0;
const BEATMAPSET_ITEM_SIZE: Vector2<f64> = Vector2::new(550.0, 50.0);
const BEATMAPSET_PAD_RIGHT: f64 = 5.0;

const BEATMAP_ITEM_PADDING: f64 = 5.0;
const BEATMAP_ITEM_SIZE: Vector2<f64> = Vector2::new(450.0, 50.0);

const LEADERBOARD_POS: Vector2<f64> = Vector2::new(10.0, 100.0);
const LEADERBOARD_ITEM_SIZE: Vector2<f64> = Vector2::new(200.0, 50.0);

pub struct BeatmapSelectMenu {
    /// tag of the selected set
    selected: Option<String>,
    selected_beatmap: Option<String>, // hash of selected map, needed for score refresh
    
    current_scores: HashMap<String, Arc<Mutex<Score>>>,
    beatmap_scroll: ScrollableArea,
    leaderboard_scroll: ScrollableArea,

    background_texture: Option<Image>,
    pending_refresh: bool,
}
impl BeatmapSelectMenu {
    pub fn new() -> BeatmapSelectMenu {
        let mut b = BeatmapSelectMenu {
            selected: None,
            selected_beatmap: None,
            pending_refresh: false,
            current_scores: HashMap::new(),
            background_texture: None,

            beatmap_scroll: ScrollableArea::new(Vector2::new(WINDOW_SIZE.x as f64 - (BEATMAPSET_ITEM_SIZE.x+BEATMAPSET_PAD_RIGHT), INFO_BAR_HEIGHT), Vector2::new(BEATMAPSET_ITEM_SIZE.x, WINDOW_SIZE.y as f64 - INFO_BAR_HEIGHT), true),
            leaderboard_scroll: ScrollableArea::new(LEADERBOARD_POS, Vector2::new(BEATMAPSET_ITEM_SIZE.x, WINDOW_SIZE.y as f64 - LEADERBOARD_POS.y), true),
        };
        b.refresh_maps();
        b
    }

    /// returns the selected item
    pub fn get_selected(&self) -> Option<(Arc<Mutex<Beatmap>>, bool)> {
        if self.selected.is_none() {return None}
        let s = self.beatmap_scroll.get_tagged(self.selected.as_ref().unwrap().clone()).first().unwrap().get_value();
        let s = s.downcast_ref::<(Arc<Mutex<Beatmap>>, bool)>();
        if let Some(b) = s {Some(b.clone())} else {None}
    }

    pub fn refresh_maps(&mut self) {
        self.pending_refresh = false;
        self.beatmap_scroll.clear();
        let folders = read_dir(SONGS_DIR).unwrap();

        let mut full_list = Vec::new();

        for f in folders {
            let f = f.unwrap().path();
            let f = f.to_str().unwrap();
            if !Path::new(f).is_dir() {continue;}
            let dir_files = read_dir(f).unwrap();
            let mut list = Vec::new();

            for file in dir_files {
                let file = file.unwrap().path();
                let file = file.to_str().unwrap();

                if file.ends_with(".osu") {
                    let map = Beatmap::load(file.to_owned());
                    if map.lock().unwrap().metadata.mode as u8 > 1 {
                        println!("skipping {}, not a taiko map or convert", map.lock().unwrap().metadata.version_string());
                        continue;
                    }
                    list.push(map);
                }
            }

            if list.len() > 0 {
                full_list.push(Box::new(BeatmapsetItem::new(list)));
            }
        }

        // sort by artist
        full_list.sort_by(|a, b| a.meta.artist.to_lowercase().cmp(&b.meta.artist.to_lowercase()));
        for i in full_list {self.beatmap_scroll.add_item(i)}
    }

    pub fn load_scores(&mut self, map_hash:String) {
        self.leaderboard_scroll.clear();
        self.current_scores.clear();

        // load scores
        let scores = get_scores(map_hash.to_owned());
        let mut scores = scores.lock().unwrap().clone();
        scores.sort_by(|a, b| {
            b.score.cmp(&a.score)
        });

        for s in scores.iter() {
            self.current_scores.insert(s.username.clone(), Arc::new(Mutex::new(s.clone())));
            self.leaderboard_scroll.add_item(Box::new(LeaderboardItem::new(s.to_owned())));
        }
    }
}
impl Menu for BeatmapSelectMenu {
    fn update(&mut self, game:Arc<Mutex<&mut Game>>) {
        if game.lock().unwrap().beatmap_pending_refresh {
            game.lock().unwrap().beatmap_pending_refresh = false;
            self.pending_refresh = true;
            crate::game::extract_all();
        }

        if self.pending_refresh {
            // wait for main to finish extracting everything from downloads
            let list = std::fs::read_dir(DOWNLOADS_DIR).unwrap();
            if list.count() <= 0 {self.refresh_maps();}
        }
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
            Vector2::new(0.0, 0.0),
            Vector2::new(args.window_size[0], INFO_BAR_HEIGHT),
            Some(Border::new(Color::BLACK, 1.2))
        );
        items.push(Box::new(bar_rect));

        // draw selected map info
        if let Some((b, _play)) = self.get_selected() {
            let meta = b.lock().unwrap().metadata.clone();

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
        items.extend(self.beatmap_scroll.draw(args));

        // leaderboard scroll
        items.extend(self.leaderboard_scroll.draw(args));

        // draw background image
        if let Some(img) = self.background_texture.as_ref() {
            items.push(Box::new(img.clone()));
        }

        items
    }

    fn on_volume_change(&mut self) {self.beatmap_scroll.on_volume_change();}
    fn on_change(&mut self) {
        self.beatmap_scroll.refresh_layout();
        if let Some(map_hash) = &self.selected_beatmap.clone() {
            self.load_scores(map_hash.clone());
        }
    }

    fn on_click(&mut self, pos:Vector2<f64>, button:MouseButton, game:Arc<Mutex<&mut Game>>) {

        // check if leaderboard item was clicked
        if let Some(score_tag) = self.leaderboard_scroll.on_click(pos, button, game.clone()) {
            // score display
            let mut game = game.lock().unwrap();
            if let Some(score) = self.current_scores.get(&score_tag) {
                let score = score.lock().unwrap().clone();
                let menu = ScoreMenu::new(score);
                game.queue_mode_change(GameMode::InMenu(Arc::new(Mutex::new(Box::new(menu)))));
            }
            return;
        }

        // check if beatmap item was clicked
        if let Some(clicked_tag) = self.beatmap_scroll.on_click(pos, button, game.clone()) {
            let clicked = self.beatmap_scroll.get_tagged(clicked_tag.clone()).first().unwrap().get_value();
            let (clicked, play) = clicked.downcast_ref::<(Arc<Mutex<Beatmap>>, bool)>().unwrap();

            if *play {
                // reset pending_play var in every item
                for i in self.beatmap_scroll.items.as_mut_slice() {
                    // dirty hack lmao
                    i.set_tag("");
                }
                let mut map = clicked.lock().unwrap();
                map.song.stop();
                map.reset();
                map.start(); // TODO: figure out how to do this when checking mode change
                game.lock().unwrap().start_map(clicked.clone());
                return;
            }

            // get current selected map
            if let Some((b, _play)) = self.get_selected() {
                b.lock().unwrap().song.stop();
            }

            self.selected = Some(clicked_tag.clone());
            self.beatmap_scroll.refresh_layout();

            #[cfg(feature = "textures")] {
                let t = opengl_graphics::Texture::from_path(clicked.lock().unwrap().metadata.image_filename.clone(), &opengl_graphics::TextureSettings::new()).unwrap();
                self.background_texture = Some(Image::new(Vector2::new(0.0,0.0), -99999.0, t));
            }

            let hash = clicked.lock().unwrap().hash.clone();
            self.selected_beatmap = Some(hash.clone());
            self.load_scores(hash.clone());
        } else {
            //TODO: hmm
            self.selected = None;
            self.beatmap_scroll.refresh_layout();
            self.leaderboard_scroll.clear();
        }

    }
    fn on_mouse_move(&mut self, pos:Vector2<f64>, game:Arc<Mutex<&mut Game>>) {
        self.beatmap_scroll.on_mouse_move(pos, game.clone());
        self.leaderboard_scroll.on_mouse_move(pos, game.clone());
    }
    fn on_scroll(&mut self, delta:f64, _game:Arc<Mutex<&mut Game>>) {
        self.beatmap_scroll.on_scroll(delta);
        self.leaderboard_scroll.on_scroll(delta);
    }

    fn on_key_press(&mut self, key:piston::Key, game:Arc<Mutex<&mut Game>>, _mods:KeyModifiers) {
        let mut game = game.lock().unwrap();
        if key == Key::Escape {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_mode_change(GameMode::InMenu(menu));
        }
        if key == Key::F5 {
            self.refresh_maps();
        }
    }

    //TODO: implement search (oh god)
    fn on_text(&mut self, _text:String) {
        
    }
}


struct BeatmapsetItem {
    pos: Vector2<f64>,
    hover: bool,
    selected: bool,
    tag: String,
    pending_play: bool,

    beatmaps: Vec<Arc<Mutex<Beatmap>>>,
    meta: BeatmapMeta,
    selected_item: usize, // index of selected item
    mouse_pos:Vector2<f64>,

    // use this for audio
    first: Arc<Mutex<Beatmap>>
}
impl BeatmapsetItem {
    fn new(beatmaps: Vec<Arc<Mutex<Beatmap>>>) -> BeatmapsetItem {

        // sort beatmaps by sr
        let mut beatmaps = beatmaps.clone();
        beatmaps.sort_by(|a, b| {
            let a = a.lock().unwrap().metadata.sr;
            let b = b.lock().unwrap().metadata.sr;
            a.partial_cmp(&b).unwrap()
        });

        let _first = beatmaps.first().unwrap();
        let first = _first.lock().unwrap();
        let tag = first.metadata.version_string();

        BeatmapsetItem {
            beatmaps: beatmaps.clone(), 
            pos: Vector2::new(0.0, 0.0),
            hover: false,
            selected: false,
            pending_play: false,
            tag,
            meta: first.metadata.clone(),

            selected_item: 0,
            first: _first.clone(),
            mouse_pos: Vector2::new(0.0,0.0)
        }
    }
}
impl ScrollableItem for BeatmapsetItem {
    fn size(&self) -> Vector2<f64> {
        if !self.selected {
            BEATMAPSET_ITEM_SIZE
        } else {
            Vector2::new(BEATMAPSET_ITEM_SIZE.x, (BEATMAPSET_ITEM_SIZE.y + BEATMAP_ITEM_PADDING) * (self.beatmaps.len()+1) as f64)
        }
    }
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, _tag:&str) {self.pending_play = false;} // bit of a jank strat: when this is called, reset the play_pending property
    fn get_pos(&self) -> Vector2<f64> {self.pos}
    fn set_pos(&mut self, pos:Vector2<f64>) {self.pos = pos}
    fn get_value(&self) -> Box<dyn std::any::Any> {
        Box::new((self.beatmaps.get(self.selected_item).unwrap().clone(), self.pending_play))
    }

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2<f64>) -> Vec<Box<dyn Renderable>> {
        let mut items: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");

        let depth = 5.0;

        // draw rectangle
        items.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            depth,
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
            depth - 1.0,
            self.pos+pos_offset + Vector2::new(5.0, 20.0),
            15,
            format!("{} // {} - {}", self.meta.creator, self.meta.artist, self.meta.title),
            font.clone()
        )));

        // if selected, draw map items
        if self.selected {
            let mut pos = self.pos+pos_offset + Vector2::new(BEATMAPSET_ITEM_SIZE.x - BEATMAP_ITEM_SIZE.x, BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING);
            let mut counter = 0;
            
            // try to find the clicked item
            // // we only care about y pos, because we know we were clicked
            let mut index:usize = 999;

            // if x is in correct area to hover over beatmap items
            if self.mouse_pos.x >= self.pos.x + (BEATMAPSET_ITEM_SIZE.x - BEATMAP_ITEM_SIZE.x) {
                let rel_y2 = (self.mouse_pos.y - self.pos.y).abs() - BEATMAPSET_ITEM_SIZE.y;
                index = ((rel_y2 + BEATMAP_ITEM_PADDING/2.0) / (BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING)).floor() as usize;
            }

            for b in self.beatmaps.as_slice() {
                // bounding rect
                items.push(Box::new(Rectangle::new(
                    [0.2, 0.2, 0.2, 1.0].into(),
                    depth,
                    pos,
                    BEATMAP_ITEM_SIZE,
                    if counter == index {
                        Some(Border::new(Color::BLUE, 1.0))
                    } else if counter == self.selected_item {
                        Some(Border::new(Color::RED, 1.0))
                    } else {
                        Some(Border::new(Color::BLACK, 1.0))
                    }
                )));
                // version text
                items.push(Box::new(Text::new(
                    Color::WHITE,
                    depth - 1.0,
                    pos + Vector2::new(5.0, 20.0),
                    12,
                    format!("{}", b.lock().unwrap().metadata.version),
                    font.clone()
                )));

                pos.y += BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING;
                counter += 1;
            }
        }
        items
    }

    fn on_click(&mut self, pos:Vector2<f64>, _button:MouseButton) -> bool {

        if self.selected && self.hover {
            // find the clicked item

            // we only care about y pos, because we know we were clicked
            let rel_y2 = (pos.y - self.pos.y).abs() - BEATMAPSET_ITEM_SIZE.y;
            let index = (((rel_y2 + BEATMAP_ITEM_PADDING/2.0) / (BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING)).floor() as usize).clamp(0, self.beatmaps.len() - 1);

            if self.selected_item == index {
                // queue play map
                self.pending_play = true;
                self.first.lock().unwrap().song.stop();
            } else {
                self.selected_item = index;
            }
            return true;
        }

        // not yet selected
        if !self.selected && self.hover {
            // start song
            self.first.lock().unwrap().song.play();
            self.first.lock().unwrap().song.set_volume(Settings::get().get_music_vol());
        } else { // was selected, not anymore
            // stop music
            self.first.lock().unwrap().song.stop();
        }

        self.selected = self.hover;
        self.hover
    }
    fn on_mouse_move(&mut self, pos:Vector2<f64>) {
        self.mouse_pos = pos;
        self.hover = self.hover(pos)
    }
    fn on_volume_change(&mut self) {
        self.first.lock().unwrap().song.set_volume(Settings::get().get_music_vol());
    }

    fn dispose(&mut self) {
        self.first.lock().unwrap().song.stop();
    }
}


struct LeaderboardItem {
    pos: Vector2<f64>,
    hover: bool,
    tag: String,

    score: Score,
    acc: f64,
}
impl LeaderboardItem {
    pub fn new(score:Score) -> LeaderboardItem {
        let tag = score.username.clone();
        let acc = score.acc() * 100.0;

        LeaderboardItem {
            pos: Vector2::new(0.0,0.0),
            score,
            tag,
            acc,
            hover: false
        }
    }
}
impl ScrollableItem for LeaderboardItem {
    fn size(&self) -> Vector2<f64> {LEADERBOARD_ITEM_SIZE}
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, tag:&str) {self.tag = tag.to_owned()}
    fn get_pos(&self) -> Vector2<f64> {self.pos}
    fn set_pos(&mut self, pos:Vector2<f64>) {self.pos = pos}

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2<f64>) -> Vec<Box<dyn Renderable>> {
        let mut items: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");
        
        let depth = 5.0;

        // bounding rect
        items.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            depth,
            self.pos+pos_offset,
            LEADERBOARD_ITEM_SIZE,
            if self.hover {
                Some(Border::new(Color::RED, 1.0))
            } else {None}
        )));

        // score text
        items.push(Box::new(Text::new(
            Color::WHITE,
            depth - 1.0,
            self.pos+pos_offset + Vector2::new(5.0, 20.0),
            15,
            format!("{}: {}", self.score.username, crate::format(self.score.score)),
            font.clone()
        )));

        // combo text
        items.push(Box::new(Text::new(
            Color::WHITE,
            depth - 1.0,
            self.pos+pos_offset + Vector2::new(5.0, 40.0),
            12,
            format!("{}x, {:.2}%", crate::format(self.score.max_combo), self.acc),
            font.clone()
        )));

        items
    }

    fn on_click(&mut self, _pos:Vector2<f64>, _button:MouseButton) -> bool {self.hover}
    fn on_mouse_move(&mut self, pos:Vector2<f64>) {self.hover = self.hover(pos);}
}
