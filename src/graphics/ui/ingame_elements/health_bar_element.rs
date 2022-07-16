use crate::prelude::*;

pub struct HealthBarElement {
    common_game_settings: Arc<CommonGameplaySettings>,
    health_ratio: f64,
    window_size: Arc<WindowSize>
}
impl HealthBarElement {
    pub fn new(common_game_settings: Arc<CommonGameplaySettings>) -> Self {
        Self {
            common_game_settings,
            health_ratio: 0.0,
            window_size: WindowSize::get()
        }
    }
}
impl InnerUIElement for HealthBarElement {
    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::zero(),
            Vector2::new(self.window_size.x / 2.0, DURATION_HEIGHT)
        )
    }
    
    fn update(&mut self, manager: &mut IngameManager) {
        self.window_size = WindowSize::get();
        self.health_ratio = manager.health.get_ratio() as f64
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        let percent = self.health_ratio;
        let len = self.common_game_settings.healthbar_colors.len();
        let index = ((len as f64 * percent) as usize).min(len - 1);
        // bg
        list.push(Box::new(Rectangle::new(
            self.common_game_settings.healthbar_bg_color,
            1.0,
            pos_offset,
            Vector2::new(self.window_size.x / 2.0, DURATION_HEIGHT) * scale,
            Some(Border::new(self.common_game_settings.healthbar_border_color, 1.8))
        )));
        // fill
        list.push(Box::new(Rectangle::new(
            self.common_game_settings.healthbar_colors[index],
            2.0,
            pos_offset,
            Vector2::new((self.window_size.x / 2.0) * percent, DURATION_HEIGHT) * scale,
            None
        )));

    }
}