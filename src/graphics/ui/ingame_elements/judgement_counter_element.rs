

use crate::prelude::*;

const BOX_SIZE:Vector2 = Vector2::new(40.0, 40.0);

pub struct JudgementCounterElement {
    hit_counts: Vec<(String, u32)>,
    button_image: Option<Image>
}
impl JudgementCounterElement {
    pub fn new() -> Self {
        let mut button_image= SKIN_MANAGER.write().get_texture("inputoverlay-key", true);
        if let Some(image) = &mut button_image {
            image.depth = -100.1;
        }

        Self {
            hit_counts: Vec::new(),
            button_image,
        }
    }
}
impl InnerUIElement for JudgementCounterElement {
    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            -BOX_SIZE,
            Vector2::new(BOX_SIZE.x, BOX_SIZE.y * self.hit_counts.len() as f64)
        )
    }
    
    fn update(&mut self, manager: &mut IngameManager) {

        // TODO: improve this
        self.hit_counts.clear();
        let playmode = manager.gamemode.playmode();
        let score = &manager.score.score;
        for (hit_type, count) in [
            (ScoreHit::Miss, score.xmiss),
            (ScoreHit::X50, score.x50),
            (ScoreHit::X100, score.x100),
            (ScoreHit::Xkatu, score.xkatu),
            (ScoreHit::X300, score.x300),
            (ScoreHit::Xgeki, score.xgeki),
        ] {
            let txt = get_score_hit_string(&playmode, &hit_type);
            if txt.is_empty() {continue}

            self.hit_counts.push((txt, count as u32));
        }

    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        let font = get_font();
        let pad = BOX_SIZE;
        
        let base_pos = pos_offset - pad;

        for (i, (txt, count)) in self.hit_counts.iter().enumerate() {
            let pos = base_pos + Vector2::new(0.0, pad.y * i as f64);

            if let Some(btn) = &self.button_image {
                let mut btn = btn.clone();
                btn.current_pos = pos + pad / 2.0;
                btn.current_scale = scale;
                
                list.push(Box::new(btn));
            } else {
                // draw bg box
                list.push(Box::new(Rectangle::new(
                    Color::new(0.0, 0.0, 0.0, 0.8), // TODO: get a proper color
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
                (20.0 * scale.x) as u32,
                if count == &0 {txt.clone()} else {format!("{}", count)},
                font.clone()
            );
            text.center_text(Rectangle::bounds_only(pos, BOX_SIZE));
            list.push(Box::new(text));
        }
    }
}