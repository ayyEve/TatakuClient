use crate::prelude::*;

const FONT_SIZE:f32 = 30.0;
const MAX_CHARS:usize = 7; // 1 for neg, 1 for colon, 2 for secs, 3 for mins
const SIZE:Vector2 = Vector2::new(FONT_SIZE * MAX_CHARS as f32, FONT_SIZE);

pub struct RemainingElement {
    // elapsed_image: Option<SkinnedNumber>,
    elapsed_bounds: Bounds,

    speed: f32,
    start_time: f32,
    end_time: f32,

    elapsed: f32,
}
impl RemainingElement {
    pub async fn new() -> Self {
        Self {
            // elapsed_image: SkinnedNumber::new(Color::WHITE, -5000.0, Vector2::ZERO, 0.0, "normal", None, 0).await.ok(),
            elapsed_bounds: Bounds::new(Vector2::ZERO, SIZE),
            
            speed: 1.0,
            start_time: -1.0,
            end_time: -1.0,
            elapsed: 0.0,
        }
    }
}
#[async_trait]
impl InnerUIElement for RemainingElement {
    fn display_name(&self) -> &'static str { "Time Remaining" }
    fn get_bounds(&self) -> Bounds { self.elapsed_bounds }

    fn update(&mut self, manager: &mut GameplayManager) {
        // if the values arent set yet, set them
        if self.start_time == -1.0 {
            self.speed = manager.current_mods.get_speed();
            self.end_time = manager.end_time / self.speed;
            self.start_time = self.end_time - manager.metadata.duration / self.speed;
        }

        self.elapsed = manager.time() / self.speed;
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
        let mut bounds = self.elapsed_bounds;
        bounds.pos = pos_offset;
        bounds.size *= scale;

        let diff = self.elapsed - self.end_time;
        let sign = if diff < 0.0 {"-"} else {""};
        let secs = (diff / 1000.0).floor().abs();
        let mins = (secs / 60.0).floor() as i16;
        let secs = secs as i16 % 60;

        let mut text = Text::new(
            Vector2::ZERO,
            30.0 * scale.x,
            format!("{sign}{mins:02}:{secs:02}"),
            Color::WHITE,
            Font::Main
        );
        text.center_text(&bounds);
        list.push(text);
    }
}