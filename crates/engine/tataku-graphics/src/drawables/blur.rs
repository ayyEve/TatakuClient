use crate::*;

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
    fn get_pipeline(&self) -> GraphicsPipeline {
        match self.blur_type {
            BlurType::Gaussian { .. } => GraphicsPipeline::GaussianBlur,
            BlurType::Box { .. } => GraphicsPipeline::BoxBlur,
        }
    }
    fn set_pipeline(&mut self, _blend_mode: GraphicsPipeline) {}

    #[cfg(feature="graphics")]
    fn draw(
        &self, 
        _options: &DrawOptions,
        _transform: Matrix, 
        g: &mut dyn DrawEngine,
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
