use crate::prelude::*;
const PADDING:Vector2 = Vector2::new(3.0, 3.0);

const WHITE_TEXT:bool = true;

pub struct AccuracyElement {
    acc_image: Option<SkinnedNumber>,
    acc: f64,

    bounds_size: Vector2,
}
impl AccuracyElement {
    pub async fn new() -> Self {
        Self {
            bounds_size: Text::measure_text_raw(&[Font::Main], 30.0, "100.00%", Vector2::ONE, 0.0),
            acc_image: None,
            acc: 0.0,
        }
    }
}

#[async_trait]
impl InnerUIElement for AccuracyElement {
    fn display_name(&self) -> &'static str { "Accuracy" }

    fn get_bounds(&self) -> Bounds {
        Bounds::new(
            -self.bounds_size.x_portion() - PADDING,
            self.bounds_size + PADDING * 2.0
        )
    }

    fn update(&mut self, manager: &mut GameplayManager) {
        self.acc = manager.score.accuracy * 100.0;    
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
        let bounds_x = self.bounds_size.x_portion();

        if let Some(mut acc) = self.acc_image.clone() {
            acc.number = self.acc;
            acc.scale = scale;

            // let size = acc.measure_text();
            acc.pos = pos_offset - bounds_x;
            list.push(acc);
        } else {

            // score bg
            let mut text = Text::new(
                pos_offset - self.bounds_size.x_portion(),
                30.0 * scale.y,
                format!("{:.2}%", self.acc),
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


    async fn reload_skin(&mut self, skin_manager: &mut SkinManager) {
        self.acc_image = SkinnedNumber::new(Vector2::ZERO, 0.0, Color::WHITE, "score", Some('%'), 2, skin_manager).await.ok();
        
        // get the bounds
        // TODO: make it not rely on this shit
        self.bounds_size = if let Some(im) = &mut self.acc_image {
            im.number = 100.0; 
            im.measure_text()
        } else {
            Text::measure_text_raw(&[Font::Main], 30.0, "100.00%", Vector2::ONE, 0.0)
        };
    }
}