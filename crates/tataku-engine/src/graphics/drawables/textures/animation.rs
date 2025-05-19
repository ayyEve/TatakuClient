use crate::prelude::*;

#[derive(Clone)]
pub struct Animation {
    pub size: Vector2,
    pub origin: Vector2,
    pub base_scale: Vector2,

    /// when did the current frame start being drawn?
    /// this will always be related to the delay
    /// 
    /// ie, if the frame was drawn at 100, but the last frame draw time + its delay is only 96, this will be 96, not 100
    /// 
    /// hooray for terrible explanations
    pub frame_start_time: f32,
    pub frames: Vec<Arc<TextureReference>>,
    pub frame_index: usize,
    pub frame_delay: f32,

    scissor: Scissor,
    blend_mode: BlendMode,

    // current
    pub color: Color,
    pub pos: Vector2,
    pub scale: Vector2,
    pub rotation: f32, 

    pub draw_debug: bool,
}
impl Animation {
    pub fn new(
        pos: Vector2, 
        size: Vector2, 
        frames: Vec<Arc<TextureReference>>, 
        frame_delay: f32, 
        base_scale: Vector2,
    ) -> Self {
        // let scale = Vector2::new(tex.get_width() as f64 / size.x, tex.get_height() as f64 / size.y);
        let tex_size = Vector2::new(
            frames[0].width as f32, 
            frames[0].height as f32
        );
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
            blend_mode: BlendMode::AlphaBlending,

            frames,
            frame_index: 0,
            frame_delay,
            frame_start_time: 0.0,

            size: tex_size,
            origin,
            draw_debug: false,
        }
    }

    /// time is used to determine which frame we should be on
    pub fn update(&mut self, time: f32) {
        // how long since the current frame has been drawn
        let mut delta_time = time - self.frame_start_time;

        // update index
        loop {
            // if its time for the next frame
            if delta_time >= self.frame_delay {
                // update the index
                self.frame_index = (self.frame_index + 1) % self.frames.len();
                // subtract from the delta
                delta_time -= self.frame_delay;
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

    pub fn set_start_time(&mut self, time: f32) {
        self.frame_start_time = time;
    }

    pub fn current_frame_as_image(&self) -> Image {
        Image {
            tex: self.frames[self.frame_index].clone(),
            base_scale: self.base_scale,
            origin: self.origin,
            scissor: self.scissor,
            blend_mode: self.blend_mode,
            color: self.color,
            pos: self.pos,
            scale: self.scale,
            rotation: self.rotation,
            draw_debug: self.draw_debug,
            flip: ImageFlip::None,
        }
    }

}

impl TatakuRenderable for Animation {
    fn get_name(&self) -> String { "animation".into() }
    fn get_bounds(&self) -> Bounds { Bounds::new(self.pos, self.size()) }

    fn get_scissor(&self) -> Scissor { self.scissor }
    fn set_scissor(&mut self, s: Scissor) { self.scissor = s }
    fn get_blend_mode(&self) -> BlendMode { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: BlendMode) { self.blend_mode = blend_mode }

    fn draw(
        &self, 
        options: &DrawOptions, 
        transform: Matrix, 
        g: &mut dyn GraphicsEngine
    ) {
        let color = options.color_with_alpha(self.color);

        let transform = transform
            .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate
            .scale(self.scale * self.base_scale) // scale
            .trans(self.pos) // move to pos
        ;

        g.draw_tex(
            &self.frames[self.frame_index], 
            color, 
            false, 
            false, 
            transform, 
            self.blend_mode
        );

        // if self.draw_debug {
        //     let size = self.size();

        //     g.draw_rect(
        //         [ self.pos.x, self.pos.y, size.x, size.y ], 
        //         Some(Border::new(Color::CYAN, 5.0)), 
        //         Shape::Square, 
        //         Color::TRANSPARENT, 
        //         transform, 
        //         BlendMode::AlphaBlending
        //     )
        // }
    }
}
