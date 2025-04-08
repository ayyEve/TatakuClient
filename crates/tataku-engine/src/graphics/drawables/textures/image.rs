use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct Image {
    // pub size: Vector2,
    pub tex: Arc<TextureReference>,
    /// underlying scale of this image, mainly used for 2x res sprites
    pub base_scale: Vector2,

    /// origin of rotation/scale in px, relative to image position
    /// 
    /// BEFORE SCALE
    pub origin: Vector2,

    pub scissor: Scissor,
    pub blend_mode: BlendMode,

    pub color: Color,
    pub pos: Vector2,
    pub scale: Vector2,
    pub rotation: f32,

    pub draw_debug: bool,
}
impl Image {
    pub fn new(
        pos: Vector2, 
        tex: Arc<TextureReference>, 
        base_scale: Vector2
    ) -> Self {
        // let scale = Vector2::new(tex.get_width() as f64 / size.x, tex.get_height() as f64 / size.y);
        let tex_size = Vector2::new(tex.width as f32, tex.height as f32);

        let rotation = 0.0;
        let color = Color::WHITE;

        let origin = tex_size / 2.0;

        Self {
            pos,
            scale: Vector2::ONE,
            rotation,
            color,

            // size: tex_size,
            origin,
            tex,
            scissor: None,
            blend_mode: BlendMode::AlphaBlending,
            base_scale,
            draw_debug: false,
        }
    }

    pub fn size(&self) -> Vector2 {
        self.tex_size() * self.scale
    }
    pub fn set_size(&mut self, size: Vector2) {
        let tex_size = self.tex_size();
        self.scale = size / tex_size;
    }

    fn raw_tex_size(&self) -> Vector2 {
        Vector2::new(self.tex.width as f32, self.tex.height as f32)
    }
    pub fn tex_size(&self) -> Vector2 { 
        self.raw_tex_size() * self.base_scale
    }

    pub fn centered(&mut self) {
        self.origin = self.raw_tex_size() / 2.0;
        self.pos = self.size() / 2.0;
    }

    // NOTE: this will change the origin to top-left
    pub fn fit_to(&mut self, fit: ImageFit, bounds: Bounds) {
        let image_size = self.tex_size();
        let size = bounds.size;

        match fit {
            ImageFit::Fill => {
                self.set_size(bounds.size);
            }
            ImageFit::Contain => {
                // resize to maintain aspect ratio
                let ratio = image_size.y / image_size.x;
                
                let new_size = if image_size.x > image_size.y {
                    // use width as base
                    Vector2::new(
                        size.x, 
                        size.x * ratio
                    )
                } else {
                    // use height as base
                    Vector2::new(
                        size.y * ratio,
                        size.y
                    )
                };

                // transform to Contain

                self.set_size(new_size);
            }
            ImageFit::Cover => {
                // resize to maintain aspect ratio
                let ratio = image_size.y / image_size.x;

                let new_size = if image_size.x > image_size.y {
                    // use width as base
                    Vector2::new(
                        size.x, 
                        size.x * ratio
                    )
                } else {
                    // use height as base
                    Vector2::new(
                        size.y * ratio,
                        size.y
                    )
                };
                
                // TODO: transform to Cover
                self.set_size(new_size);
            }
            ImageFit::None => {},
        }
    }

    pub fn fit_to_bg_size(&mut self, size: Vector2) {
        self.fit_to(ImageFit::Contain, Bounds::new(Vector2::ZERO, size));
        self.origin = Vector2::ZERO;
        self.pos = (size - self.size()) / 2.0;
    }

    pub fn reference_count(&self) -> usize {
        Arc::strong_count(&self.tex)
    }
}

impl TatakuRenderable for Image {
    fn get_name(&self) -> String { "Texture".to_owned() }
    fn get_bounds(&self) -> Bounds { Bounds::new(self.pos, self.size()) }
    
    fn get_scissor(&self) -> Scissor { self.scissor }
    fn set_scissor(&mut self, s: Scissor) { self.scissor = s }
    fn get_blend_mode(&self) -> BlendMode { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: BlendMode) { self.blend_mode = blend_mode }

    fn draw(
        &self, 
        options: &DrawOptions, 
        mut transform: Matrix, 
        g: &mut dyn GraphicsEngine
    ) {
        let color = options.color_with_alpha(self.color);

        let h_flip = false;
        let v_flip = false;

        // if scale.x < 0.0 {
        //     scale.x = scale.x.abs();
        //     h_flip = true;
        // }
        // if scale.y < 0.0 {
        //     scale.y = scale.y.abs();
        //     v_flip = true;
        // }

        transform = transform * Matrix::identity()
            .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate
            .scale(self.scale * self.base_scale) // scale
            .trans(self.pos) // move to pos
        ;

        g.draw_tex(
            &self.tex, 
            color,
            h_flip, 
            v_flip, 
            transform, 
            self.blend_mode
        );

        // if self.draw_debug {
        //     if alpha < 0.4 {
        //         println!("low alpha!!")
        //     }
        //     let size = self.size();

        //     g.draw_rect(
        //         [self.pos.x, self.pos.y, size.x, size.y], 
        //         Some(Border::new(Color::RED, 5.0)), 
        //         Shape::Square, 
        //         Color::TRANSPARENT_WHITE, 
        //         transform, 
        //         BlendMode::AlphaBlending
        //     )
        // }
    }
}



#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ImageFit {
    /// The image is resized to fill the given dimension. 
    /// 
    /// If necessary, the image will be stretched or squished to fit
    #[default] Fill,
    
    /// The image keeps its aspect ratio, but is resized to fit within the given dimension
    Contain,

    /// The image keeps its aspect ratio and fills the given dimension. The image will be clipped to fit
    Cover,

    /// The image is not resized
    None,
}
