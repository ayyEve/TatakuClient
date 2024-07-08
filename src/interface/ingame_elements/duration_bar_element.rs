use crate::prelude::*;

pub struct DurationBarElement {
    common_game_settings: Arc<CommonGameplaySettings>,
    duration_ratio: f32,
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
#[async_trait]
impl InnerUIElement for DurationBarElement {
    fn display_name(&self) -> &'static str { "Duration Bar" }

    fn get_bounds(&self) -> Bounds {
        Bounds::new(
            Vector2::with_y(-(DURATION_HEIGHT + 3.0)),
            Vector2::new(self.window_size.x, DURATION_HEIGHT)
        )
    }

    fn update(&mut self, manager: &mut GameplayManager) {
        self.window_size = WindowSize::get();
        self.duration_ratio = manager.time() / manager.end_time
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
        // fill
        list.push(Rectangle::new(
            pos_offset - Vector2::with_y(DURATION_HEIGHT + 3.0),
            Vector2::new(self.window_size.x * self.duration_ratio, DURATION_HEIGHT) * scale,
            self.common_game_settings.duration_color_full,
            None
        ));

        // border
        list.push(Rectangle::new(
            pos_offset + Vector2::with_y(-(DURATION_HEIGHT + 3.0)),
            Vector2::new(self.window_size.x, DURATION_HEIGHT) * scale,
            self.common_game_settings.duration_color,
            Some(Border::new(self.common_game_settings.duration_border_color, 1.8 * scale.x))
        ));
    }
}