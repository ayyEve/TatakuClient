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
    fn get_name(&self) -> String { "Blur".to_string() }

    #[cfg(feature="graphics")]
    fn draw(
        &self, 
        options: &DrawOptions,
        _transform: Matrix, 
        g: &mut dyn DrawEngine,
    ) {
        let pipeline = match self.blur_type {
            BlurType::Gaussian { .. } => GraphicsPipeline::GaussianBlur,
            BlurType::Box { .. } => GraphicsPipeline::BoxBlur,
        };

        match (options.pipeline, pipeline) {
            (None, _) => {},
            (Some(got), expected) if expected != got => {
                error!("expected blur {expected:?}, got {got:?}");

                return;
            },
            (Some(_), _) => {},
        }

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
