use crate::prelude::*;
const PADDING:Vector2 = Vector2::new(3.0, 3.0);
const WHITE_TEXT:bool = true;


pub struct ScoreElement {
    score: u64,
    bounds_size: Vector2,
    score_image: Option<SkinnedNumber>,
}
impl ScoreElement {
    pub async fn new() -> Self {
        let number:u32 = 1_000_000_000;
        let mut score_image = SkinnedNumber::new(Vector2::ZERO, 0.0, Color::WHITE, "score", None, 0).await.ok();

        // get the bounds
        // TODO: make it not rely on this shit
        let bounds_size = if let Some(im) = &mut score_image {
            im.number = number as f64; 
            im.measure_text()
        } else {
            Text::new(
                Vector2::ZERO,
                30.0,
                crate::format_number(number),
                Color::BLACK,
                get_font()
            ).measure_text()
        };


        Self {
            bounds_size,
            score_image,
            score: 0
        }
    }
}

impl InnerUIElement for ScoreElement {
    fn display_name(&self) -> &'static str { "Score" }

    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            -self.bounds_size.x_portion() - PADDING,
            self.bounds_size + PADDING * 2.0
        )
    }


    fn update(&mut self, manager: &mut IngameManager) {
        self.score = manager.score.score.score;
    }

    fn draw(&mut self, pos_offset:Vector2, scale:Vector2, list: &mut RenderableCollection) {
        // score
        if let Some(score) = &mut self.score_image {
            score.number = self.score as f64;

            let mut score = score.clone();
            score.scale = scale;

            let size = score.measure_text();

            // TODO: right align
            score.pos = pos_offset-self.bounds_size.x_portion() + ((self.bounds_size * scale).x_portion() - size.x_portion());
            
            list.push(score.clone());
        } else {
            
            // score bg
            let mut text = Text::new(
                pos_offset - self.bounds_size.x_portion(),
                30.0 * scale.y,
                crate::format_number(self.score),
                if WHITE_TEXT { Color::WHITE } else { Color::BLACK },
                get_font()
            );
            // space needed to align this to the right
            let text_size = text.measure_text();
            let right_align = self.bounds_size.x - text_size.x;
            // offset text position to account for right alrign
            text.pos.x = pos_offset.x - self.bounds_size.x + right_align;

            if !WHITE_TEXT {
                list.push(visibility_bg(
                    text.pos - PADDING,
                    text_size + PADDING * 2.0
                ));
            }
            
            // score text
            list.push(text);
        }

    }
}

