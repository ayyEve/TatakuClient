use crate::prelude::*;


pub const BEATMAPSET_ITEM_SIZE:Vector2 = Vector2::new(700.0, 50.0);
// pub const BEATMAPSET_PAD_RIGHT:f64 = 5.0;
const BEATMAP_ITEM_Y_PADDING:f64 = 5.0;
const BEATMAP_ITEM_SIZE:Vector2 = Vector2::new(BEATMAPSET_ITEM_SIZE.x * 0.8, 50.0);


pub struct BeatmapsetItem {
    pos: Vector2,
    hover: bool,
    selected: bool,
    scale: Vector2,
    
    pub beatmaps: Vec<BeatmapMetaWithDiff>,
    selected_index: usize,
    mouse_pos: Vector2,
    // playmode: String,
    mods: ModManagerHelper,
    playmode: CurrentPlaymodeHelper,


    display_text: String,
    double_clicked: bool,

    button_image: Option<SkinnedButton>
}
impl BeatmapsetItem {
    pub async fn new(beatmaps: Vec<BeatmapMetaWithDiff>, display_text: String) -> BeatmapsetItem {
        BeatmapsetItem {
            beatmaps, 
            pos: Vector2::zero(),
            hover: false,
            selected: false,
            display_text,
            scale: Vector2::one(),

            selected_index: 0,
            mouse_pos: Vector2::zero(),
            mods: ModManagerHelper::new(),
            playmode: CurrentPlaymodeHelper::new(),
            double_clicked: false,
            button_image: SkinnedButton::new(Vector2::zero(), BEATMAPSET_ITEM_SIZE, 5.0).await,
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

    fn index_at(&self, pos: Vector2) -> usize {
        let index = self.beatmaps.len() + 20;
        if pos.y < self.pos.y { return index }

        let set_item_size = BEATMAPSET_ITEM_SIZE * self.scale;
        let item_size = BEATMAP_ITEM_SIZE * self.scale;
        let padding = BEATMAP_ITEM_Y_PADDING * self.scale.y;

        if pos.x < self.pos.x + (set_item_size.x - item_size.x) { return index }

        let rel_y = (pos.y - self.pos.y).abs() - set_item_size.y;
        (((rel_y + padding/2.0) / (item_size.y + padding)).floor() as usize).clamp(0, self.beatmaps.len() - 1)
    }

}
impl ScrollableItemGettersSetters for BeatmapsetItem {
    fn size(&self) -> Vector2 {
        if !self.selected {
            BEATMAPSET_ITEM_SIZE * self.scale
        } else {
            Vector2::new(BEATMAPSET_ITEM_SIZE.x, (BEATMAPSET_ITEM_SIZE.y + BEATMAP_ITEM_Y_PADDING) * (self.beatmaps.len()+1) as f64) * self.scale
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
    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.double_clicked)}
    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.scale = scale;
    }

    fn on_click(&mut self, pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> bool {
        if self.selected && self.hover {
            let last_index = self.selected_index;
            // find the clicked item
            // we only care about y pos, because we know we were clicked
            // let rel_y2 = (pos.y - self.pos.y).abs() - BEATMAPSET_ITEM_SIZE.y;
            // let index = (((rel_y2 + BEATMAP_ITEM_Y_PADDING/2.0) / (BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_Y_PADDING)).floor() as usize).clamp(0, self.beatmaps.len() - 1);
            self.selected_index = self.index_at(pos);
            self.double_clicked = self.selected_index == last_index;

            return true;
        }

        self.selected = self.hover;
        self.hover
    }
    fn on_mouse_move(&mut self, pos:Vector2) {
        self.mouse_pos = pos;
        self.check_hover(pos);
    }

    fn on_key_press(&mut self, key:Key, mods:KeyModifiers) -> bool {
        if !self.selected {return false}

        if key == Key::Down && !mods.alt  {
            self.selected_index += 1;
            if self.selected_index >= self.beatmaps.len() {
                self.selected_index = 0;
            }
            
            return true;
        }

        if key == Key::Up && !mods.alt {
            if self.selected_index == 0 {
                self.selected_index = self.beatmaps.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            
            return true;
        }

        false
    }
    
    fn update(&mut self) {
        let mut needs_sort = false;
        for b in self.beatmaps.iter_mut() {
            needs_sort |= b.sort_pending;
            b.sort_pending = false;
        }

        if needs_sort {
            let previous_selected = self.beatmaps[self.selected_index].beatmap_hash.clone();
            self.beatmaps.sort_by(|a, b| a.diff.partial_cmp(&b.diff).unwrap());
    
            // reselect the proper index
            for (i, map) in self.beatmaps.iter().enumerate() {
                if map.beatmap_hash == previous_selected {
                    self.selected_index = i;
                    break
                }
            }
        }

    }

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        let font = get_font();

        // draw rectangle
        if let Some(mut button_image) = self.button_image.clone() {
            button_image.pos = self.pos;
            // button_image
            button_image.draw(_args, pos_offset, list);
        } else {
            list.push(Box::new(Rectangle::new(
                [0.2, 0.2, 0.2, 1.0].into(),
                parent_depth + 5.0,
                self.pos + pos_offset,
                BEATMAPSET_ITEM_SIZE * self.scale,
                if self.hover {
                    Some(Border::new(Color::BLUE, 1.0))
                } else if self.selected {
                    Some(Border::new(Color::RED, 1.0))
                } else {
                    Some(Border::new(Color::WHITE * 0.8, 1.0))
                }
            ).shape(Shape::Round(5.0, 10))));
        }

        // line 1
        let title_line = Text::new(
            Color::WHITE,
            parent_depth + 4.0,
            self.pos + pos_offset + Vector2::new(15.0, 5.0),
            (15.0 * self.scale.y) as u32,
            self.display_text.clone(),
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
            let mut updated = self.mods.update();
            updated |= self.playmode.update();
            if updated {
                let playmode = self.playmode.0.clone();
                for i in self.beatmaps.iter_mut() {
                    let playmode = i.check_mode_override(playmode.clone());
                    i.diff = get_diff(&i, &playmode, &self.mods);
                }
            }
            

            let mut pos = self.pos + pos_offset + Vector2::new(BEATMAPSET_ITEM_SIZE.x - BEATMAP_ITEM_SIZE.x, BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_Y_PADDING) * self.scale;
            
            // try to find the clicked item
            // // we only care about y pos, because we know we were clicked
            // let mut index:usize = 999;

            // // if x is in correct area to hover over beatmap items
            // let set_item_size = BEATMAPSET_ITEM_SIZE * self.scale;
            // let item_size = BEATMAP_ITEM_SIZE * self.scale;
            // let padding = BEATMAP_ITEM_Y_PADDING * self.scale.y;

            // if self.mouse_pos.x >= self.pos.x + (set_item_size.x - item_size.x) {
            //     let rel_y2 = (self.mouse_pos.y - self.pos.y).abs() - set_item_size.y;
            //     index = ((rel_y2 + padding/2.0) / (item_size.y + padding)).floor() as usize;
            // }

            // if self.mouse_pos.y < self.pos.y {
            //     index = 999;
            // }
            let index = self.index_at(self.mouse_pos);

            for i in 0..self.beatmaps.len() {
                let meta = &mut self.beatmaps[i];

                // bounding rect
                list.push(Box::new(Rectangle::new(
                    [0.2, 0.2, 0.2, 1.0].into(),
                    parent_depth + 5.0,
                    pos,
                    BEATMAP_ITEM_SIZE * self.scale,
                    if i == index { // hover
                        Some(Border::new(Color::BLUE, 1.0))
                    } else if i == self.selected_index { // selected
                        Some(Border::new(Color::RED, 1.0))
                    } else {
                        Some(Border::new(Color::WHITE * 0.8, 1.0))
                    }
                ).shape(Shape::Round(5.0, 10))));

                // version text
                list.push(Box::new(Text::new(
                    Color::WHITE,
                    parent_depth + 4.0,
                    pos + Vector2::new(5.0, 5.0) * self.scale,
                    (12.0 * self.scale.y) as u32,
                    format!("{} - {}", gamemode_display_name(&meta.mode), meta.version),
                    font.clone()
                )));


                // diff text
                let playmode = self.playmode.0.clone();
                if let Some(info) = get_gamemode_info(&meta.check_mode_override(playmode)) { 
                    list.push(Box::new(Text::new(
                        Color::WHITE,
                        parent_depth + 4.0,
                        pos + Vector2::new(5.0, 5.0 + 20.0) * self.scale,
                        (12.0 * self.scale.y) as u32,
                        info.get_diff_string(meta, &self.mods),
                        font.clone()
                    )));
                };


                pos.y += (BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_Y_PADDING) * self.scale.y;
            }
        }
    }

}
