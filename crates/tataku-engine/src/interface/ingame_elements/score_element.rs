use crate::prelude::*;
const PADDING:Vector2 = Vector2::new(3.0, 3.0);
const WHITE_TEXT:bool = true;
const NUMBER: u32 = 1_000_000_000;


pub struct ScoreElement {
    score: u64,
    bounds_size: Vector2,
    score_image: Option<SkinnedNumber>,
}
impl ScoreElement {
    pub async fn new() -> Self {
        Self {
            bounds_size: Text::measure_text_raw(&[Font::Main], 30.0, &format_number(NUMBER), Vector2::ONE, 0.0),
            score_image: None,
            score: 0
        }
    }
}

#[async_trait]
impl InnerUIElement for ScoreElement {
    fn display_name(&self) -> &'static str { "Score" }

    fn get_bounds(&self) -> Bounds {
        Bounds::new(
            -self.bounds_size.x_portion() - PADDING,
            self.bounds_size + PADDING * 2.0
        )
    }


    fn update(&mut self, manager: &mut GameplayManager) {
        self.score = manager.score.score.score;
    }

    #[cfg(feature="graphics")]
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
                format_number(self.score),
                if WHITE_TEXT { Color::WHITE } else { Color::BLACK },
                Font::Main
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


    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut dyn SkinProvider) {
        self.score_image = SkinnedNumber::new(Vector2::ZERO, 0.0, Color::WHITE, "score", None, 0, skin_manager, source, SkinUsage::Gamemode).await.ok();

        self.bounds_size = if let Some(im) = &mut self.score_image {
            im.number = NUMBER as f64;
            im.measure_text()
        } else {
            Text::measure_text_raw(&[Font::Main], 30.0, &format_number(NUMBER), Vector2::ONE, 0.0)
        };
    }
}

