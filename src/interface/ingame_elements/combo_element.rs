use crate::prelude::*;

pub struct ComboElement {
    combo_image: Option<SkinnedNumber>,
    combo_bounds: Bounds,
    combo: u16,
}
impl ComboElement {
    pub async fn new(combo_bounds: Bounds) -> Self {
        Self {
            combo_image: None,
            combo_bounds,
            combo: 0,
        }
    }
}
#[async_trait]
impl InnerUIElement for ComboElement {
    fn display_name(&self) -> &'static str { "Combo" }

    fn get_bounds(&self) -> Bounds { self.combo_bounds }

    fn update(&mut self, manager: &mut GameplayManager) {
        self.combo = manager.score.score.combo;
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
        let mut combo_bounds = self.combo_bounds;
        combo_bounds.pos = pos_offset;
        combo_bounds.size *= scale;
        
        if let Some(combo) = &mut self.combo_image {
            combo.number = self.combo as f64;

            let mut combo = combo.clone();
            combo.scale = scale;
            combo.center_text(&combo_bounds);
            list.push(combo);
        } else {
            let mut combo_text = Text::new(
                Vector2::ZERO,
                30.0 * scale.x,
                format_number(self.combo),
                Color::WHITE,
                Font::Main
            );
            combo_text.center_text(&combo_bounds);
            list.push(combo_text);
        }
    }

    async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut SkinManager) {
        self.combo_image = SkinnedNumber::new(Vector2::ZERO, 0.0, Color::WHITE, "combo", Some('x'), 0, skin_manager, source, SkinUsage::Gamemode).await.ok()
    }
}

