use crate::prelude::*;

pub struct HealthBarElement {
    common_game_settings: Arc<CommonGameplaySettings>,
    health_ratio: f32,
    window_size: Arc<WindowSize>,

    healthbar_image: Option<Image>,
    healthbar_bg_image: Option<Image>,
}
impl HealthBarElement {
    pub async fn new(common_game_settings: Arc<CommonGameplaySettings>) -> Self {
        // peppy calls the healthbar texture "scorebar"
        let mut healthbar_image = SkinManager::get_texture("scorebar-colour", true).await;
        let mut healthbar_bg_image = SkinManager::get_texture("scorebar-bg", true).await;

        for i in [&mut healthbar_image, &mut healthbar_bg_image] {
            if let Some(i) = i {
                i.origin = Vector2::ZERO;
                i.color = Color::WHITE;
            }
        }

        Self {
            common_game_settings,
            health_ratio: 0.0,
            window_size: WindowSize::get(),

            healthbar_image,
            healthbar_bg_image
        }
    }
}
impl InnerUIElement for HealthBarElement {
    fn display_name(&self) -> &'static str { "Health Bar" }

    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::ZERO,
            Vector2::new(self.window_size.x / 2.0, DURATION_HEIGHT)
        )
    }
    
    fn update(&mut self, manager: &mut IngameManager) {
        self.window_size = WindowSize::get();
        self.health_ratio = manager.health.get_ratio();
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
        let percent = self.health_ratio;
        let bg_size = Vector2::new(self.window_size.x / 2.0, DURATION_HEIGHT) * scale;

        if let Some(mut fill) = self.healthbar_image.clone() {

            // add bg
            if let Some(mut bg) = self.healthbar_bg_image.clone() {
                bg.pos = pos_offset;
                bg.depth = 1.0;
                bg.set_size(bg_size);
                list.push(bg);
            }
        
            // let snip = DrawState::default().scissor([
            //     pos_offset.x as u32,
            //     pos_offset.y as u32,
            //     (bg_size.x * percent) as u32,
            //     bg_size.y as u32
            // ]);

            
            fill.set_scissor(Some([
                pos_offset.x, 
                pos_offset.y, 
                bg_size.x * percent, 
                bg_size.y
            ]));
            fill.depth = 1.0;
            fill.pos = pos_offset;
            fill.set_size(bg_size);
            // fill.set_draw_state(Some(snip));
            list.push(fill);

        } else {
            let len = self.common_game_settings.healthbar_colors.len();
            let index = ((len as f32 * percent) as usize).min(len - 1);

            // bg
            list.push(Rectangle::new(
                self.common_game_settings.healthbar_bg_color,
                1.0,
                pos_offset,
                bg_size,
                Some(Border::new(self.common_game_settings.healthbar_border_color, 1.8))
            ));

            // fill
            list.push(Rectangle::new(
                self.common_game_settings.healthbar_colors[index],
                2.0,
                pos_offset,
                Vector2::new((self.window_size.x / 2.0) * percent, DURATION_HEIGHT) * scale,
                None
            ));
        }


    }
}