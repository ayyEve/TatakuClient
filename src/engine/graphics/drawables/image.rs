use crate::prelude::*;

#[derive(Clone)]
pub struct Image {
    pub size: Vector2,
    pub depth: f64,
    pub tex: Arc<Texture>,

    // rotation of origin, relative to image size
    pub origin: Vector2,

    
    // initial
    pub initial_color: Color,
    pub initial_pos: Vector2,
    pub initial_scale: Vector2,
    pub initial_rotation: f64,
    context: Option<Context>,

    // current
    pub current_color: Color,
    pub current_pos: Vector2,
    pub current_scale: Vector2,
    pub current_rotation: f64,
}
impl Image {
    pub fn new(pos:Vector2, depth:f64, tex:Arc<Texture>, size:Vector2) -> Image {
        // let scale = Vector2::new(tex.get_width() as f64 / size.x, tex.get_height() as f64 / size.y);
        let tex_size = Vector2::new(tex.get_width() as f64, tex.get_height() as f64);
        let scale = size / tex_size;

        let initial_pos = pos;
        let initial_rotation = 0.0;
        let initial_scale = scale;
        let initial_color = Color::WHITE;

        let current_pos = pos;
        let current_rotation = 0.0;
        let current_scale = scale;
        let current_color = Color::WHITE;

        let origin = tex_size / 2.0;

        Image {
            initial_pos,
            initial_scale,
            initial_rotation,
            initial_color,

            current_pos,
            current_scale,
            current_rotation,
            current_color,

            size: tex_size,
            depth,
            origin,
            tex,
            context: None,
        }
    }

    pub fn size(&self) -> Vector2 {
        self.size * self.current_scale
    }
    pub fn set_size(&mut self, size: Vector2) {
        let tex_size = Vector2::new(
            self.tex.get_width() as f64, 
            self.tex.get_height() as f64
        );
        let scale = size / tex_size;
        self.initial_scale = scale;
        self.current_scale = scale;
    }

    pub fn tex_size(&self) -> Vector2 {
        let (w, h) = self.tex.get_size();
        Vector2::new(w as f64, h as f64)
    }


    pub async fn from_path<P: AsRef<Path>>(path: P, pos:Vector2, depth:f64, size: Vector2) -> TatakuResult<Self> {
        match load_texture(path).await {
            Ok(tex) => Ok(Self::new(pos, depth, tex, size)),
            Err(e) => return Err(e),
        }
    }
}
impl Renderable for Image {
    fn get_name(&self) -> String { format!("Texture with id {}", self.tex.get_id()) }
    fn get_depth(&self) -> f64 { self.depth }

    fn get_context(&self) -> Option<Context> { self.context }
    fn set_context(&mut self, c:Option<Context>) { self.context = c }
    fn draw(&self, g: &mut GlGraphics, c: Context) {
        let transform = c.transform
            // move to pos
            .trans_pos(self.current_pos)

            // scale to size
            .scale_pos(self.current_scale)

            // rotate to rotate
            .rot_rad(self.current_rotation)
            
            // apply origin
            .trans_pos(-self.origin)
        ;

        graphics::Image::new()
            .color(self.current_color.into())
            .draw(self.tex.as_ref(), &self.context.unwrap_or(c).draw_state, transform, g);
    }
}
impl Transformable for Image {
    fn apply_transform(&mut self, transform: &Transformation, val: TransformValueResult) {
        match transform.trans_type {
            TransformType::Position { .. } => {
                let val:Vector2 = val.into();
                // trace!("val: {:?}", val);
                self.current_pos = self.initial_pos + val;
            },
            TransformType::Scale { .. } => {
                let val:f64 = val.into();
                self.current_scale = self.initial_scale + val;
            },
            TransformType::Rotation { .. } => {
                let val:f64 = val.into();
                self.current_rotation = self.current_rotation + val;
            }
            
            TransformType::Transparency { .. } => {
                let val:f64 = val.into();
                self.current_color = self.current_color.alpha(val.clamp(0.0, 1.0) as f32);
            },

            // no color, ignore
            TransformType::Color { .. } => {},

            // this doesnt have a border, ignore
            TransformType::BorderTransparency { .. } => {},
            TransformType::BorderSize { .. } => {},
            TransformType::BorderColor { .. } => {},

            TransformType::None => {},
        }
    }
    
    fn visible(&self) -> bool {
        self.current_scale.x != 0.0 && self.current_scale.y != 0.0 && self.current_color.a != 0.0
    }

}