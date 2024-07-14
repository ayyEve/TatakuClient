use crate::prelude::*;

pub trait TatakuRenderable: Sync + Send {
    fn get_name(&self) -> String { "Unnamed".to_owned() }
    fn get_bounds(&self) -> Bounds;
    
    fn get_scissor(&self) -> Scissor { None }
    fn set_scissor(&mut self, _c: Scissor) {}

    fn get_blend_mode(&self) -> BlendMode;
    fn set_blend_mode(&mut self, blend_mode: BlendMode);
    fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self where Self:Sized { self.set_blend_mode(blend_mode); self }

    // fn draw(&self, transform: Matrix, g: &mut dyn GraphicsEngine);
    fn draw(
        &self, 
        options: &DrawOptions,
        transform: Matrix, 
        g: &mut dyn GraphicsEngine,
    );
}


/// draw option overrides
#[derive(Copy, Clone, Debug, Default)]
pub struct DrawOptions {
    pub alpha: Option<f32>,
    pub border_alpha: Option<f32>,

    pub color: Option<Color>,
    pub border_color: Option<Color>,
}
impl DrawOptions {
    /// get the modified alpha value for the provided alpha
    pub fn alpha(&self, other: f32) -> f32 {
        self.alpha.unwrap_or(1.0) * other
    }
    /// get the modified alpha value for the provided border alpha
    pub fn border_alpha(&self, other: f32) -> f32 {
        self.alpha.unwrap_or(1.0) * other
    }

    /// get the modified color value for the provided color
    /// (this really just returns our color or the provided color if we dont have one)
    pub fn color(&self, other: Color) -> Color {
        self.color.unwrap_or(other)
    }

    /// get the modified color with the modified alpha for the provided color
    pub fn color_with_alpha(&self, other: Color) -> Color {
        self.color(other).alpha(self.alpha(other.a))
    }

    
    /// get the modified color value for the provided border color
    /// (this really just returns our color or the provided color if we dont have one)
    pub fn border_color(&self, other: Color) -> Color {
        self.border_color.unwrap_or(other)
    }

    /// get the modified color with the modified alpha for the provided border color
    pub fn border_color_with_alpha(&self, other: Color) -> Color {
        self.border_color(other).alpha(self.border_alpha(other.a))
    }


    /// merge self with other
    /// color and border color will be whichever is Some(), or other's if both are Some()
    pub fn merge(self, other: Self) -> Self {
        Self {
            alpha: merge_opts(self.alpha, other.alpha),
            border_alpha: merge_opts(self.border_alpha, other.border_alpha),
            color: other.color.or(self.color),
            border_color: other.border_color.or(self.border_color),
        }
    }
}

fn merge_opts(a: Option<f32>, b: Option<f32>) -> Option<f32> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a * b),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}