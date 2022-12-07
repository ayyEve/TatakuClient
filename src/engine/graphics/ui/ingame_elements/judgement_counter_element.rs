

use crate::prelude::*;

const BOX_SIZE:Vector2 = Vector2::new(40.0, 40.0);

pub struct JudgementCounterElement {
    hit_counts: Vec<(String, u32)>,
    button_image: Option<Image>,
    colors: HashMap<String, Color>,
}
impl JudgementCounterElement {
    pub async fn new() -> Self {

        let mut button_image= SkinManager::get_texture("inputoverlay-key", true).await;
        if let Some(image) = &mut button_image {
            image.depth = -100.1;
        }

        Self {
            hit_counts: Vec::new(),
            button_image,
            colors: HashMap::new()
        }
    }
}
impl InnerUIElement for JudgementCounterElement {
    fn display_name(&self) -> &'static str { "Judgement Counter" }

    fn get_bounds(&self) -> Rectangle {
        let box_size = self.button_image.as_ref().map(|b|b.size()).unwrap_or(BOX_SIZE);
        Rectangle::bounds_only(
            -box_size,
            Vector2::new(box_size.x, box_size.y * self.hit_counts.len() as f64)
        )
    }
    
    fn update(&mut self, manager: &mut IngameManager) {
        // TODO: improve this
        self.hit_counts.clear();
        let score = &manager.score.score;

        let load_colors = self.colors.is_empty();

        for judge in manager.judgment_type.variants().iter() {
            let txt = judge.as_str_display();
            if txt.is_empty() { continue }

            let count = score.judgments.get(judge.as_str_internal()).map(|n|*n).unwrap_or_default();
            self.hit_counts.push((txt.to_owned(), count as u32));

            if load_colors {
                self.colors.insert(txt.to_owned(), judge.color());
            }
        }
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        let font = get_font();
        let box_size = self.button_image.as_ref().map(|b|b.size()).unwrap_or(BOX_SIZE) * scale;
        
        let base_pos = pos_offset - BOX_SIZE;

        for (i, (txt, count)) in self.hit_counts.iter().enumerate() {
            let pos = base_pos + Vector2::new(0.0, box_size.y * i as f64);
            let box_width;

            if let Some(mut btn) = self.button_image.clone() {
                btn.current_pos = pos + box_size / 2.0;
                btn.current_scale = scale;
                box_width = btn.size().x * scale.x;

                if let Some(&color) = self.colors.get(txt) {
                    btn.current_color = color;
                }
                
                list.push(Box::new(btn));
            } else {
                box_width = (BOX_SIZE * scale).x;

                // draw bg box
                list.push(Box::new(Rectangle::new(
                    *self.colors.get(txt).unwrap_or(&Color::new(0.0, 0.0, 0.0, 0.8)), // TODO: get a proper color
                    -100.0,
                    pos,
                    BOX_SIZE * scale,
                    Some(Border::new(Color::BLACK, 2.0))
                )));
            }

            // draw text/count
            let mut text = Text::new(
                Color::WHITE,
                -100.1,
                pos,
                (20.0 * scale.y) as u32,
                if count == &0 {txt.clone()} else {format!("{}", count)},
                font.clone()
            );
            let text_size = text.measure_text();
            let max_width = box_width - 10.0; // padding of 10
            if text_size.x >= max_width {
                text.font_size = FontSize::new((20.0 * scale.x * max_width / text_size.x) as f32).unwrap();
            }
            text.center_text(Rectangle::bounds_only(pos, box_size));

            list.push(Box::new(text));
        }
    }
}
