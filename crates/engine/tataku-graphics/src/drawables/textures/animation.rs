use crate::*;

#[derive(Clone)]
pub struct Animation {
    pub base_scale: f32,
    pub max_size: Vector2,

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

    // current
    pub color: Color,

    pub draw_debug: bool,
}
impl Animation {
    pub fn new(
        frames: Vec<Arc<TextureReference>>, 
        frame_delay: f32, 
        base_scale: f32,
    ) -> Self {
        let max_width = frames.iter()
            .map(|tex| tex.width)
            .max()
            .unwrap_or_default();

        let max_height = frames.iter()
            .map(|tex| tex.height)
            .max()
            .unwrap_or_default();

        let max_size = Vector2::new(
            max_width as f32,
            max_height as f32,
        );

        Self {
            color: Color::WHITE,
            base_scale,
            max_size,

            frames,
            frame_index: 0,
            frame_delay,
            frame_start_time: 0.0,

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
        self.max_size * self.base_scale
    }

    pub fn set_start_time(&mut self, time: f32) {
        self.frame_start_time = time;
    }

    pub fn current_frame_as_image(&self) -> Image {
        Image {
            tex: self.frames[self.frame_index].clone(),
            base_scale: self.base_scale,
            color: self.color,
            draw_debug: self.draw_debug,
            flip: ImageFlip::None,
        }
    }

}

#[cfg(feature="graphics")]
impl TatakuRenderable for Animation {
    fn get_name(&self) -> String { "animation".into() }

    fn draw(
        &self, 
        options: &DrawOptions, 
        transform: Matrix, 
        g: &mut dyn DrawEngine
    ) {
        let color = options.color_with_alpha(self.color);

        let Some(blend_mode) = options.blend_mode() else { return; };

        g.draw_tex(
            TextureDraw::new(
                &self.frames[self.frame_index],
                color, 
            ),
            transform, 
            blend_mode
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
