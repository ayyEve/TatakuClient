use crate::*;

pub struct FlashlightDrawable {
    pub center: Vector2,
    pub radius: f32,
    pub fade_radius: f32,
    pub bounds: Bounds,

    pub color: Color,
}
impl FlashlightDrawable {
    pub fn new(
        center: Vector2,
        radius: f32, 
        fade_radius: f32, 
        bounds: Bounds, 
        color: Color
    ) -> Self {
        Self {
            center,
            radius, 
            fade_radius, 
            bounds,
            color,
        }
    }
}

impl TatakuRenderable for FlashlightDrawable {
    fn get_name(&self) -> String { "Flashlight".to_owned() }

    #[cfg(feature="graphics")]
    fn draw(
        &self, 
        options: &DrawOptions,
        transform: Matrix, 
        g: &mut dyn DrawEngine,
    ) {
        if !matches!(options.pipeline, None | Some(GraphicsPipeline::Flashlight))
        {
            error!("expected flashlight, got {:?}", options.pipeline);
            return;
        }

        g.draw_flashlight(
            transform, 
            FlashlightData {
                center: self.center,
                flashlight_radius: self.radius,
                fade_radius: self.fade_radius,
                color: self.color,
            }
        );
    }
} 
