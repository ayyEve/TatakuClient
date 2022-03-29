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
    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::zero(),
            if let Some(i) = &self.combo_image {
                i.measure_text()
            } else {
                Text::new(
                    Color::BLACK,
                    0.0,
                    Vector2::zero(),
                    30,
                    crate::format_number(self.combo),
                    get_font()
                ).measure_text()
            }
        )
    }

    fn update(&mut self, manager: &mut IngameManager) {
        self.combo = manager.score.score.combo;
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        let mut combo_bounds = self.combo_bounds.clone();
        combo_bounds.current_pos += pos_offset;
        combo_bounds.size *= scale;
        
        if let Some(combo) = &mut self.combo_image {
            combo.number = self.combo as f64;

            let mut combo = combo.clone();
            combo.current_scale = scale;
            combo.center_text(combo_bounds);
            list.push(Box::new(combo));
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