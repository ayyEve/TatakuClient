use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default)]
pub enum ImageFlip {
    #[default]
    None,
    Horizontal,
    Vertical,
    Both,
}
impl ImageFlip {
    pub fn new(
        flip_h: bool,
        flip_v: bool,
    ) -> Self {
        match (flip_h, flip_v) {
            (true, true) => Self::Both,
            (true, false) => Self::Horizontal,
            (false, true) => Self::Vertical,
            (false, false) => Self::None,
        }
    }
    
    fn flip_h(self) -> bool {
        matches!(self, Self::Horizontal | Self::Both)
    }
    fn flip_v(self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }
    pub fn xor(self, other: Self) -> Self {
        Self::new(
            self.flip_h() ^ other.flip_h(),
            self.flip_v() ^ other.flip_v(),
        )
    }
}



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
    pub blend_mode: Pipeline,

    pub color: Color,
    pub pos: Vector2,
    pub scale: Vector2,
    pub rotation: f32,

    pub flip: ImageFlip,
    pub draw_debug: bool,
}
impl Image {
    pub fn new(
        pos: Vector2, 
        tex: Arc<TextureReference>, 
        base_scale: Vector2
    ) -> Self {
        let tex_size = Vector2::new(tex.width as f32, tex.height as f32);
        let origin = tex_size / 2.0;

        Self {
            pos,
            scale: Vector2::ONE,
            rotation: 0.0,
            color: Color::WHITE,
            origin,
            tex,
            scissor: None,
            flip: ImageFlip::None,
            blend_mode: Pipeline::AlphaBlending,
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
        Vector2::new(
            self.tex.width as f32, 
            self.tex.height as f32,
        )
    }
    pub fn tex_size(&self) -> Vector2 { 
        self.raw_tex_size() * self.base_scale
    }

    pub fn centered(&mut self) {
        self.origin = self.raw_tex_size() / 2.0;
        self.pos = self.size() / 2.0;
    }

    /// NOTE: this will change the origin to top-left
    pub fn fit_to(&mut self, fit: ImageStretch, bounds: Bounds) {
        let image_size = self.tex_size();
        let size = bounds.size;

        match fit {
            ImageStretch::Fill => {
                self.set_size(bounds.size);
            }
            ImageStretch::Contain => {
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

                // TODO: transform to Contain

                self.set_size(new_size);
            }
            ImageStretch::Cover => {
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
            ImageStretch::None => {},
        }
    }

    pub fn fit_to_bg_size(&mut self, size: Vector2) {
        self.fit_to(ImageStretch::Contain, Bounds::new(Vector2::ZERO, size));
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
    fn get_blend_mode(&self) -> Pipeline { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: Pipeline) { self.blend_mode = blend_mode }

    fn draw(
        &self, 
        options: &DrawOptions, 
        mut transform: Matrix, 
        g: &mut dyn GraphicsEngine,
    ) {
        let color = options.color_with_alpha(self.color);

        // let h_flip = false;
        // let v_flip = false;

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

        let flip = self.flip.xor(options.image_flip);
        g.draw_tex(
            &self.tex, 
            color,
            flip.flip_h(), 
            flip.flip_v(), 
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
pub enum ImageStretch {
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
