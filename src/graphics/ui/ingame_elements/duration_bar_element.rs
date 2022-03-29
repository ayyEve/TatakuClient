use crate::prelude::*;

pub struct DurationBarElement {
    common_game_settings: Arc<CommonGameplaySettings>,
    duration_ratio: f64
}
impl DurationBarElement {
    pub fn new(common_game_settings: Arc<CommonGameplaySettings>) -> Self {
        Self {
            common_game_settings,
            duration_ratio: 0.0
        }
    }
}
impl InnerUIElement for DurationBarElement {
    fn update(&mut self, manager: &mut IngameManager) {
        self.duration_ratio = (manager.time()/manager.end_time) as f64
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        let window_size = Settings::window_size();

        // duration bar
        // duration remaining
        list.push(Box::new(Rectangle::new(
            self.common_game_settings.duration_color,
            1.0,
            pos_offset - Vector2::y_only(DURATION_HEIGHT + 3.0),
            Vector2::new(window_size.x, DURATION_HEIGHT) * scale,
            Some(Border::new(self.common_game_settings.duration_border_color, 1.8 * scale.x))
        )));

        // fill
        list.push(Box::new(Rectangle::new(
            self.common_game_settings.duration_color_full,
            2.0,
            pos_offset - Vector2::y_only(DURATION_HEIGHT + 3.0),
            Vector2::new(window_size.x * self.duration_ratio, DURATION_HEIGHT) * scale,
            None
        )));

    }
}