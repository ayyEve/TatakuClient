use crate::prelude::*;



pub struct AccuracyElement {
    acc_image: Option<SkinnedNumber>,
    acc: f64,
}
impl AccuracyElement {
    pub fn new() -> Self {
        Self {
            acc_image: SkinnedNumber::new(Color::WHITE, -5000.0, Vector2::zero(), 0.0, "score", Some('%'), 2).ok(),
            acc: 0.0,
        }
    }
}

impl InnerUIElement for AccuracyElement {
    fn update(&mut self, manager: &mut IngameManager) {
        self.acc = calc_acc(&manager.score) * 100.0;
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        if let Some(acc) = &mut self.acc_image {
            let mut acc = acc.clone();
            acc.current_scale = scale;
            acc.number = self.acc;

            let size = acc.measure_text();
            acc.current_pos = pos_offset - Vector2::x_only(size.x);
            list.push(Box::new(acc));
        } else {
            list.push(Box::new(Text::new(
                Color::BLACK,
                0.0,
                pos_offset - Vector2::x_only(200.0),
                (30.0 * scale.x) as u32,
                format!("{:.2}%", self.acc),
                get_font()
            )));
        }
        
    }
}