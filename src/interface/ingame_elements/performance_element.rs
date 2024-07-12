use crate::prelude::*;
const PADDING:Vector2 = Vector2::new(3.0, 3.0);

const WHITE_TEXT:bool = true;
const NUMBER: u32 = 1_000_000_000;

pub struct PerformanceElement {
    perf_image: Option<SkinnedNumber>,
    perf: f32,

    bounds_size: Vector2,
}
impl PerformanceElement {
    pub async fn new() -> Self {
        Self {
            bounds_size: Text::measure_text_raw(&[Font::Main], 30.0, &format_float(NUMBER, 2), Vector2::ONE, 0.0),
            perf_image: None,
            perf: 0.0,
        }
    }
}

#[async_trait]
impl InnerUIElement for PerformanceElement {
    fn display_name(&self) -> &'static str { "Performance" }

    fn get_bounds(&self) -> Bounds {
        Bounds::new(
            -self.bounds_size.x_portion() - PADDING,
            self.bounds_size + PADDING * 2.0
        )
    }

    fn update(&mut self, manager: &mut GameplayManager) {
        self.perf = manager.score.performance;
    }

    #[cfg(feature="graphics")]
    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
        let bounds_x = self.bounds_size.x_portion();

        if let Some(perf) = &mut self.perf_image {
            perf.number = self.perf as f64;
            perf.scale = scale;

            let mut perf = perf.clone();
            perf.scale = scale;

            // right align
            let size = perf.measure_text();
            perf.pos = pos_offset-bounds_x + ((self.bounds_size * scale).x_portion() - size.x_portion());

            // let size = acc.measure_text();
            // perf.current_pos = pos_offset - bounds_x;
            list.push(perf);
        } else {

            // score bg
            let mut text = Text::new(
                pos_offset - self.bounds_size.x_portion(),
                30.0 * scale.y,
                format!("{:.2}", self.perf),
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
    async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut SkinManager) {
        self.perf_image = SkinnedNumber::new(Vector2::ZERO, self.perf as f64, Color::WHITE, "score", None, 2, skin_manager, source, SkinUsage::Gamemode).await.ok();

        self.bounds_size = if let Some(im) = &mut self.perf_image {
            im.number = NUMBER as f64;
            im.measure_text()
        } else {
            Text::measure_text_raw(&[Font::Main], 30.0, &format_float(NUMBER, 2), Vector2::ONE, 0.0)
        };
    }
}
