use crate::prelude::*;
use tataku::{
    Color,
    Border,
    Vector2,
};
use engine::{
    gameplay,
    beatmaps::NoteType,
};

#[derive(Default)]
pub struct ManiaNote {
    time: f32, // ms
    column: u8,
    
    hit_time: f32,
    hit: bool,
    missed: bool,
    
    #[cfg(feature="graphics")] pos: Vector2,
    #[cfg(feature="graphics")] color: Color,
    #[cfg(feature="graphics")] sv_mult: f32,
    #[cfg(feature="graphics")] relative_y: f32,
    #[cfg(feature="graphics")] note_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] playfield: Arc<ManiaPlayfield>,
    #[cfg(feature="graphics")] position_function_index: usize,
    #[cfg(feature="graphics")] position_function: Arc<Vec<PositionPoint>>,
    #[cfg(feature="graphics")] mania_skin_settings: Option<Arc<graphics::ManiaSkinSettings>>,

    #[cfg(feature="gameplay")] hitsounds: Vec<gameplay::Hitsound>
}
impl ManiaNote {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        time: f32, column: u8, 
        #[cfg(feature="graphics")] color: Color, 
        #[cfg(feature="graphics")] x: f32, 
        #[cfg(feature="graphics")] sv_mult: f32,
        #[cfg(feature="graphics")] playfield: Arc<ManiaPlayfield>, 
        #[cfg(feature="graphics")] mania_skin_settings: Option<Arc<graphics::ManiaSkinSettings>>,
        #[cfg(feature="gameplay")] hitsounds: Vec<gameplay::Hitsound>,
    ) -> Self {
        Self {
            time,
            column,
            #[cfg(feature="graphics")] color,
            #[cfg(feature="graphics")] sv_mult,
            #[cfg(feature="graphics")] playfield,
            #[cfg(feature="graphics")] mania_skin_settings,
            #[cfg(feature="graphics")] pos: Vector2::with_x(x),

            #[cfg(feature="gameplay")] hitsounds,
            ..Self::default()
        }
    }

    #[cfg(feature="graphics")] 
    fn y_at(&mut self, time: f32) -> f32 {
        let speed = self.sv_mult * if self.playfield.upside_down {-1.0} else {1.0};

        self.playfield.hit_y() - (self.relative_y - ManiaGame::pos_at(&self.position_function, time, &mut self.position_function_index)) * speed
    }
}
impl gameplay::HitObject for ManiaNote {
    fn note_type(&self) -> NoteType { NoteType::Note }
    fn time(&self) -> f32 { self.time }
    fn end_time(&self, hw_miss:f32) -> f32 { self.time + hw_miss }
 
    fn update(&mut self, beatmap_time: f32) {
        #[cfg(feature="graphics")] {
            self.pos.y = self.y_at(beatmap_time); // + self.playfield.note_size().y;
        }
    }
    #[cfg(feature="graphics")] 
    fn draw(&mut self, _time: f32, list: &mut graphics::RenderableCollection) {
        if self.hit || self.pos.y + self.playfield.note_size().y < self.playfield.bounds.pos.y || self.pos.y > self.playfield.bounds.pos.y + self.playfield.bounds.size.y { return } 
        
        if let Some(mut img) = self.note_image.clone() {
            img.pos = self.pos;
            list.push(img);
        } else {
            list.push(graphics::Rectangle::new(
                self.pos,
                self.playfield.note_size(),
                self.color,
            ).border(Border::new(Color::BLACK, self.playfield.note_border_width)));
        }
    }

    fn reset(&mut self) {
        self.hit_time = 0.0;
        self.hit = false;
        self.missed = false;
        #[cfg(feature="graphics")] {
            self.pos.y = 0.0;
            self.position_function_index = 0;
        }
    }

    #[cfg(feature="graphics")]
    fn reload_skin(
        &mut self, 
        source: &graphics::TextureSource, 
        skin_manager: &mut dyn graphics::SkinProvider
    ) {
        self.note_image = None;
        let Some(settings) = &self.mania_skin_settings 
        else { return }; 
        let Some(path) = settings.note_image.get(&self.column) 
        else { return };

        let Some(mut img) = skin_manager.get_texture(
            Path::new(path), 
            source, 
            graphics::SkinUsage::Gamemode, 
            true
        ) else { return };
        
        self.playfield.note_image(&mut img);
        img.color = self.color;
        self.note_image = Some(img);
    }
}
impl ManiaHitObject for ManiaNote {
    fn hit(&mut self, time:f32) {
        self.hit = true;
        self.hit_time = time;
    }
    // fn miss(&mut self, time:f32) {
    //     self.missed = true;
    //     self.hit_time = time;
    // }
    
    #[cfg(feature="graphics")] 
    fn set_sv_mult(&mut self, sv: f32) {
        self.sv_mult = sv;
    }

    #[cfg(feature="graphics")] 
    fn set_position_function(&mut self, p: Arc<Vec<PositionPoint>>) {
        self.position_function = p;

        self.relative_y = ManiaGame::pos_at(&self.position_function, self.time, &mut 0);
    }
    #[cfg(feature="graphics")] 
    fn playfield_changed(&mut self, playfield: Arc<ManiaPlayfield>) {
        self.playfield = playfield;
        self.pos.x = self.playfield.col_pos(self.column);

        if let Some(img) = &mut self.note_image {
            self.playfield.note_image(img);
        }
    }

    #[cfg(feature="gameplay")] 
    fn get_hitsound(&self) -> &Vec<gameplay::Hitsound> {
        &self.hitsounds
    }
}
