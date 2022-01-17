use crate::prelude::*;
use super::prelude::*;
use opengl_graphics::{TextureSettings};

#[derive(Clone)]
pub struct Image {
    size: Vector2,
    pub depth: f64,
    pub tex: Arc<Texture>,

    // rotation of origin, relative to image size
    pub origin: Vector2,

    
    // initial
    pub initial_pos: Vector2,
    pub initial_scale: Vector2,
    pub initial_rotation: f64,
    context: Option<Context>,

    // current
    pub current_pos: Vector2,
    pub current_scale: Vector2,
    pub current_rotation: f64,
    
    spawn_time:u64,
}
impl Image {
    pub fn new(pos:Vector2, depth:f64, tex:Texture, size:Vector2) -> Image {
        // let scale = Vector2::new(tex.get_width() as f64 / size.x, tex.get_height() as f64 / size.y);
        let tex_size = Vector2::new(tex.get_width() as f64, tex.get_height() as f64);
        let scale = size / tex_size;

        let initial_pos = pos;
        let initial_scale = scale;

        let current_pos = pos;
        let current_scale = scale;

        let initial_rotation = 0.0;
        let current_rotation = 0.0;

        let origin = tex_size / 2.0;

        Image {
            initial_pos,
            initial_scale,
            initial_rotation,

            current_pos,
            current_scale,
            current_rotation,

            size,
            depth,
            origin,
            tex: Arc::new(tex),
            spawn_time: 0,
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

    pub fn from_path<P: AsRef<Path>>(path: P, pos:Vector2, depth:f64, size: Vector2) -> TatakuResult<Self> {
        let settings = TextureSettings::new();
        let tex = Texture::from_path(path, &settings)?;
        Ok(Self::new(pos, depth, tex, size))
    }
}
impl Renderable for Image {
    fn get_lifetime(&self) -> u64 {0}
    fn set_lifetime(&mut self, _lifetime:u64) {}
    fn get_spawn_time(&self) -> u64 {self.spawn_time}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}
    fn get_depth(&self) -> f64 {self.depth}
    fn get_context(&self) -> Option<Context> {self.context}
    fn set_context(&mut self, c:Option<Context>) {self.context = c}
    fn draw(&mut self, g: &mut GlGraphics, c: Context) {

        let pre_rotation = self.current_pos / self.current_scale;

        let transform = c
            .transform
            // scale to size
            .scale(self.current_scale.x, self.current_scale.y)

            // move to pos
            .trans(pre_rotation.x, pre_rotation.y)

            // rotate to rotate
            .rot_rad(self.current_rotation)
            
            // apply origin
            .trans(-self.origin.x, -self.origin.y)
        ;

        graphics::image(
            self.tex.as_ref(), 
            transform, 
            g
        );
    }
}
impl Transformable for Image {
    fn apply_transform(&mut self, transform: &Transformation, val: TransformValueResult) {
        match transform.trans_type {
            TransformType::Position { .. } => {
                let val:Vector2 = val.into();
                // println!("val: {:?}", val);
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
            
            //TODO!
            TransformType::Transparency { .. } => {
                // let val:f64 = val.into();
                // self.current_color = self.current_color.alpha(val.clamp(0.0, 1.0) as f32);
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
        self.current_scale.x != 0.0 && self.current_scale.y != 0.0
    }

}