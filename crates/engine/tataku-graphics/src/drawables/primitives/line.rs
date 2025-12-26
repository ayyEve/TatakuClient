use crate::*;

#[derive(Copy, Clone)]
pub struct Line {
    color: Color,
    vector: Vector2,
    thickness: f32,
}
impl Line {
    pub fn new(
        vector: Vector2,
        thickness: f32, 
        color: Color,
    ) -> Self {
        Self {
            vector,
            thickness,
            color,
        }
    }
}

#[cfg(feature="graphics")]
impl TatakuRenderable for Line {
    fn get_name(&self) -> String { "Line".to_owned() }

    fn draw(
        &self, 
        options: &DrawOptions, 
        transform: Matrix, 
        g: &mut dyn DrawEngine
    ) {
        let color = options.color_with_alpha(self.color);

        let Some(blend_mode) = options.blend_mode() else { return; };

        g.draw_line(
            self.vector,
            self.thickness, 
            color, 
            transform, 
            blend_mode
        );
    }
}
