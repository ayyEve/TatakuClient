use crate::prelude::*;


const BEATMAP_ITEM_Y_PADDING:f32 = 5.0;
pub const BEATMAPSET_ITEM_SIZE:Vector2 = Vector2::new(700.0, 50.0);
const BEATMAP_ITEM_SIZE:Vector2 = Vector2::new(BEATMAPSET_ITEM_SIZE.x() * 0.8, 50.0);



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

    button_image: Option<Image>, //Option<GenericButtonImage>,
    theme: ThemeHelper,
    skin: CurrentSkinHelper,
}
impl BeatmapsetItem {
    pub async fn new(beatmaps: Vec<BeatmapMetaWithDiff>, display_text: String) -> BeatmapsetItem {
        let mut button_image = SkinManager::get_texture("menu-button-background", true).await; //GenericButtonImage::new(Vector2::ZERO, BEATMAPSET_ITEM_SIZE).await,
        if let Some(image) = &mut button_image {
            image.origin = Vector2::ZERO;
        }

        let skin = CurrentSkinHelper::new();

        BeatmapsetItem {
            beatmaps, 
            pos: Vector2::ZERO,
            hover: false,
            selected: false,
            display_text,
            scale: Vector2::ONE,

            theme: ThemeHelper::new(),
            selected_index: 0,
            mouse_pos: Vector2::ZERO,
            mods: ModManagerHelper::new(),
            playmode: CurrentPlaymodeHelper::new(),
            double_clicked: false,
            button_image,
            skin,
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
        let my_pos = self.get_pos();
        let scale = self.get_scale();

        if pos.y < my_pos.y || pos.y > my_pos.y + self.size().y { return index }

        let set_item_size = BEATMAPSET_ITEM_SIZE * scale;
        let item_size = BEATMAP_ITEM_SIZE * scale;
        let padding = BEATMAP_ITEM_Y_PADDING * scale.y;

        if pos.x < my_pos.x + (set_item_size.x - item_size.x) { return index }

        let rel_y = (pos.y - my_pos.y).abs() - set_item_size.y;
        // (((rel_y + padding / 2.0) / (item_size.y + padding)).floor() as usize).clamp(0, self.beatmaps.len() - 1)

        let btn_scale = self.theme.get_scale(ThemeScale::BeatmapSelectMapScale).unwrap_or(Vector2::ONE).y * self.scale.y;
        let btn_selected_scale = self.theme.get_scale(ThemeScale::BeatmapSelectMapSelectedScale).unwrap_or(Vector2::ONE).y * self.scale.y;

        let mut y = padding;
        for i in 0..self.beatmaps.len() {
            y += BEATMAP_ITEM_SIZE.y * if self.selected_index == i {btn_selected_scale} else {btn_scale} + padding;
            if rel_y <= y { return i }
        }

        return 99
    }

    fn get_scale(&self) -> Vector2 {
        (if self.hover { 
            self.theme.get_scale(ThemeScale::BeatmapSelectSetHoveredScale) 
        } else if self.selected { 
            self.theme.get_scale(ThemeScale::BeatmapSelectSetSelectedScale) 
        } else { 
            self.theme.get_scale(ThemeScale::BeatmapSelectSetScale) 
        }).unwrap_or(Vector2::ONE)
        * self.scale
    }


}
impl ScrollableItemGettersSetters for BeatmapsetItem {
    fn size(&self) -> Vector2 {
        let scale = self.get_scale();

        if !self.selected {
            BEATMAPSET_ITEM_SIZE * scale
        } else {
            let map_scale = self.theme.get_scale(ThemeScale::BeatmapSelectMapScale).unwrap_or(Vector2::ONE) * self.scale;
            let count = self.beatmaps.len() as f32;
            BEATMAPSET_ITEM_SIZE * scale + 
            Vector2::with_y(
                // button sizes
                BEATMAP_ITEM_SIZE.y * count * map_scale.y
                // button margins, with extra at the bottom
                + BEATMAP_ITEM_Y_PADDING * (count + 1.0) * scale.y
            ) 
        }
    }
    fn get_tag(&self) -> String {self.beatmaps[self.selected_index.min(self.beatmaps.len()-1)].beatmap_hash.clone()}
    // fn set_tag(&mut self, _tag:&str) {self.pending_play = false} // bit of a jank strat: when this is called, reset the pending_play property
    fn get_pos(&self) -> Vector2 {
        self.pos +
        (if self.hover { 
            self.theme.get_pos(ThemePosition::BeatmapSelectSetHoveredOffset) 
        } else if self.selected { 
            self.theme.get_pos(ThemePosition::BeatmapSelectSetSelectedOffset) 
        } else { 
            self.theme.get_pos(ThemePosition::BeatmapSelectSetOffset) 
        }).unwrap_or(Vector2::ZERO)
    }
    fn set_pos(&mut self, pos:Vector2) { self.pos = pos }

    fn get_hover(&self) -> bool { self.hover }
    fn set_hover(&mut self, hover:bool) { self.hover = hover }
    fn get_selected(&self) -> bool { self.selected }
    fn set_selected(&mut self, selected:bool) { self.selected = selected }
}
impl ScrollableItem for BeatmapsetItem {
    fn get_value(&self) -> Box<dyn std::any::Any> { Box::new(self.double_clicked) }
    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.scale = scale;
        if let Some(btn) = &mut self.button_image {
            btn.set_size(BEATMAPSET_ITEM_SIZE * scale)
        }
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

        self.theme.update();
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        let color = Color::new(0.2, 0.2, 0.2, 1.0);
        let text_color = self.skin.song_select_inactive_text.unwrap_or_else(||self.theme.get_color(ThemeColor::BeatmapSelectText).unwrap_or_else(||Color::WHITE));

        let hovered_text_color = self.skin.song_select_active_text.unwrap_or_else(||self.theme.get_color(ThemeColor::BeatmapSelectTextHovered).unwrap_or_else(||text_color));
        let selected_text_color = self.skin.song_select_active_text.unwrap_or_else(||self.theme.get_color(ThemeColor::BeatmapSelectTextSelected).unwrap_or_else(||text_color));

        let scale = self.get_scale();
        let pos = self.get_pos() + pos_offset;

        // draw button
        if let Some(mut button_image) = self.button_image.clone() {
            // button_image
            let color = if self.hover { 
                self.theme.get_color(ThemeColor::BeatmapSelectSetHover).unwrap_or_else(||Color::BLUE)
            } else if self.selected { 
                self.theme.get_color(ThemeColor::BeatmapSelectSetSelect).unwrap_or_else(||Color::RED)
            } else { 
                self.theme.get_color(ThemeColor::BeatmapSelectSetBg).unwrap_or_else(||Color::new(0.2, 0.2, 0.2, 1.0))
            };

            button_image.pos = pos;
            button_image.color = color;
            button_image.set_size(BEATMAPSET_ITEM_SIZE * scale);
            list.push(button_image);
        } else {
            list.push(Rectangle::new(
                pos,
                BEATMAPSET_ITEM_SIZE * scale,
                color,
                if self.hover {
                    let color = self.theme.get_color(ThemeColor::BeatmapSelectSetHover).unwrap_or_else(||Color::BLUE);
                    Some(Border::new(color, 1.0))
                } else if self.selected {
                    let color = self.theme.get_color(ThemeColor::BeatmapSelectSetSelect).unwrap_or_else(||Color::RED);
                    Some(Border::new(color, 1.0))
                } else {
                    Some(Border::new(Color::WHITE * 0.8, 1.0))
                }
            ).shape(Shape::Round(5.0)));
        }

        // title line
        list.push(Text::new(
            pos + Vector2::new(20.0, 10.0) * scale,
            15.0 * scale.y,
            self.display_text.clone(),
            if self.selected { selected_text_color } else if self.hover {hovered_text_color} else { text_color },
            Font::Main
        ));


        // if selected, draw map items
        if self.selected {
            let mut updated = self.mods.update();
            updated |= self.playmode.update();
            if updated {
                let playmode = self.playmode.0.clone();
                for i in self.beatmaps.iter_mut() {
                    let playmode = i.check_mode_override(playmode.clone());
                    i.diff = get_diff(&i, &playmode, &self.mods);
                    i.sort_pending = true;
                }
            }
            
            let set_button_size = BEATMAPSET_ITEM_SIZE * scale;
            let padding = BEATMAP_ITEM_Y_PADDING * scale.y;

            let btn_scale = self.theme.get_scale(ThemeScale::BeatmapSelectMapScale).unwrap_or(Vector2::ONE) * self.scale;
            let btn_hover_scale = self.theme.get_scale(ThemeScale::BeatmapSelectMapHoveredScale).unwrap_or(Vector2::ONE) * self.scale;
            let btn_selected_scale = self.theme.get_scale(ThemeScale::BeatmapSelectMapSelectedScale).unwrap_or(Vector2::ONE) * self.scale;
            
            let btn_offset = self.theme.get_pos(ThemePosition::BeatmapSelectMapOffset).unwrap_or(Vector2::ZERO);
            let btn_hover_offset = self.theme.get_pos(ThemePosition::BeatmapSelectMapHoveredOffset).unwrap_or(Vector2::ZERO);
            let btn_selected_offset = self.theme.get_pos(ThemePosition::BeatmapSelectMapSelectedOffset).unwrap_or(Vector2::ZERO);

            let btn_color = self.theme.get_color(ThemeColor::BeatmapSelectMapBg).unwrap_or_else(||Color::new(0.2, 0.2, 0.2, 1.0));
            let btn_hover_color = self.theme.get_color(ThemeColor::BeatmapSelectMapHover).unwrap_or_else(||Color::BLUE);
            let btn_select_color = self.theme.get_color(ThemeColor::BeatmapSelectMapSelect).unwrap_or_else(||Color::RED);


            let mut pos = pos + Vector2::new(set_button_size.x, set_button_size.y + padding);

            // try to find the clicked item
            let index = self.index_at(self.mouse_pos);
            let btn_base = self.button_image.clone();

            for i in 0..self.beatmaps.len() {
                let meta = &mut self.beatmaps[i];
                let hover = i == index;
                let selected = i == self.selected_index;

                let color = if hover { btn_hover_color } else if selected { btn_select_color } else { btn_color };
                let btn_scale = if hover { btn_hover_scale } else if selected { btn_selected_scale } else { btn_scale };
                let mut btn_pos = pos + if hover { btn_hover_offset } else if selected { btn_selected_offset } else { btn_offset } * btn_scale;
                // maintain right alignment
                btn_pos.x -= BEATMAP_ITEM_SIZE.x * btn_scale.x;

                // bounding rect
                if let Some(mut btn) = btn_base.clone() {
                    btn.color = color;
                    btn.pos = btn_pos;

                    btn.set_size(BEATMAP_ITEM_SIZE * btn_scale);
                    list.push(btn)
                    // btn.draw(args, parent_depth + 5.0, Vector2::ZERO, list);
                } else {
                    let radius = 1.0 * btn_scale.y;
                    list.push(Rectangle::new(
                        btn_pos,
                        BEATMAP_ITEM_SIZE * btn_scale,
                        Color::new(0.2, 0.2, 0.2, 1.0),
                        Some(Border::new(color, radius))
                    ).shape(Shape::Round(5.0)));
                }

                // version text
                list.push(Text::new(
                    btn_pos + Vector2::new(10.0, 5.0) * btn_scale,
                    12.0 * btn_scale.y,
                    format!("{} - {}", gamemode_display_name(&meta.mode), meta.version),
                    if selected { selected_text_color } else { text_color },
                    Font::Main
                ));


                // diff text
                let playmode = self.playmode.0.clone();
                if let Some(info) = get_gamemode_info(&meta.check_mode_override(playmode)) { 
                    list.push(Text::new(
                        btn_pos + Vector2::new(10.0, 5.0 + 20.0) * btn_scale,
                        12.0 * btn_scale.y,
                        info.get_diff_string(meta, &self.mods),
                        if selected { selected_text_color } else { text_color },
                        Font::Main
                    ));
                };


                pos.y += (BEATMAP_ITEM_SIZE.y * btn_scale.y) + padding;
            }
        }
    
    }

}
