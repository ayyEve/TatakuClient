use crate::prelude::*;


const BEATMAPSET_ITEM_SIZE:Vector2 = Vector2::new(800.0, 50.0);
const BEATMAPSET_PAD_RIGHT:f64 = 5.0;
const BEATMAP_ITEM_PADDING:f64 = 5.0;
const BEATMAP_ITEM_SIZE:Vector2 = Vector2::new(BEATMAPSET_ITEM_SIZE.x * 0.8, 50.0);


pub struct BeatmapsetItem {
    pos: Vector2,
    hover: bool,
    selected: bool,
    
    pub beatmaps: Vec<BeatmapMeta>,
    selected_index: usize,
    mouse_pos: Vector2,
    playmode: String
}
impl BeatmapsetItem {
    pub fn new(mut beatmaps: Vec<BeatmapMeta>, playmode: PlayMode) -> BeatmapsetItem {
        // ensure diff is calced for all maps
        let maps_clone = beatmaps.clone();
        let playmode2 = playmode.clone();
        // tokio::spawn(async move {
        let mods = ModManager::get();
        maps_clone.iter().for_each(|b| {
            if let None = get_diff(&b.beatmap_hash, &playmode2, &mods) {
                let diff = calc_diff(b, playmode2.clone(), &mods).unwrap_or_default();
                insert_diff(&b.beatmap_hash, &playmode2, &mods, diff);
            }
        });
        // });
        
        beatmaps.sort_by(|a, b| a.diff.partial_cmp(&b.diff).unwrap());

        let x = Settings::window_size().x - (BEATMAPSET_ITEM_SIZE.x + BEATMAPSET_PAD_RIGHT + LEADERBOARD_POS.x + LEADERBOARD_ITEM_SIZE.x);

        BeatmapsetItem {
            beatmaps: beatmaps.clone(), 
            pos: Vector2::new(x, 0.0),
            hover: false,
            selected: false,
            // pending_play: false,
            // tag,

            selected_index: 0,
            mouse_pos: Vector2::zero(),
            playmode
        }
    }

    /// set the currently selected map
    pub fn check_selected(&mut self, current_hash: &String) -> bool {
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
impl ScrollableItemGettersSetters for BeatmapsetItem {
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

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, selected:bool) {self.selected = selected}
}
impl ScrollableItem for BeatmapsetItem {
    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.beatmaps[self.selected_index].clone())}

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

    fn on_key_press(&mut self, key:Key, _mods:KeyModifiers) -> bool {
        // press this key if you want to recalculate things
        if key == Key::Calculator {
            let previous_selected = self.beatmaps[self.selected_index].beatmap_hash.clone();

            // get the diff values from the beatmap manager
            for i in self.beatmaps.iter_mut() {
                i.diff = BEATMAP_MANAGER.read().get_by_hash(&i.beatmap_hash).unwrap().diff;
            }
            self.beatmaps.sort_by(|a, b| a.diff.partial_cmp(&b.diff).unwrap());

            // reselect the proper index
            for (i, map) in self.beatmaps.iter().enumerate() {
                if map.beatmap_hash == previous_selected {
                    self.selected_index = i;
                    break
                }
            }

            return false;
        }

        if !self.selected {return false}

        if key == Key::Down {
            self.selected_index += 1;
            if self.selected_index >= self.beatmaps.len() {
                self.selected_index = 0;
            }
            
            return true;
        }

        if key == Key::Up {
            if self.selected_index == 0 {
                self.selected_index = self.beatmaps.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            
            return true;
        }

        false
    }

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        let font = get_font();
        let meta = &self.beatmaps[0];

        // draw rectangle
        list.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            parent_depth + 5.0,
            self.pos + pos_offset,
            BEATMAPSET_ITEM_SIZE,
            if self.hover {
                Some(Border::new(Color::RED, 1.0))
            } else if self.selected {
                Some(Border::new(Color::BLUE, 1.0))
            } else {
                None
            }
        ).shape(Shape::Round(5.0, 10))));

        // line 1
        let title_line = Text::new(
            Color::WHITE,
            parent_depth + 4.0,
            self.pos + pos_offset + Vector2::new(5.0, 5.0),
            15,
            format!("{} // {} - {}", meta.creator, meta.artist, meta.title),
            font.clone()
        );

        // let mut colors = Vec::new();
        // // creator color
        // colors.extend(meta.creator.chars().map(|_|Color::RED));
        // // spacer
        // for _ in 0..4 {colors.push(Color::WHITE)}
        // // artist
        // colors.extend(meta.artist.chars().map(|_|Color::WHITE));
        // // spacer
        // for _ in 0..3 {colors.push(Color::WHITE)}
        // // title
        // colors.extend(meta.title.chars().map(|_|Color::WHITE));

        // title_line.set_text_colors(colors);

        list.push(Box::new(title_line));


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

            if self.mouse_pos.y < self.pos.y {
                index = 999;
            }

            for i in 0..self.beatmaps.len() {
                let meta = &mut self.beatmaps[i];

                // bounding rect
                list.push(Box::new(Rectangle::new(
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
                ).shape(Shape::Round(5.0, 10))));

                // version text
                list.push(Box::new(Text::new(
                    Color::WHITE,
                    parent_depth + 4.0,
                    pos + Vector2::new(5.0, 5.0),
                    12,
                    format!("{} - {}", gamemode_display_name(&meta.mode), meta.version),
                    font.clone()
                )));

                // diff text
                list.push(Box::new(Text::new(
                    Color::WHITE,
                    parent_depth + 4.0,
                    pos + Vector2::new(5.0, 5.0 + 20.0),
                    12,
                    meta.diff_string(self.playmode.clone(), &ModManager::get()),
                    font.clone()
                )));

                pos.y += BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING;
            }
        }
    }

    fn on_text(&mut self, playmode:String) {
        self.playmode = playmode;
    }
}
