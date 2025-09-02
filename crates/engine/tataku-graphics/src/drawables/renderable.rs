use crate::prelude::*;

pub trait TatakuRenderable: Sync + Send {
    fn get_name(&self) -> String { "Unnamed".to_owned() }
    
    fn get_blend_mode(&self) -> GraphicsPipeline;
    fn set_blend_mode(&mut self, blend_mode: GraphicsPipeline);
    fn with_blend_mode(mut self, blend_mode: GraphicsPipeline) -> Self where Self:Sized { 
        self.set_blend_mode(blend_mode); 
        self 
    }

    #[cfg(feature="graphics")]
    fn draw(
        &self, 
        options: &DrawOptions,
        transform: Matrix, 
        g: &mut dyn DrawEngine,
    );
}


/// draw option overrides
#[derive(Copy, Clone, Debug, Default)]
pub struct DrawOptions {
    pub alpha: Option<u8>,
    pub border_alpha: Option<u8>,

    pub color: Option<Color>,
    pub border_color: Option<Color>,

    pub image_flip: ImageFlip,
}
impl DrawOptions {
    fn apply_alpha(alpha: Option<u8>, other: u8) -> u8 {
        Color::to_u8((
            Color::to_f32(alpha.unwrap_or(Color::MAX)) 
            * Color::to_f32(other)
        ).clamp(0.0, 1.0))
    }

    /// get the modified alpha value for the provided alpha
    pub fn alpha(&self, other: u8) -> u8 {
        Self::apply_alpha(self.alpha, other)
    }
    /// get the modified alpha value for the provided border alpha
    pub fn border_alpha(&self, other: u8) -> u8 {
        let b = Self::apply_alpha(self.border_alpha, other);
        let a = self.alpha.unwrap_or(Color::MAX);

        Color::to_u8(Color::to_f32(b) * Color::to_f32(a))
    }

    /// get the modified color value for the provided color
    /// (this really just returns our color or the provided color if we dont have one)
    pub fn color(&self, other: Color) -> Color {
        self.color.unwrap_or(other)
    }

    /// get the modified color with the modified alpha for the provided color
    pub fn color_with_alpha(&self, other: Color) -> Color {
        self.color(other).alpha8(self.alpha(other.a))
    }

    
    /// get the modified color value for the provided border color
    /// (this really just returns our color or the provided color if we dont have one)
    pub fn border_color(&self, other: Color) -> Color {
        self.border_color.unwrap_or(other)
    }

    /// get the modified color with the modified alpha for the provided border color
    pub fn border_color_with_alpha(&self, other: Color) -> Color {
        self.border_color(other).alpha8(self.border_alpha(other.a))
    }


    /// Merge self with other
    /// 
    /// Color and border color will be whichever is Some(), or other's if both are Some()
    /// 
    /// Will also xor the image flips
    pub fn merge(self, other: Self) -> Self {
        Self {
            alpha: merge_opts(self.alpha, other.alpha),
            border_alpha: merge_opts(self.border_alpha, other.border_alpha),
            color: other.color.or(self.color),
            border_color: other.border_color.or(self.border_color),
            image_flip: self.image_flip.xor(other.image_flip),
        }
    }
}

fn merge_opts<T: std::ops::Mul<Output=T>>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a * b),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}
