use crate::*;


// this is bad, i dont care
// TODO: care
#[derive(Copy, Clone)]
pub struct Sector {
    pub color: Color,

    pub radius: f32,
    pub start: f32,
    pub end: f32,

    pub border: Option<Border>
}
impl Sector {
    pub fn new(
        radius: f32, 
        start: f32, 
        end: f32, 
        color: Color, 
        border: Option<Border>
    ) -> Self {
        Self {
            radius,
            start,
            end,

            color,

            border,
        }
    }
}

#[cfg(feature="graphics")]
impl TatakuRenderable for Sector {
    fn get_name(&self) -> String { "Sector".to_owned() }

    fn draw(
        &self,
        options: &DrawOptions, 
        transform: Matrix, 
        g: &mut dyn DrawEngine
    ) {
        let Some(blend_mode) = options.blend_mode() else { return; };

        g.draw_arc(
            self.start,
            self.end,
            options.color_with_alpha(self.color),
            self.border,
            20,
            transform,
            blend_mode
        );
    }
}
