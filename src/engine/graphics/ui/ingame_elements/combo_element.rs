use crate::prelude::*;

pub struct ComboElement {
    combo_image: Option<SkinnedNumber>,
    combo_bounds: Rectangle,
    combo: u16,
}
impl ComboElement {
    pub async fn new(combo_bounds: Rectangle) -> Self {
        Self {
            combo_image: SkinnedNumber::new(Color::WHITE, -5000.0, Vector2::zero(), 0.0, "combo", Some('x'), 0).await.ok(),
            combo_bounds,
            combo: 0,
        }
    }
}

impl InnerUIElement for ComboElement {
    fn display_name(&self) -> &'static str { "Combo" }

    fn get_bounds(&self) -> Rectangle {
        self.combo_bounds.clone()
    }

    fn update(&mut self, manager: &mut IngameManager) {
        self.combo = manager.score.score.combo;
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
        let mut combo_bounds = self.combo_bounds.clone();
        combo_bounds.current_pos = pos_offset;
        combo_bounds.size *= scale;
        
        if let Some(combo) = &mut self.combo_image {
            combo.number = self.combo as f64;

            let mut combo = combo.clone();
            combo.current_scale = scale;
            combo.center_text(combo_bounds);
            list.push(combo);
        } else {
            let mut combo_text = Text::new(
                Color::WHITE,
                0.0,
                Vector2::zero(),
                (30.0 * scale.x) as u32,
                crate::format_number(self.combo),
                get_font()
            );
            combo_text.center_text(&combo_bounds);
            list.push(combo_text);
        }
    }
}
