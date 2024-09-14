use crate::prelude::*;

#[derive(Clone)]
pub struct ManiaPlayfield {
    pub settings: ManiaPlayfieldSettings,
    pub bounds: Bounds,
    pub col_count: u8,
    pub total_width: f32,

    /// bullshit peppy fuck
    pub skin_hit_pos: f32,

    column_origin: Arc<AtomicU32>,
}
impl ManiaPlayfield {
    pub fn new(mut settings: ManiaPlayfieldSettings, bounds: Bounds, col_count: u8, skin_hit_pos: f32) -> Self {
        let window_size = WindowSize::get().0;
        let total_width = col_count as f32 * settings.column_width;

        if bounds.size != window_size {
            // if we're not fullscreen, center the playfield
            settings.x_offset = bounds.pos.x + (total_width - bounds.size.x) / 2.0;
        }

        Self {
            settings, 
            bounds,
            col_count,
            total_width,

            skin_hit_pos,
            column_origin: Arc::new(AtomicU32::new(0))
        }
    }

    /// y coordinate of the hit area
    pub fn hit_y(&self) -> f32 {
        self.bounds.pos.y + if self.upside_down {
            self.hit_pos
        } else {
            self.bounds.size.y - self.hit_pos
        }
    }

    /// leftmost x coordinate of the given column
    pub fn col_pos(&self, col: u8) -> f32 {
        let x_offset = self.x_offset + (self.bounds.size.x - self.total_width) / 2.0;

        x_offset + (self.column_width + self.column_spacing) * col as f32
    }

    /// calculate the note's origin and scale
    /// 
    /// this assumes notes are drawn with the origin bottom-left
    pub fn note_image(&self, img: &mut Image) {
        let tex_size = img.tex_size();
        // img.origin = Vector2::with_y(tex_size.y - self.skin_hit_pos);
        
        let a = f32::from_bits(self.column_origin.load(Ordering::Relaxed));
        img.origin = Vector2::with_y(a + tex_size.y / 2.0);

        img.scale = Vector2::ONE * (self.column_width / tex_size.x);
    }

    /// calculate the column's image's origin
    /// 
    /// this assumes notes are drawn with the origin bottom-left
    pub fn column_image(&self, img: &mut Image) {
        let tex_size = img.tex_size();
        // img.origin = Vector2::with_y(tex_size.y - self.skin_hit_pos);
        img.origin = Vector2::with_y(tex_size.y - self.skin_hit_pos);


        // info!("setting new column origin: {}", img.origin.y);
        let a:u32 = unsafe {std::mem::transmute_copy(&img.origin.y)};
        self.column_origin.store(a, Ordering::Release);

        img.scale = Vector2::ONE * (self.column_width / img.tex_size().x);
    }

}


impl Deref for ManiaPlayfield {
    type Target = ManiaPlayfieldSettings;

    fn deref(&self) -> &Self::Target { 
        &self.settings 
    }
}
