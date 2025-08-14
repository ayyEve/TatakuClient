use crate::prelude::*;

#[derive(Clone)]
pub struct CharData {
    pub texture: TextureReference,
    pub metrics: fontdue::Metrics
}
impl CharData {
    pub fn advance_width(&self) -> f32 {
        self.metrics.advance_width
    }
}
