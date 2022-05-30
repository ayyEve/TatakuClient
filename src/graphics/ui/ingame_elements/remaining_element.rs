use crate::prelude::*;

const FONT_SIZE:f64 = 30.0;
const MAX_CHARS:usize = 7; // 1 for neg, 1 for colon, 2 for secs, 3 for mins
const SIZE:Vector2 = Vector2::new(FONT_SIZE * MAX_CHARS as f64, FONT_SIZE);

pub struct RemainingElement {
    // elapsed_image: Option<SkinnedNumber>,
    elapsed_bounds: Rectangle,

    speed: f32,
    start_time: f32,
    end_time: f32,

    elapsed: f32,
}
impl RemainingElement {
    pub async fn new() -> Self {
        Self {
            // elapsed_image: SkinnedNumber::new(Color::WHITE, -5000.0, Vector2::zero(), 0.0, "normal", None, 0).await.ok(),
            elapsed_bounds: Rectangle::bounds_only(Vector2::zero(), SIZE),
            
            speed: 1.0,
            start_time: -1.0,
            end_time: -1.0,
            elapsed: 0.0,
        }
    }
}

impl InnerUIElement for RemainingElement {
    fn get_bounds(&self) -> Rectangle {
        self.elapsed_bounds.clone()
    }

    fn update(&mut self, manager: &mut IngameManager) {
        // if the values arent set yet, set them
        if self.start_time == -1.0 {
            self.speed = manager.current_mods.speed;
            self.end_time = manager.end_time / self.speed;
            self.start_time = self.end_time - manager.metadata.duration / self.speed;
        }

        self.elapsed = manager.time() / self.speed;
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        let mut bounds = self.elapsed_bounds.clone();
        bounds.current_pos = pos_offset;
        bounds.size *= scale;

        let diff = self.elapsed - self.end_time;
        let secs = (diff / 1000.0).floor();
        let mins = (secs / 60.0).floor() as i16;
        let secs = secs.abs() as i16 % 60;

        let mut text = Text::new(
            Color::WHITE,
            0.0,
            Vector2::zero(),
            (30.0 * scale.x) as u32,
            format!("{mins:02}:{secs:02}"),
            get_font()
        );
        text.center_text(bounds);
        list.push(Box::new(text));
    }
}