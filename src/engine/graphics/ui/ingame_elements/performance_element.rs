use crate::prelude::*;
const PADDING:Vector2 = Vector2::new(3.0, 3.0);

const WHITE_TEXT:bool = true;

pub struct PerformanceElement {
    perf_image: Option<SkinnedNumber>,
    perf: f32,

    bounds_size: Vector2,
}
impl PerformanceElement {
    pub async fn new() -> Self {
        let number:u32 = 1_000_000_000;
        let mut perf_image = SkinnedNumber::new(Color::WHITE, -5000.0, Vector2::zero(), 0.0, "score", None, 2).await.ok();
        
        // get the bounds
        // TODO: make it not rely on this shit
        let bounds_size = if let Some(im) = &mut perf_image {
            im.number = number as f64;
            im.measure_text()
        } else {
            Text::new(
                Color::BLACK,
                0.0,
                Vector2::zero(),
                30,
                crate::format_float(number, 2),
                get_font()
            ).measure_text()
        };

        Self {
            bounds_size,
            perf_image,
            perf: 0.0,
        }
    }
}

impl InnerUIElement for PerformanceElement {
    fn display_name(&self) -> &'static str { "Performance" }

    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            -self.bounds_size.x() - PADDING,
            self.bounds_size + PADDING * 2.0
        )
    }

    fn update(&mut self, manager: &mut IngameManager) {
        self.perf = manager.score.performance;
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
        let bounds_x = self.bounds_size.x();

        if let Some(perf) = &mut self.perf_image {
            perf.number = self.perf as f64;
            perf.current_scale = scale;

            let mut perf = perf.clone();
            perf.current_scale = scale;

            // right align
            let size = perf.measure_text();
            perf.current_pos = pos_offset-bounds_x + ((self.bounds_size * scale).x() - size.x());

            // let size = acc.measure_text();
            // perf.current_pos = pos_offset - bounds_x;
            list.push(perf);
        } else {

            // score bg
            let mut text = Text::new(
                if WHITE_TEXT { Color::WHITE } else { Color::BLACK },
                0.0,
                pos_offset - self.bounds_size.x(),
                30 * scale.y as u32,
                format!("{:.2}", self.perf),
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
            list.push(text);
        }
        
    }
}
