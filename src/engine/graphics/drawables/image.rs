use crate::prelude::*;

#[derive(Clone)]
pub struct Image {
    // pub size: Vector2,
    pub depth: f64,
    pub tex: Arc<Texture>,
    /// underlying scale of this image, mainly used for 2x res sprites
    pub base_scale: Vector2,

    // origin of rotation in px, relative to image position
    pub origin: Vector2,

    draw_state: Option<DrawState>,

    pub color: Color,
    pub pos: Vector2,
    pub scale: Vector2,
    pub rotation: f64,
}
impl Image {
    pub fn new(pos:Vector2, depth:f64, tex:Arc<Texture>, base_scale: Vector2) -> Image {
        // let scale = Vector2::new(tex.get_width() as f64 / size.x, tex.get_height() as f64 / size.y);
        let tex_size = Vector2::new(tex.get_width() as f64, tex.get_height() as f64);

        let rotation = 0.0;
        let color = Color::WHITE;

        let origin = tex_size / 2.0;

        Image {
            pos,
            scale: Vector2::ONE,
            rotation,
            color,

            // size: tex_size,
            depth,
            origin,
            tex,
            draw_state: None,
            base_scale
        }
    }

    pub fn size(&self) -> Vector2 {
        self.tex_size() * self.scale
    }
    pub fn set_size(&mut self, size: Vector2) {
        let tex_size = self.tex_size();
        self.scale = size / tex_size;
    }

    pub fn tex_size(&self) -> Vector2 {
        let (w, h) = self.tex.get_size();
        Vector2::new(w as f64, h as f64) * self.base_scale
    }


    // pub async fn from_path<P: AsRef<Path>>(path: P, pos:Vector2, depth:f64, size: Vector2) -> TatakuResult<Self> {
    //     match load_texture(path).await {
    //         Ok(tex) => Ok(Self::new(pos, depth, tex, size)),
    //         Err(e) => return Err(e),
    //     }
    // }
}
impl Renderable for Image {
    fn get_name(&self) -> String { format!("Texture with id {}", self.tex.get_id()) }
    fn get_depth(&self) -> f64 { self.depth }

    fn get_draw_state(&self) -> Option<DrawState> { self.draw_state }
    fn set_draw_state(&mut self, c:Option<DrawState>) { self.draw_state = c }
    fn draw(&self, g: &mut GlGraphics, c: Context) {
        self.draw_with_transparency(c, self.color.a, 0.0, g)
    }
}

impl TatakuRenderable for Image {
    fn draw_with_transparency(&self, c: Context, alpha: f32, _: f32, g: &mut GlGraphics) {
        let transform = c.transform
            // move to pos
            .trans_pos(self.pos)

            // scale to size
            .scale_pos(self.scale * self.base_scale)

            // rotate to rotate
            .rot_rad(self.rotation)
            
            // apply origin
            .trans_pos(-self.origin)
        ;

        graphics::Image::new()
        .color(self.color.alpha(alpha).into())
        .draw(
            self.tex.as_ref(), 
            &self.draw_state.unwrap_or(c.draw_state), 
            transform, 
            g
        );
    }
}
