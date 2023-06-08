use crate::prelude::*;

#[derive(Clone)]
pub struct Animation {
    pub size: Vector2,
    pub depth: f32,
    pub origin: Vector2,
    pub base_scale: Vector2,

    /// when did the current frame start being drawn?
    /// this will always be related to the delay
    /// 
    /// ie, if the frame was drawn at 100, but the last frame draw time + its delay is only 96, this will be 96, not 100
    /// 
    /// hooray for terrible explanations
    pub frame_start_time: f32,
    pub frames: Vec<TextureReference>,
    pub frame_index: usize,
    pub frame_delays: Vec<f32>,

    scissor: Scissor,

    // current
    pub color: Color,
    pub pos: Vector2,
    pub scale: Vector2,
    pub rotation: f32,
}
#[allow(unused)]
impl Animation {
    pub fn new(pos:Vector2, depth:f32, size:Vector2, frames: Vec<TextureReference>, frame_delays: Vec<f32>, base_scale: Vector2) -> Self {
        // let scale = Vector2::new(tex.get_width() as f64 / size.x, tex.get_height() as f64 / size.y);
        let tex_size = Vector2::new(frames[0].width as f32, frames[0].height as f32);
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
            scissor: None,

            frames,
            frame_index: 0,
            frame_delays,
            frame_start_time: 0.0,

            size: tex_size,
            depth,
            origin,
            // draw_state: None,
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
        Vector2::new(self.frames[0].width as f32, self.frames[0].height as f32) * self.base_scale
    }


    pub async fn from_paths<P: AsRef<Path>>(paths: Vec<P>, delays: Vec<f32>, pos:Vector2, depth:f32, size: Vector2) -> TatakuResult<Self> {
        let mut frames = Vec::new();
        for p in paths {
            frames.push(GameWindow::load_texture(p).await?);
        }

        Ok(Self::new(pos, depth, size, frames, delays, Vector2::ONE))
    }
}
impl TatakuRenderable for Animation {
    fn get_depth(&self) -> f32 {self.depth}

    fn get_scissor(&self) -> Scissor {self.scissor}
    fn set_scissor(&mut self, s:Scissor) {self.scissor = s}

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        self.draw_with_transparency(self.color.a, 0.0, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, _: f32, transform: Matrix, g: &mut GraphicsState) {
        let mut scale = self.scale;
        let mut h_flip = false;
        let mut v_flip = false;

        if scale.x < 0.0 {
            scale.x = scale.x.abs();
            h_flip = true;
        }
        if scale.y < 0.0 {
            scale.y = scale.y.abs();
            v_flip = true;
        }

        let transform = transform
            // apply origin
            .trans(-self.origin)

            // rotate to rotate
            .rot(self.rotation)

            // scale to size
            .scale(scale * self.base_scale)

            // move to pos
            .trans(self.pos)
        ;

        let image = &self.frames[self.frame_index];
        g.draw_tex(image, self.depth, self.color.alpha(alpha), h_flip, v_flip, transform, self.scissor);
    }
}
