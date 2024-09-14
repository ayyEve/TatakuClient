use crate::prelude::*;

const BOX_SIZE:Vector2 = Vector2::new(40.0, 40.0);

pub struct JudgementCounterElement {
    hit_counts: Vec<(String, u32)>,
    button_image: Option<Image>,
    colors: HashMap<String, Color>,
}
impl JudgementCounterElement {
    pub async fn new() -> Self {
        Self {
            hit_counts: Vec::new(),
            button_image: None,
            colors: HashMap::new()
        }
    }
}
#[async_trait]
impl InnerUIElement for JudgementCounterElement {
    fn display_name(&self) -> &'static str { "Judgement Counter" }

    fn get_bounds(&self) -> Bounds {
        let box_size = self.button_image.as_ref().map(|b|b.size()).unwrap_or(BOX_SIZE);
        Bounds::new(
            -box_size,
            Vector2::new(box_size.x, box_size.y * self.hit_counts.len() as f32)
        )
    }
    
    fn update(&mut self, manager: &mut GameplayManager) {
        // TODO: improve this
        self.hit_counts.clear();
        let score = &manager.score.score;

        let load_colors = self.colors.is_empty();

        for judge in manager.judgments.iter() {
            let txt = judge.display_name;
            if txt.is_empty() { continue }

            let count = score.judgments.get(judge.id).cloned().unwrap_or_default();
            self.hit_counts.push((txt.to_owned(), count as u32));

            if load_colors {
                self.colors.insert(txt.to_owned(), judge.color);
            }
        }
    }

    #[cfg(feature="graphics")]
    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
        let box_size = self.button_image.as_ref().map(|b|b.size()).unwrap_or(BOX_SIZE) * scale;
        
        let base_pos = pos_offset - BOX_SIZE;

        for (i, (txt, count)) in self.hit_counts.iter().enumerate() {
            let pos = base_pos + Vector2::new(0.0, box_size.y * i as f32);
            let box_width;

            if let Some(mut btn) = self.button_image.clone() {
                btn.pos = pos + box_size / 2.0;
                btn.scale = scale;
                box_width = btn.size().x * scale.x;

                if let Some(&color) = self.colors.get(txt) {
                    btn.color = color;
                }
                
                list.push(btn);
            } else {
                box_width = (BOX_SIZE * scale).x;

                // draw bg box
                list.push(Rectangle::new(
                    pos,
                    BOX_SIZE * scale,
                    *self.colors.get(txt).unwrap_or(&Color::new(0.0, 0.0, 0.0, 0.8)), // TODO: get a proper color
                    Some(Border::new(Color::BLACK, 2.0))
                ));
            }

            // draw text/count
            let mut text = Text::new(
                pos,
                20.0 * scale.y,
                if count == &0 {txt.clone()} else {format!("{}", count)},
                Color::WHITE,
                Font::Main
            );
            let text_size = text.measure_text();
            let max_width = box_width - 10.0; // padding of 10
            if text_size.x >= max_width {
                text.set_font_size(20.0 * scale.x * max_width / text_size.x)
            }
            text.center_text(&Bounds::new(pos, box_size));

            list.push(text);
        }
    }

    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut dyn SkinProvider) {
        self.button_image = skin_manager.get_texture("inputoverlay-key", source, SkinUsage::Gamemode, false).await
    }
}
