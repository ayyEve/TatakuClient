use crate::prelude::*;

const MANIA_NOTE_DEPTH: f64 = 100.0;
const MANIA_SLIDER_DEPTH: f64 = 100.1;

pub trait ManiaHitObject: HitObject {
    fn hit(&mut self, time:f32);
    fn release(&mut self, _time:f32) {}
    fn miss(&mut self, time:f32);
    fn was_hit(&self) -> bool {false}

    fn y_at(&self, time:f32) -> f64;
    fn time_at(&self, y: f32) -> f32;
    fn set_sv(&mut self, sv:f32);
}

// note
#[derive(Clone)]
pub struct ManiaNote {
    pos: Vector2,
    time: f32, // ms
    // column: u8,
    color: Color,

    hit_time: f32,
    hit: bool,
    missed: bool,
    speed: f32,

    playfield: Arc<ManiaPlayfieldSettings>,

    note_image: Option<Image>
}
impl ManiaNote {
    pub async fn new(time:f32, column:u8, color: Color, x:f64, playfield: Arc<ManiaPlayfieldSettings>, mania_skin_settings: Option<Arc<ManiaSkinSettings>>) -> Self {
        let mut note_image = None;
        if let Some(settings) = &mania_skin_settings {
            let map = &settings.note_image;
            
            if let Some(path) = map.get(&column) {
                if let Some(img) = SkinManager::get_texture_grayscale(path, true, true).await {
                    let mut img = img.clone();
                    img.current_color = color;
                    img.origin = Vector2::zero();
                    img.depth = MANIA_NOTE_DEPTH;
                    note_image = Some(img);
                }
            }
        }
        
        Self {
            time, 
            speed: 1.0,
            // column,
            color,

            hit_time: 0.0,
            hit: false,
            missed: false,
            pos: Vector2::new(x, 0.0),

            playfield,
            note_image
        }
    }
}
#[async_trait]
impl HitObject for ManiaNote {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {self.time + hw_miss}

    async fn update(&mut self, beatmap_time: f32) {
        self.pos.y = self.y_at(beatmap_time);
    }
    async fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();
        if self.pos.y + self.playfield.note_size().y < 0.0 || self.pos.y > args.window_size[1] as f64 {return list}
        if self.hit {return list}
        
        
        if let Some(img) = &self.note_image {
            let mut img = img.clone();
            
            img.current_pos = self.pos;
            img.current_scale = self.playfield.note_size() / img.tex_size();
            list.push(Box::new(img));
        } else {
            list.push(Box::new(Rectangle::new(
                self.color,
                MANIA_NOTE_DEPTH,
                self.pos,
                self.playfield.note_size(),
                Some(Border::new(Color::BLACK, self.playfield.note_border_width))
            )));
        }

        list
    }

    async fn reset(&mut self) {
        self.pos.y = 0.0;
        self.hit_time = 0.0;
        self.hit = false;
        self.missed = false;
    }
}
impl ManiaHitObject for ManiaNote {
    fn hit(&mut self, time:f32) {
        self.hit = true;
        self.hit_time = time;
    }
    fn miss(&mut self, time:f32) {
        self.missed = true;
        self.hit_time = time;
    }

    fn y_at(&self, time:f32) -> f64 {
        let speed = self.speed * if self.playfield.upside_down {-1.0} else {1.0};
        self.playfield.hit_y() - ((self.time - time) * speed) as f64
    }

    fn time_at(&self, y:f32) -> f32 {
        let speed = self.speed * if self.playfield.upside_down {-1.0} else {1.0};
        self.time + (y - self.playfield.hit_y() as f32) / speed 
    }


    fn set_sv(&mut self, sv:f32) {
        self.speed = sv;
    }
}

// slider
#[derive(Clone)]
pub struct ManiaHold {
    pos: Vector2,
    time: f32, // ms
    end_time: f32, // ms
    // column: u8,
    color: Color,

    /// when the user started holding
    hold_starts: Vec<f32>,
    hold_ends: Vec<f32>,
    holding: bool,

    speed: f32,
    //TODO: figure out how to pre-calc this
    end_y: f64,

    playfield: Arc<ManiaPlayfieldSettings>,

    start_image: Option<Image>,
    end_image: Option<Image>,
    middle_image: Option<Image>,
}
impl ManiaHold {
    pub async fn new(time:f32, end_time:f32, column: u8, color: Color, x:f64, playfield: Arc<ManiaPlayfieldSettings>, mania_skin_settings: Option<Arc<ManiaSkinSettings>>) -> Self {
        let mut start_image = None;
        if let Some(settings) = &mania_skin_settings {
            let map = &settings.note_image_h;
            
            if let Some(path) = map.get(&column) {
                if let Some(mut img) = SkinManager::get_texture_grayscale(path, true, true).await {
                    img.current_color = color;
                    img.origin = Vector2::zero();
                    img.depth = MANIA_NOTE_DEPTH;

                    start_image = Some(img);
                }
            }
        }

        let mut middle_image = None;
        if let Some(settings) = &mania_skin_settings {
            let map = &settings.note_image_l;
            
            if let Some(path) = map.get(&column) {
                if let Some(mut img) = SkinManager::get_texture_grayscale(path, true, true).await {
                    img.origin = Vector2::zero();
                    img.current_color = Color::WHITE;
                    img.depth = MANIA_NOTE_DEPTH;

                    middle_image = Some(img);
                }
            }
        }
        
        let mut end_image = None;
        if let Some(settings) = &mania_skin_settings {
            let map = &settings.note_image_t;
            
            if let Some(path) = map.get(&column) {
                if let Some(mut img) = SkinManager::get_texture_grayscale(path, true, true).await {
                    img.origin = Vector2::zero();
                    img.current_color = Color::WHITE;
                    img.current_scale.y *= -1.0;
                    img.depth = MANIA_NOTE_DEPTH;
                    end_image = Some(img)
                }
            }
        }

        Self {
            time, 
            end_time,
            // column,
            speed: 1.0,
            holding: false,
            color,

            pos: Vector2::x_only(x),
            hold_starts: Vec::new(),
            hold_ends: Vec::new(),
            end_y: 0.0,

            playfield,
            start_image,
            end_image,
            middle_image,
        }
    }
}
#[async_trait]
impl HitObject for ManiaHold {
    fn note_type(&self) -> NoteType {NoteType::Hold}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,hw_miss:f32) -> f32 {self.end_time + hw_miss}

    async fn update(&mut self, beatmap_time: f32) {
        // self.pos.x = HIT_POSITION.x + (self.time as f64 - beatmap_time as f64) * self.speed;
        let speed = self.speed * if self.playfield.upside_down {-1.0} else {1.0};
        
        self.end_y = self.playfield.hit_y() - ((self.end_time - beatmap_time) * speed) as f64;
        self.pos.y = self.playfield.hit_y() - ((self.time - beatmap_time) * speed) as f64;

        if self.playfield.upside_down {
            std::mem::swap(&mut self.end_y, &mut self.pos.y)
        }

        let note_size = self.playfield.note_size();
        let y = if self.holding {self.playfield.hit_y()} else {self.pos.y} + note_size.y / 2.0;

        // update start tex
        if let Some(img) = &mut self.start_image {
            img.current_pos = self.pos;
            img.current_scale = self.playfield.note_size() / img.tex_size();
        }

        // update middle tex
        if let Some(img) = &mut self.middle_image {
            img.current_pos = Vector2::new(self.pos.x, y);
            img.current_scale = Vector2::new(self.playfield.column_width, self.end_y - y + note_size.y) / img.tex_size();
        }

        // update end tex
        if let Some(img) = &mut self.end_image {
            img.current_pos = Vector2::new(self.pos.x, self.end_y + note_size.y);
            img.current_scale = self.playfield.note_size() / img.tex_size();
        }

    }
    async fn draw(&mut self, _args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();
        // if self.playfield.upside_down {
        //     if self.end_y < 0.0 || self.pos.y > args.window_size[1] as f64 {return}
        // } 
        let note_size = self.playfield.note_size();

        let border = Some(Border::new(Color::BLACK, self.playfield.note_border_width));
        let color = self.color;

        if self.playfield.upside_down {
            // start
            if self.pos.y > self.playfield.hit_y() {
                list.push(Box::new(Rectangle::new(
                    color,
                    MANIA_NOTE_DEPTH,
                    self.pos,
                    self.playfield.note_size(),
                    border.clone()
                )));
            }

            // end
            if self.end_y > self.playfield.hit_y() {
                list.push(Box::new(Rectangle::new(
                    color,
                    MANIA_NOTE_DEPTH,
                    Vector2::new(self.pos.x, self.end_y),
                    self.playfield.note_size(),
                    border.clone()
                )));
            }
        } else {

            // middle
            if self.end_y < self.playfield.hit_y() {
                let y = if self.holding {self.playfield.hit_y()} else {self.pos.y} + note_size.y / 2.0;

                if let Some(img) = &self.middle_image {
                    list.push(Box::new(img.clone()));
                } else {
                    list.push(Box::new(Rectangle::new(
                        color,
                        MANIA_SLIDER_DEPTH,
                        Vector2::new(self.pos.x, y),
                        Vector2::new(self.playfield.column_width, self.end_y - y),
                        border.clone()
                    )));
                }
            }

            // start of hold
            if self.pos.y < self.playfield.hit_y() {
                if let Some(img) = &self.start_image {
                    list.push(Box::new(img.clone()));
                } else {
                    list.push(Box::new(Rectangle::new(
                        color,
                        MANIA_NOTE_DEPTH,
                        self.pos,
                        self.playfield.note_size(),
                        border.clone()
                    )));
                }
            }


            // end
            if self.end_y < self.playfield.hit_y() {
                if let Some(img) = &self.end_image {
                    list.push(Box::new(img.clone()));
                } else {
                    list.push(Box::new(Rectangle::new(
                        color,
                        MANIA_NOTE_DEPTH,
                        Vector2::new(self.pos.x, self.end_y + note_size.y),
                        self.playfield.note_size(),
                        border.clone()
                    )));
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

        list
    }

    async fn reset(&mut self) {
        self.pos.y = 0.0;
        self.holding = false;
        self.hold_starts.clear();
        self.hold_ends.clear();
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

    fn y_at(&self, time:f32) -> f64 {
        let speed = self.speed * if self.playfield.upside_down {-1.0} else {1.0};
        self.playfield.hit_y() - ((self.time - time) * speed) as f64
    }

    fn time_at(&self, y:f32) -> f32 {
        let speed = self.speed * if self.playfield.upside_down {-1.0} else {1.0};
        self.time + (y - self.playfield.hit_y() as f32) / speed 
    }

    fn set_sv(&mut self, sv:f32) {
        self.speed = sv;
    }
}
