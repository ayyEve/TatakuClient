use crate::prelude::*;
const HEALTH_DAMP: f32 = 1.0;

pub struct HealthBarElement {
    common_game_settings: Arc<CommonGameplaySettings>,
    health_ratio: f32,
    window_size: Arc<WindowSize>,

    
    last_health_ratio: f32,
    last_health_time: f32,

    healthbar_image: Option<Image>,
    healthbar_bg_image: Option<Image>,
}
impl HealthBarElement {
    pub async fn new(common_game_settings: Arc<CommonGameplaySettings>) -> Self {


        Self {
            common_game_settings,
            health_ratio: 0.0,
            last_health_ratio: -1000.0,
            
            window_size: WindowSize::get(),
            last_health_time: 0.0,

            healthbar_image: None,
            healthbar_bg_image: None
        }
    }
}
#[async_trait]
impl InnerUIElement for HealthBarElement {
    fn display_name(&self) -> &'static str { "Health Bar" }

    fn get_bounds(&self) -> Bounds {
        Bounds::new(
            Vector2::ZERO,
            Vector2::new(self.window_size.x / 2.0, DURATION_HEIGHT)
        )
    }

    async fn reload_skin(&mut self, skin_manager: &mut SkinManager) {
        // peppy calls the healthbar texture "scorebar"
        self.healthbar_image = skin_manager.get_texture("scorebar-colour", true).await;
        self.healthbar_bg_image = skin_manager.get_texture("scorebar-bg", true).await;

        for i in [&mut self.healthbar_image, &mut self.healthbar_bg_image] {
            let Some(i) = i else { continue };
            i.origin = Vector2::ZERO;
            i.color = Color::WHITE;
        }
    }
    
    
    fn update(&mut self, manager: &mut IngameManager) {
        self.window_size = WindowSize::get();

        self.health_ratio = manager.health.get_ratio();
        if self.last_health_ratio == -1000.0 {
            self.last_health_ratio = self.health_ratio; 
        }

        let time = manager.time();
        let time_diff = time - self.last_health_time;
        self.last_health_time = time;

        if self.health_ratio < self.last_health_ratio {
            let ratio_diff = (self.last_health_ratio - self.health_ratio).max(0.05) * time_diff * HEALTH_DAMP / 1000.0;
            self.last_health_ratio = self.last_health_ratio - ratio_diff;
        } else {
            self.last_health_ratio = self.health_ratio;
        }
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
        let percent = self.health_ratio;
        let bg_size = Vector2::new(self.window_size.x / 2.0, DURATION_HEIGHT) * scale;

        if let Some(mut fill) = self.healthbar_image.clone() {

            // add bg
            if let Some(mut bg) = self.healthbar_bg_image.clone() {
                bg.pos = pos_offset;
                bg.set_size(bg_size);
                list.push(bg);
            }

            fill.pos = pos_offset;
            fill.set_size(bg_size);

            let width = bg_size.x * percent;
            list.push_scissor([pos_offset.x, pos_offset.y, width, bg_size.y]);
            list.push(fill.clone());
            list.pop_scissor();

            // add drained health
            let width2 = bg_size.x * self.last_health_ratio;
            // if width2 != width {
                fill.color.a = 0.2;

                list.push_scissor([pos_offset.x + width, pos_offset.y, width2 - width, bg_size.y]);
                list.push(fill);
                list.pop_scissor();
            // }

        } else {
            let len = self.common_game_settings.healthbar_colors.len();
            let index = ((len as f32 * percent) as usize).min(len - 1);

            // bg
            list.push(Rectangle::new(
                pos_offset,
                bg_size,
                self.common_game_settings.healthbar_bg_color,
                Some(Border::new(self.common_game_settings.healthbar_border_color, 1.8))
            ));

            // fill
            list.push(Rectangle::new(
                pos_offset,
                Vector2::new((self.window_size.x / 2.0) * percent, DURATION_HEIGHT) * scale,
                self.common_game_settings.healthbar_colors[index],
                None
            ));
        }


    }
}