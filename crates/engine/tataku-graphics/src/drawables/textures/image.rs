use crate::*;
use common::reflect::*;

#[derive(Copy, Clone, Debug, Default)]
pub enum ImageFlip {
    #[default]
    None,
    Horizontal,
    Vertical,
    Both,
}
impl ImageFlip {
    pub const fn new(
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
    
    pub const fn flip_h(self) -> bool {
        matches!(self, Self::Horizontal | Self::Both)
    }
    pub const fn flip_v(self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }
    pub const fn xor(self, other: Self) -> Self {
        Self::new(
            self.flip_h() ^ other.flip_h(),
            self.flip_v() ^ other.flip_v(),
        )
    }
}
impl std::ops::BitXor for ImageFlip {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.xor(rhs)
    }
}
impl std::ops::BitXorAssign for ImageFlip {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.xor(rhs);
    }
}



#[derive(Clone, Debug)]
pub struct Image {
    // pub size: Vector2,
    pub tex: Arc<TextureReference>,
    /// underlying scale of this image, mainly used for 2x res sprites
    pub base_scale: f32,

    pub blend_mode: BlendMode,

    pub color: Color,

    pub flip: ImageFlip,
    pub draw_debug: bool,
}
impl Image {
    pub fn new(
        tex: Arc<TextureReference>, 
        base_scale: f32
    ) -> Self {
        Self {
            color: Color::WHITE,
            tex,
            flip: ImageFlip::None,
            blend_mode: BlendMode::AlphaBlending,
            base_scale,
            draw_debug: false,
        }
    }

    pub fn tex_size(&self) -> Vector2 {
        Vector2::new(
            self.tex.width as f32, 
            self.tex.height as f32,
        )
    }
    pub fn size(&self) -> Vector2 {
        self.tex_size() * self.base_scale
    }

    pub fn reference_count(&self) -> usize {
        Arc::strong_count(&self.tex)
    }
}

#[cfg(feature="graphics")]
impl TatakuRenderable for Image {
    fn get_name(&self) -> String { "Texture".to_owned() }
    
    fn get_pipeline(&self) -> GraphicsPipeline { GraphicsPipeline::Standard(self.blend_mode) }
    fn set_pipeline(&mut self, pipeline: GraphicsPipeline) { 
        let GraphicsPipeline::Standard(blend_mode) = pipeline 
        else { return };

        self.blend_mode = blend_mode; 
    }

    fn draw(
        &self, 
        options: &DrawOptions, 
        transform: Matrix,
        g: &mut dyn DrawEngine,
    ) {
        let color = options.color_with_alpha(self.color);

        let flip = self.flip.xor(options.image_flip);
        g.draw_tex(
            TextureDraw::new(
                &self.tex, 
                color,
            ).with_flip(flip),
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




#[derive(common::macros::Reflect)]
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

impl ImageStretch {
    /// Returns the scale needed to fit an image in the container.
    pub fn fit_to(self, image_size: Vector2, container_size: Vector2) -> Vector2 {
        match self {
            ImageStretch::Fill => {
                container_size / image_size
            }
            ImageStretch::Contain => {
                // Scale along longest axis
                let new_size = if image_size.x > image_size.y {
                    // use container width
                    Vector2::new(
                        container_size.x,
                        container_size.x / image_size.x * image_size.y,
                    )
                } else {
                    // use container height
                    Vector2::new(
                        container_size.y / image_size.y * image_size.x,
                        container_size.y,
                    )
                };

                new_size / image_size
            }
            ImageStretch::Cover => {
                // Scale along shortest axis
                let new_size = if image_size.x < image_size.y {
                    // use container width
                    Vector2::new(
                        container_size.x,
                        container_size.x / image_size.x * image_size.y,
                    )
                } else {
                    // use container height
                    Vector2::new(
                        container_size.y / image_size.y * image_size.x,
                        container_size.y,
                    )
                };

                new_size / image_size
            }
            ImageStretch::None => Vector2::ONE,
        }
    }
}
