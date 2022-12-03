use crate::prelude::*;
use super::super::prelude::*;

pub struct ManiaNote {
    pos: Vector2,
    relative_y: f64,
    time: f32, // ms
    column: u8,
    color: Color,

    hit_time: f32,
    hit: bool,
    missed: bool,
    
    position_function: Arc<Vec<PositionPoint>>,
    position_function_index: usize,

    sv_mult: f64,

    playfield: Arc<ManiaPlayfield>,
    note_image: Option<Image>,
    mania_skin_settings: Option<Arc<ManiaSkinSettings>>,

    hitsounds: Vec<Hitsound>
}
impl ManiaNote {
    pub async fn new(
        time:f32, column:u8, color: Color, x:f64, 
        
        sv_mult: f64,

        playfield: Arc<ManiaPlayfield>, mania_skin_settings: Option<Arc<ManiaSkinSettings>>,

        hitsounds: Vec<Hitsound>,
    ) -> Self {
        Self {
            time,
            position_function: Arc::new(Vec::new()),
            relative_y: 0.0,
            sv_mult,
            column,
            color,

            hit_time: 0.0,
            hit: false,
            missed: false,
            pos: Vector2::x_only(x),

            playfield,
            note_image:None,
            position_function_index: 0,

            mania_skin_settings,

            hitsounds,
        }
    }

    fn y_at(&mut self, time: f32) -> f64 {
        let speed = self.sv_mult * if self.playfield.upside_down {-1.0} else {1.0};

        self.playfield.hit_y() - (self.relative_y - ManiaGame::pos_at(&self.position_function, time, &mut self.position_function_index)) * speed
    }
}
#[async_trait]
impl HitObject for ManiaNote {
    fn note_type(&self) -> NoteType { NoteType::Note }
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {self.time + hw_miss}

    async fn update(&mut self, beatmap_time: f32) {
        self.pos.y = self.y_at(beatmap_time);
    }
    async fn draw(&mut self, args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.pos.y + self.playfield.note_size().y < 0.0 || self.pos.y > args.window_size[1] as f64 { return }
        if self.hit { return }
        
        
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
    }

    async fn reset(&mut self) {
        self.pos.y = 0.0;
        self.hit_time = 0.0;
        self.hit = false;
        self.missed = false;
        self.position_function_index = 0;
    }

    async fn reload_skin(&mut self) {
        let mut note_image = None;
        if let Some(settings) = &self.mania_skin_settings {
            let map = &settings.note_image;
            
            if let Some(path) = map.get(&self.column) {
                if let Some(img) = SkinManager::get_texture_grayscale(path, true, true).await {
                    let mut img = img.clone();
                    img.current_color = self.color;
                    img.origin = Vector2::zero();
                    img.depth = MANIA_NOTE_DEPTH;
                    note_image = Some(img);
                }
            }
        }

        self.note_image = note_image;
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

    fn set_sv_mult(&mut self, sv: f64) {
        self.sv_mult = sv;
    }

    fn set_position_function(&mut self, p: Arc<Vec<PositionPoint>>) {
        self.position_function = p;

        self.relative_y = ManiaGame::pos_at(&self.position_function, self.time, &mut 0);
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
