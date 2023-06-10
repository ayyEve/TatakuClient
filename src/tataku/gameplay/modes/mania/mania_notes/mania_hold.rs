use crate::prelude::*;
use super::super::prelude::*;

const MANIA_SLIDER_DEPTH:f32 = 100.1;

pub struct ManiaHold {
    pos: Vector2,
    time: f32, // ms
    end_time: f32, // ms

    start_relative_pos: f32,
    end_relative_pos: f32,
    column: u8,
    color: Color,

    /// when the user started holding
    hold_starts: Vec<f32>,
    hold_ends: Vec<f32>,
    holding: bool,

    position_function: Arc<Vec<PositionPoint>>,
    position_function_index: usize,

    sv_mult: f32,
    //TODO: figure out how to pre-calc this
    end_y: f32,

    playfield: Arc<ManiaPlayfield>,

    start_image: Option<Image>,
    end_image: Option<Image>,
    middle_image: Option<Image>,

    hitsounds: Vec<Hitsound>,

    mania_skin_settings: Option<Arc<ManiaSkinSettings>>,
}
impl ManiaHold {
    pub async fn new(
        time:f32, end_time:f32, column: u8, color: Color, x:f32, 
        
        sv_mult: f32,
        
        playfield: Arc<ManiaPlayfield>, mania_skin_settings: Option<Arc<ManiaSkinSettings>>,

        hitsounds: Vec<Hitsound>,
    ) -> Self {

        Self {
            time, 
            end_time,
            column,
            position_function: Arc::new(Vec::new()),
            position_function_index: 0,

            start_relative_pos: 0.0,
            end_relative_pos: 0.0,
            sv_mult,
            holding: false,
            color,

            pos: Vector2::with_x(x),
            hold_starts: Vec::new(),
            hold_ends: Vec::new(),
            end_y: 0.0,

            playfield,
            start_image: None,
            end_image: None,
            middle_image: None,
            mania_skin_settings,

            hitsounds
        }
    }

    fn y_at(&mut self, beatmap_time: f32) -> (f32, f32) {
        let speed = self.sv_mult * if self.playfield.upside_down {-1.0} else {1.0};

        let rel_start = self.start_relative_pos;
        let rel_end = self.end_relative_pos;

        let mut a = |y| self.playfield.hit_y() - (y - ManiaGame::pos_at(&self.position_function, beatmap_time, &mut self.position_function_index)) * speed;

        (a(rel_start), a(rel_end))
    }
}
#[async_trait]
impl HitObject for ManiaHold {
    fn note_type(&self) -> NoteType {NoteType::Hold}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,hw_miss:f32) -> f32 {self.end_time + hw_miss}

    async fn update(&mut self, beatmap_time: f32) {
        let (start, end) = self.y_at(beatmap_time);

        self.pos.y = start;
        self.end_y = end;

        if self.playfield.upside_down {
            std::mem::swap(&mut self.end_y, &mut self.pos.y)
        }

        let note_size = self.playfield.note_size();
        let y = if self.holding {self.playfield.hit_y()} else {self.pos.y} + note_size.y / 2.0;

        // update start tex
        if let Some(img) = &mut self.start_image {
            img.pos = self.pos;
            img.scale = self.playfield.note_size() / img.tex_size();
        }

        // update middle tex
        if let Some(img) = &mut self.middle_image {
            img.pos = Vector2::new(self.pos.x, y);
            img.scale = Vector2::new(self.playfield.column_width, self.end_y - y + note_size.y) / img.tex_size();
            if img.scale.y < 0.0 {
                img.origin.y = img.tex_size().y;
            }
        }

        // update end tex
        if let Some(img) = &mut self.end_image {
            img.pos = Vector2::new(self.pos.x, self.end_y + note_size.y);
            img.scale = self.playfield.note_size() / img.tex_size();
        }

    }
    async fn draw(&mut self, list: &mut RenderableCollection) {
        // if self.playfield.upside_down {
        //     if self.end_y < 0.0 || self.pos.y > args.window_size[1] as f64 {return}
        // } 

        let border = Some(Border::new(Color::BLACK, self.playfield.note_border_width));
        let color = self.color;
        let hit_y = self.playfield.hit_y();
        let note_size = self.playfield.note_size();
        // let pf_height = self.playfield.window_size.y;

        let pf_top = 0.0; //-pf_height;


        if self.playfield.upside_down {
            // start
            if self.pos.y > hit_y {
                list.push(Rectangle::new(
                    color,
                    MANIA_NOTE_DEPTH,
                    self.pos,
                    self.playfield.note_size(),
                    border.clone()
                ));
            }

            // end
            if self.end_y > hit_y {
                list.push(Rectangle::new(
                    color,
                    MANIA_NOTE_DEPTH,
                    Vector2::new(self.pos.x, self.end_y),
                    self.playfield.note_size(),
                    border.clone()
                ));
            }
        } else {

            // middle
            if self.end_y < hit_y && self.pos.y > pf_top {
                let y = if self.holding { hit_y } else { self.pos.y } + note_size.y / 2.0;

                if let Some(img) = &self.middle_image {
                    list.push(img.clone());
                } else {
                    list.push(Rectangle::new(
                        color,
                        MANIA_SLIDER_DEPTH,
                        Vector2::new(self.pos.x, y),
                        Vector2::new(self.playfield.column_width, self.end_y - y),
                        border.clone()
                    ));
                }
            }


            // start of hold
            if self.pos.y < hit_y && self.pos.y > pf_top {
                if let Some(img) = &self.start_image {
                    list.push(img.clone());
                } else {
                    list.push(Rectangle::new(
                        color,
                        MANIA_NOTE_DEPTH,
                        self.pos,
                        self.playfield.note_size(),
                        border.clone()
                    ));
                }
            }


            // end
            if self.end_y < hit_y && self.end_y > pf_top {
                if let Some(img) = &self.end_image {
                    list.push(img.clone());
                } else {
                    list.push(Rectangle::new(
                        color,
                        MANIA_NOTE_DEPTH,
                        Vector2::new(self.pos.x, self.end_y + note_size.y),
                        self.playfield.note_size(),
                        border.clone()
                    ));
                }
            }

        }

        // draw hold fragments
        // for i in 0..self.hold_ends.len() {
        //     let start = self.hold_starts[i];
        //     let end = self.hold_ends[i];
        //     let y = hit_y() - (end - start) * self.speed;

        //     list.push(Box::new(Rectangle::new(
        //         Color::YELLOW,
        //         -100.0,
        //         Vector2::new(self.pos.x, y),
        //         Vector2::new(COLUMN_WIDTH, self.end_y - y),
        //         Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
        //     )));
        // }

    }

    async fn reset(&mut self) {
        self.pos.y = 0.0;
        self.holding = false;
        self.hold_starts.clear();
        self.hold_ends.clear();
        self.position_function_index = 0;
    }

    async fn reload_skin(&mut self) {
        
        let mut start_image = None;
        if let Some(settings) = &self.mania_skin_settings {
            let map = &settings.note_image_h;
            
            if let Some(path) = map.get(&self.column) {
                if let Some(mut img) = SkinManager::get_texture_grayscale(path, true, true).await {
                    img.color = self.color;
                    img.origin = Vector2::ZERO;
                    img.depth = MANIA_NOTE_DEPTH;

                    start_image = Some(img);
                }
            }
        }

        let mut middle_image = None;
        if let Some(settings) = &self.mania_skin_settings {
            let map = &settings.note_image_l;
            
            if let Some(path) = map.get(&self.column) {
                if let Some(mut img) = SkinManager::get_texture_grayscale(path, true, true).await {
                    img.origin = Vector2::ZERO;
                    img.color = Color::WHITE;
                    img.depth = MANIA_NOTE_DEPTH;

                    middle_image = Some(img);
                }
            }
        }
        
        let mut end_image = None;
        if let Some(settings) = &self.mania_skin_settings {
            let map = &settings.note_image_t;
            
            if let Some(path) = map.get(&self.column) {
                if let Some(mut img) = SkinManager::get_texture_grayscale(path, true, true).await {
                    img.origin = Vector2::ZERO;
                    img.color = Color::WHITE;
                    img.scale.y *= -1.0;
                    img.depth = MANIA_NOTE_DEPTH;
                    end_image = Some(img)
                }
            }
        }

        self.start_image = start_image;
        self.middle_image = middle_image;
        self.end_image = end_image;
    }
}
impl ManiaHitObject for ManiaHold {
    fn was_hit(&self) -> bool {
        self.hold_starts.len() > 0  
    }

    // key pressed
    fn hit(&mut self, time:f32) {
        self.hold_starts.push(time);
        self.holding = true;
    }
    fn release(&mut self, time:f32) {
        self.hold_ends.push(time);
        self.holding = false;
    }

    //
    fn miss(&mut self, _time:f32) {}

    fn set_sv_mult(&mut self, sv: f32) {
        self.sv_mult = sv;
    }

    fn set_position_function(&mut self, p: Arc<Vec<PositionPoint>>) {
        self.position_function = p;

        self.start_relative_pos = ManiaGame::pos_at(&self.position_function, self.time, &mut 0);
        self.end_relative_pos = ManiaGame::pos_at(&self.position_function, self.end_time, &mut 0);
    }
    
    fn playfield_changed(&mut self, playfield: Arc<ManiaPlayfield>) {
        self.playfield = playfield;
        
        self.pos.x = self.playfield.col_pos(self.column);
    }

    fn get_hitsound(&self) -> &Vec<Hitsound> {
        &self.hitsounds
    } 
    
    fn set_skin_settings(&mut self, settings: Option<Arc<ManiaSkinSettings>>) {
        self.mania_skin_settings = settings;
    }
}
