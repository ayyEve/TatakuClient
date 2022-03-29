use crate::prelude::*;



pub struct ComboElement {
    combo_image: Option<SkinnedNumber>,
    combo_bounds: Rectangle,
    combo: u16,
}
impl ComboElement {
    pub fn new(combo_bounds: Rectangle) -> Self {
        Self {
            combo_image: SkinnedNumber::new(Color::WHITE, -5000.0, Vector2::zero(), 0.0, "combo", Some('x'), 0).ok(),
            combo_bounds,
            combo: 0,
        }
    }
}

impl InnerUIElement for ComboElement {
    fn update(&mut self, manager: &mut IngameManager) {
        self.combo = manager.score.score.combo;
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        let mut combo_bounds = self.combo_bounds.clone();
        combo_bounds.pos += pos_offset;
        combo_bounds.size *= scale;
        
        if let Some(combo) = &self.combo_image {
            let mut combo = combo.clone();
            combo.number = self.combo as f64;
            combo.current_scale = scale;
            // combo.current_pos += pos_offset;
            combo.center_text(combo_bounds);
            list.push(Box::new(combo.clone()));
        } else {
            let mut combo_text = Text::new(
                Color::WHITE,
                0.0,
                Vector2::zero(),
                (30.0 * scale.x) as u32,
                crate::format_number(self.combo),
                get_font()
            );
            combo_text.center_text(combo_bounds);
            list.push(Box::new(combo_text));
        }
    }
}