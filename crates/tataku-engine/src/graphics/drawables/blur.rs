use crate::prelude::*;

pub struct Blur {
    bounds: Bounds,
    blur_type: BlurType,
}
impl Blur {
    pub fn new(
        bounds: Bounds, 
        blur_type: BlurType
    ) -> Self { 
        Self { 
            bounds, 
            blur_type 
        }
    }
}
impl TatakuRenderable for Blur {
    fn get_bounds(&self) -> Bounds { self.bounds }

    fn get_blend_mode(&self) -> Pipeline {
        match self.blur_type {
            BlurType::Gaussian { .. } => Pipeline::GaussianBlur,
            BlurType::Box { .. } => Pipeline::BoxBlur,
        }
    }
    fn set_blend_mode(&mut self, _blend_mode: Pipeline) {}

    fn draw(
        &self, 
        _options: &DrawOptions,
        _transform: Matrix, 
        g: &mut dyn GraphicsEngine,
    ) {
        match self.blur_type {
            BlurType::Gaussian { sigma } 
                => g.draw_gaussian_blur(self.bounds, sigma, 1),
            BlurType::Box { size } 
                => g.draw_box_blur(self.bounds, size),
        }

    }
}

#[derive(Copy, Clone, Debug)]
pub enum BlurType {
    Gaussian {
        sigma: f32,
    },
    Box {
        size: u32,
    },
}
