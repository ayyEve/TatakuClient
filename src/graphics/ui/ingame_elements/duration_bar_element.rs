use crate::prelude::*;

pub struct DurationBarElement {
    common_game_settings: Arc<CommonGameplaySettings>,
    duration_ratio: f64,
    window_size: Arc<WindowSize>
}
impl DurationBarElement {
    pub fn new(common_game_settings: Arc<CommonGameplaySettings>) -> Self {
        Self {
            common_game_settings,
            duration_ratio: 0.0,
            window_size: WindowSize::get()
        }
    }
}
impl InnerUIElement for DurationBarElement {
    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::y_only(-(DURATION_HEIGHT + 3.0)),
            Vector2::new(self.window_size.x, DURATION_HEIGHT)
        )
    }

    fn update(&mut self, manager: &mut IngameManager) {
        self.window_size = WindowSize::get();
        self.duration_ratio = (manager.time()/manager.end_time) as f64
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>) {

        // duration bar
        // duration remaining
        list.push(Box::new(Rectangle::new(
            self.common_game_settings.duration_color,
            1.0,
            pos_offset + Vector2::y_only(-(DURATION_HEIGHT + 3.0)),
            Vector2::new(self.window_size.x, DURATION_HEIGHT) * scale,
            Some(Border::new(self.common_game_settings.duration_border_color, 1.8 * scale.x))
        )));

        // fill
        list.push(Box::new(Rectangle::new(
            self.common_game_settings.duration_color_full,
            2.0,
            pos_offset - Vector2::y_only(DURATION_HEIGHT + 3.0),
            Vector2::new(self.window_size.x * self.duration_ratio, DURATION_HEIGHT) * scale,
            None
        )));

    }
}