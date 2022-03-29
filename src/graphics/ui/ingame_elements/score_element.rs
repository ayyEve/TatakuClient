use crate::prelude::*;



pub struct ScoreElement {
    score_image: Option<SkinnedNumber>,
    score: u64,
}
impl ScoreElement {
    pub fn new() -> Self {
        Self {
            score_image: SkinnedNumber::new(Color::WHITE, -5000.0, Vector2::zero(), 0.0, "score", None, 0).ok(),
            score: 0
        }
    }
}

impl InnerUIElement for ScoreElement {
    fn get_bounds(&self) -> Rectangle {
        let size = if let Some(i) = &self.score_image {
            i.measure_text()
        } else {
            Vector2::new(-200.0, 10.0)
        };

        Rectangle::bounds_only(
            Vector2::x_only(-size.x),
            size
        )
    }


    fn update(&mut self, manager: &mut IngameManager) {
        self.score = manager.score.score.score;
    }

    fn draw(&mut self, pos_offset:Vector2, scale:Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        // score
        if let Some(score) = &mut self.score_image {
            score.number = self.score as f64;

            let mut score = score.clone();
            score.current_pos = pos_offset + Vector2::x_only(-score.measure_text().x);
            score.current_scale = scale;
            
            list.push(Box::new(score.clone()));
        } else {
            let font = get_font();
            
            // score bg
            let text = Text::new(
                Color::BLACK,
                0.0,
                pos_offset - Vector2::new(200.0, 10.0),
                30 * scale.x as u32,
                crate::format_number(self.score),
                font.clone()
            );
            list.push(visibility_bg(
                text.current_pos,
                text.measure_text(),
                1.0
            ));
            
            // score text
            list.push(Box::new(text));
        }

    }
}