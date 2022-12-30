use crate::prelude::*;

#[derive(Clone)]
pub struct Animation {
    pub size: Vector2,
    pub depth: f64,
    pub origin: Vector2,
    pub base_scale: Vector2,

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

    draw_state: Option<DrawState>,

    // current
    pub color: Color,
    pub pos: Vector2,
    pub scale: Vector2,
    pub rotation: f64,
}
#[allow(unused)]
impl Animation {
    pub fn new(pos:Vector2, depth:f64, size:Vector2, frames: Vec<Arc<Texture>>, frame_delays: Vec<f32>, base_scale: Vector2) -> Self {
        // let scale = Vector2::new(tex.get_width() as f64 / size.x, tex.get_height() as f64 / size.y);
        let tex_size = Vector2::new(frames[0].get_width() as f64, frames[0].get_height() as f64);
        let scale = size / tex_size;

        let rotation = 0.0;
        let color = Color::WHITE;

        let origin = tex_size / 2.0;

        Self {
            pos,
            scale,
            rotation,
            color,
            base_scale,

            frames,
            frame_index: 0,
            frame_delays,
            frame_start_time: 0.0,

            size: tex_size,
            depth,
            origin,
            draw_state: None,
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
        self.size * self.scale
    }
    pub fn set_size(&mut self, size: Vector2) {
        let tex_size = self.tex_size();
        let scale = size / tex_size;
        self.scale = scale;
    }

    pub fn tex_size(&self) -> Vector2 {
        let (w, h) = self.frames[0].get_size();
        Vector2::new(w as f64, h as f64) * self.base_scale
    }


    pub async fn from_paths<P: AsRef<Path>>(paths: Vec<P>, delays: Vec<f32>, pos:Vector2, depth:f64, size: Vector2) -> TatakuResult<Self> {
        let mut frames = Vec::new();
        for p in paths {
            frames.push(load_texture(p).await?);
        }

        Ok(Self::new(pos, depth, size, frames, delays, Vector2::ONE))
    }
}
impl Renderable for Animation {
    fn get_depth(&self) -> f64 {self.depth}

    fn get_draw_state(&self) -> Option<DrawState> {self.draw_state}
    fn set_draw_state(&mut self, c:Option<DrawState>) {self.draw_state = c}
    fn draw(&self, g: &mut GlGraphics, c: Context) {
        self.draw_with_transparency(c, self.color.a, 0.0, g)
    }
}

impl TatakuRenderable for Animation {
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

        let image = &self.frames[self.frame_index];
        graphics::Image::new()
            .color(self.color.alpha(alpha).into())
            .draw(image.as_ref(), &self.draw_state.unwrap_or(c.draw_state), transform, g)
    }
}
