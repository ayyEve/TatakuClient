
use crate::prelude::*;

#[derive(Clone)]
pub struct Animation {
    pub size: Vector2,
    pub depth: f64,
    pub origin: Vector2,


    /// when did the current frame start being drawn?
    /// this will always be related to the delay
    /// 
    /// ie, if the frame was drawn at 100, but the last frame draw time + its delay is only 96, this will be 96, not 100
    /// 
    /// hooray for terrible explanations
    pub frame_start_time: f32,
    pub frames: Vec<Arc<Texture>>,
    pub frame_index: usize,
    pub frame_delays: Vec<f32>,

    context: Option<Context>,
    
    // initial
    pub initial_color: Color,
    pub initial_pos: Vector2,
    pub initial_scale: Vector2,
    pub initial_rotation: f64,

    // current
    pub current_color: Color,
    pub current_pos: Vector2,
    pub current_scale: Vector2,
    pub current_rotation: f64,
}
#[allow(unused)]
impl Animation {
    pub fn new(pos:Vector2, depth:f64, size:Vector2, frames: Vec<Arc<Texture>>, frame_delays: Vec<f32>) -> Self {
        // let scale = Vector2::new(tex.get_width() as f64 / size.x, tex.get_height() as f64 / size.y);
        let tex_size = Vector2::new(frames[0].get_width() as f64, frames[0].get_height() as f64);
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

        Self {
            initial_pos,
            initial_scale,
            initial_rotation,
            initial_color,

            frames,
            frame_index: 0,
            frame_delays,
            frame_start_time: 0.0,

            current_pos,
            current_scale,
            current_rotation,
            current_color,

            size: tex_size,
            depth,
            origin,
            context: None,
        }
    }

    /// time is used to determine which frame we should be on
    pub fn update(&mut self, time: f32) {
        // how long since the current frame has been drawn
        let mut delta_time = time - self.frame_start_time;

        // update index
        loop {
            // how long the current frame should last
            let next_delay = self.frame_delays[self.frame_index];

            // if its time for the next frame
            if delta_time >= next_delay {
                // update the index
                self.frame_index = (self.frame_index + 1) % self.frames.len();
                // subtract from the delta
                delta_time -= next_delay;
                self.frame_start_time = time - delta_time;
            } else {
                // nothing else to do, exit loop
                break;
            }
        }

    }

    pub fn size(&self) -> Vector2 {
        self.size * self.current_scale
    }
    pub fn set_size(&mut self, size: Vector2) {
        let tex_size = self.tex_size();
        let scale = size / tex_size;
        self.initial_scale = scale;
        self.current_scale = scale;
    }

    pub fn tex_size(&self) -> Vector2 {
        let (w, h) = self.frames[0].get_size();
        Vector2::new(w as f64, h as f64)
    }


    pub async fn from_paths<P: AsRef<Path>>(paths: Vec<P>, delays: Vec<f32>, pos:Vector2, depth:f64, size: Vector2) -> TatakuResult<Self> {

        let mut frames = Vec::new();
        for p in paths {
            frames.push(load_texture(p).await?);
        }

        Ok(Self::new(pos, depth, size, frames, delays))
    }
}
impl Renderable for Animation {
    fn get_depth(&self) -> f64 {self.depth}

    fn get_context(&self) -> Option<Context> {self.context}
    fn set_context(&mut self, c:Option<Context>) {self.context = c}
    fn draw(&self, g: &mut GlGraphics, c: Context) {
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

        let image = &self.frames[self.frame_index];
        graphics::Image::new()
            .color(self.current_color.into())
            .draw(image.as_ref(), &self.context.unwrap_or(c).draw_state, transform, g)
    }
}
impl Transformable for Animation {
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
