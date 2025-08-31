use crate::prelude::*;

#[derive(Default)]
pub struct ManiaHold {
    time: f32, // ms
    column: u8,
    end_time: f32, // ms
    /// when the user started holding
    hold_starts: Vec<f32>,
    hold_ends: Vec<f32>,
    holding: bool,

    #[cfg(feature="graphics")] pos: Vector2,

    #[cfg(feature="graphics")] end_relative_pos: f32,
    #[cfg(feature="graphics")] start_relative_pos: f32,
    
    #[cfg(feature="graphics")] end_y: f32,
    #[cfg(feature="graphics")] sv_mult: f32,
    #[cfg(feature="graphics")] color: Color,
    #[cfg(feature="gameplay")] hitsounds: Vec<Hitsound>,
    #[cfg(feature="graphics")] end_image: Option<Image>,
    #[cfg(feature="graphics")] start_image: Option<Image>,
    #[cfg(feature="graphics")] middle_image: Option<Image>,
    #[cfg(feature="graphics")] playfield: Arc<ManiaPlayfield>,
    #[cfg(feature="gameplay")] position_function_index: usize,
    #[cfg(feature="gameplay")] position_function: Arc<Vec<PositionPoint>>,
    #[cfg(feature="graphics")] mania_skin_settings: Option<Arc<ManiaSkinSettings>>,
}
impl ManiaHold {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        time: f32, end_time: f32, column: u8, 
        #[cfg(feature="graphics")] color: Color, 
        #[cfg(feature="graphics")] x: f32, 
        #[cfg(feature="graphics")] sv_mult: f32,
        
        #[cfg(feature="graphics")] playfield: Arc<ManiaPlayfield>, 
        #[cfg(feature="graphics")] mania_skin_settings: Option<Arc<ManiaSkinSettings>>,

        #[cfg(feature="gameplay")] hitsounds: Vec<Hitsound>,
    ) -> Self {
        Self {
            time, 
            column,
            end_time,
            #[cfg(feature="graphics")] color,
            #[cfg(feature="graphics")] sv_mult,
            #[cfg(feature="graphics")] playfield,
            #[cfg(feature="gameplay")] hitsounds,
            #[cfg(feature="graphics")] pos: Vector2::with_x(x),
            #[cfg(feature="graphics")] mania_skin_settings,
            ..Self::default()
        }
    }

    #[cfg(feature="graphics")] 
    fn y_at(&mut self, beatmap_time: f32) -> (f32, f32) {
        let speed = self.sv_mult * if self.playfield.upside_down {-1.0} else {1.0};

        let rel_start = self.start_relative_pos;
        let rel_end = self.end_relative_pos;

        let mut a = |y| self.playfield.hit_y() - (y - ManiaGame::pos_at(&self.position_function, beatmap_time, &mut self.position_function_index)) * speed;

        (a(rel_start), a(rel_end))
    }
}
impl HitObject for ManiaHold {
    fn note_type(&self) -> NoteType {NoteType::Hold}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,hw_miss:f32) -> f32 {self.end_time + hw_miss}

    fn update(&mut self, beatmap_time: f32) {
        #[cfg(feature="graphics")] {
            let (start, end) = self.y_at(beatmap_time);
            self.pos.y = start;
            self.end_y = end;
    
            if self.playfield.upside_down {
                std::mem::swap(&mut self.end_y, &mut self.pos.y);
            }
            
            let note_size = self.playfield.note_size();
            let y = if self.holding {self.playfield.hit_y()} else {self.pos.y}; // + note_size.y / 2.0;
    
            // update start tex
            if let Some(img) = self.start_image.as_mut() {
                img.pos = self.pos;
            }
    
            // update middle tex
            if let Some(img) = &mut self.middle_image {
                img.pos = Vector2::new(self.pos.x, y);
                let length = self.end_y - (y - note_size.y / 2.0);
    
                img.scale.y = length / img.tex_size().y;
            }
    
            // update end tex
            if let Some(img) = &mut self.end_image {
                img.pos = Vector2::new(self.pos.x, self.end_y);
                // img.scale = self.playfield.note_size() / img.tex_size();
            }
        }
    }

    #[cfg(feature="graphics")] 
    fn draw(&mut self, _time: f32, list: &mut RenderableCollection) {
        // if self.playfield.upside_down {
        //     if self.end_y < 0.0 || self.pos.y > args.window_size[1] as f64 {return}
        // } 
        let note_size = self.playfield.note_size();

        let border = Border::new(
            Color::BLACK, 
            self.playfield.note_border_width
        );
        let color = self.color;

        if self.playfield.upside_down {
            // start
            if self.pos.y > self.playfield.hit_y() {
                list.push(Rectangle::new(
                    self.pos,
                    self.playfield.note_size(),
                    color
                ).border(border));
            }

            // end
            if self.end_y > self.playfield.hit_y() {
                list.push(Rectangle::new(
                    Vector2::new(self.pos.x, self.end_y),
                    self.playfield.note_size(),
                    color,
                ).border(border));
            }
        } else {

            // middle
            if self.end_y < self.playfield.hit_y() {
                let y = if self.holding {self.playfield.hit_y()} else {self.pos.y} + note_size.y / 2.0;

                if let Some(img) = &self.middle_image {
                    list.push(img.clone());
                } else {
                    list.push(Rectangle::new(
                        Vector2::new(self.pos.x, y),
                        Vector2::new(self.playfield.column_width, self.end_y - y),
                        color,
                    ).border(border));
                }
            }

            // start of hold
            if self.pos.y < self.playfield.hit_y() {
                if let Some(img) = &self.start_image {
                    list.push(img.clone());
                } else {
                    list.push(Rectangle::new(
                        self.pos,
                        self.playfield.note_size(),
                        color,
                    ).border(border));
                }
            }


            // end
            if self.end_y < self.playfield.hit_y() {
                if let Some(img) = &self.end_image {
                    list.push(img.clone());
                } else {
                    list.push(Rectangle::new(
                        Vector2::new(self.pos.x, self.end_y + note_size.y),
                        self.playfield.note_size(),
                        color,
                    ).border(border));
                }
            }

        }

    }



    fn reset(&mut self) {
        self.holding = false;
        self.hold_starts.clear();
        self.hold_ends.clear();
        
        #[cfg(feature="graphics")] {
            self.pos.y = 0.0;
            self.position_function_index = 0;
        }
    }

    #[cfg(feature="graphics")]
    fn reload_skin(
        &mut self, 
        source: &TextureSource, 
        skin_manager: &mut dyn SkinProvider,
    ) {
        self.start_image = None;
        self.middle_image = None;
        self.end_image = None;

        let Some(settings) = &self.mania_skin_settings 
        else { return };
        
        // start
        if let Some(path) = settings.note_image_h.get(&self.column)
        && let Some(mut img) = skin_manager.get_texture(
            path, 
            source, 
            SkinUsage::Gamemode, 
            true
        ) {
            self.playfield.note_image(&mut img);
            img.color = self.color;
            self.start_image = Some(img);
        }
        
        // middle
        if let Some(path) = settings.note_image_l.get(&self.column)
        && let Some(mut img) = skin_manager.get_texture(
            path, 
            source, 
            SkinUsage::Gamemode, 
            true
        ) {
            img.origin = Vector2::ZERO;
            img.color = Color::WHITE;
            img.scale.x = self.playfield.column_width / img.tex_size().x;

            self.middle_image = Some(img);
        }

        // end
        if let Some(path) = settings.note_image_t.get(&self.column)
        && let Some(mut img) = skin_manager.get_texture(
            path, 
            source, 
            SkinUsage::Gamemode, 
            true
        ) {
            self.playfield.note_image(&mut img);
            img.scale.y *= -1.0;
            img.color = Color::WHITE;
            self.end_image = Some(img);
        }
    }
}
impl ManiaHitObject for ManiaHold {
    fn was_hit(&self) -> bool {
        !self.hold_starts.is_empty()  
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


    #[cfg(feature="graphics")] 
    fn set_sv_mult(&mut self, sv: f32) {
        self.sv_mult = sv;
    }

    #[cfg(feature="graphics")] 
    fn set_position_function(&mut self, p: Arc<Vec<PositionPoint>>) {
        self.position_function = p;

        self.start_relative_pos = ManiaGame::pos_at(&self.position_function, self.time, &mut 0);
        self.end_relative_pos = ManiaGame::pos_at(&self.position_function, self.end_time, &mut 0);
    }
    
    #[cfg(feature="graphics")] 
    fn playfield_changed(&mut self, playfield: Arc<ManiaPlayfield>) {
        self.playfield = playfield;
        self.pos.x = self.playfield.col_pos(self.column);

        for (img, flip) in [(&mut self.start_image, false), (&mut self.end_image, true)] {
            let Some(img) = img else { continue };
            self.playfield.note_image(img);
            if flip { img.scale.y *= -1.0; }
        }
        if let Some(img) = self.middle_image.as_mut() {
            img.scale.x = self.playfield.column_width / img.tex_size().x;
        }
    }

    #[cfg(feature="gameplay")] 
    fn get_hitsound(&self) -> &Vec<Hitsound> {
        &self.hitsounds
    } 
}
