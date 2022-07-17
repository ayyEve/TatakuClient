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
        let mut acc_image = SkinnedNumber::new(Color::WHITE, -5000.0, Vector2::zero(), 0.0, "score", Some('%'), 2).await.ok();
        
        // get the bounds
        // TODO: make it not rely on this shit
        let bounds_size = if let Some(im) = &mut acc_image {
            im.number = 100.0; 
            im.measure_text()
        } else {
            Text::new(
                Color::BLACK,
                0.0,
                Vector2::zero(),
                30,
                "100.00%".to_owned(),
                get_font()
            ).measure_text()
        };

        Self {
            bounds_size,
            acc_image,
            acc: 0.0,
        }
    }
}

impl InnerUIElement for AccuracyElement {
    fn display_name(&self) -> &'static str { "Accuracy" }

    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            -self.bounds_size.x() - PADDING,
            self.bounds_size + PADDING * 2.0
        )
    }

    fn update(&mut self, manager: &mut IngameManager) {
        self.acc = calc_acc(&manager.score) * 100.0;    
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        let bounds_x = self.bounds_size.x();

        if let Some(mut acc) = self.acc_image.clone() {
            acc.number = self.acc;
            acc.current_scale = scale;

            // let size = acc.measure_text();
            acc.current_pos = pos_offset - bounds_x;
            list.push(Box::new(acc));
        } else {

            // score bg
            let mut text = Text::new(
                if WHITE_TEXT { Color::WHITE } else { Color::BLACK },
                0.0,
                pos_offset - self.bounds_size.x(),
                30 * scale.y as u32,
                format!("{:.2}%", self.acc),
                get_font()
            );
            
            // space needed to align this to the right
            let text_size = text.measure_text();
            let right_align = self.bounds_size.x - text_size.x;
            // offset text position to account for right alrign
            text.current_pos.x = pos_offset.x - self.bounds_size.x + right_align;

            if !WHITE_TEXT {
                list.push(visibility_bg(
                    text.current_pos - PADDING,
                    text_size + PADDING * 2.0,
                    1.0
                ));
            }
            
            // score text
            list.push(Box::new(text));
        }
        
    }
}