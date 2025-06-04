use crate::prelude::*;
const HEALTH_DAMP: f32 = 1.0;

struct HealthBarElement {
    common_game_settings: Arc<CommonGameplaySettings>,
    health_ratio: f32,
    container_size: Vector2,

    
    last_health_ratio: f32,
    last_health_time: f32,

    healthbar_color: Option<Image>,
    healthbar_color_1: Option<Image>,
    healthbar_bg_image: Option<Image>,
    healthbar_bg_image_1: Option<Image>,
}
impl HealthBarElement {
    fn build(
        _: &GamemodeInfo,
        settings: &Arc<CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        Box::new(Self {
            common_game_settings: settings.clone(),
            health_ratio: 0.0,
            last_health_ratio: -1000.0,
            
            container_size: Vector2::ONE,
            last_health_time: 0.0,

            healthbar_color: None,
            healthbar_bg_image: None,
            healthbar_color_1: None,
            healthbar_bg_image_1: None,
        })
    }
}
impl GameplayWidget for HealthBarElement {
    fn display_name(&self) -> &'static str { "Health Bar" }

    fn max_size(&self) -> Vector2 {
        Vector2::new(self.container_size.x / 2.0, DURATION_HEIGHT)
    }

    fn reload_skin(
        &mut self, 
        source: &TextureSource, 
        skin_manager: &mut dyn SkinProvider,
    ) {
        // peppy calls the healthbar texture "scorebar"
        self.healthbar_color = skin_manager.get_texture(
            "scorebar-colour", 
            source, 
            SkinUsage::Gamemode, 
            false
        );
        self.healthbar_color_1 = skin_manager.get_texture(
            "scorebar-colour1", 
            source, 
            SkinUsage::Gamemode, 
            false
        );
        self.healthbar_bg_image = skin_manager.get_texture(
            "scorebar-bg", 
            source, 
            SkinUsage::Gamemode, 
            false
        );
        self.healthbar_bg_image_1 = skin_manager.get_texture(
            "scorebar-bg1", 
            source, 
            SkinUsage::Gamemode, 
            false
        );

        for i in [ 
            &mut self.healthbar_color, 
            &mut self.healthbar_bg_image 
        ] {
            let Some(i) = i else { continue };
            i.origin = Vector2::ZERO;
            i.color = Color::WHITE;
        }
    }
    
    
    fn update(&mut self, manager: &mut dyn GameplayManagerTrait) {
        self.container_size = manager.bounds().size;

        self.health_ratio = manager.health().get_ratio();
        if self.last_health_ratio == -1000.0 {
            self.last_health_ratio = self.health_ratio; 
        }

        let time = manager.time();
        let time_diff = time - self.last_health_time;
        self.last_health_time = time;

        if self.health_ratio < self.last_health_ratio {
            let ratio_diff = (self.last_health_ratio - self.health_ratio)
                .max(0.05) * time_diff 
                * HEALTH_DAMP / 1000.0;
            self.last_health_ratio -= ratio_diff;
        } else {
            self.last_health_ratio = self.health_ratio;
        }
    }

    fn draw(
        &mut self, 
        pos_offset: Vector2, 
        scale: Vector2, 
        _align: Alignment,
        list: &mut RenderableCollection
    ) {
        let percent = self.health_ratio;
        let bg_size = Vector2::new(self.container_size.x / 2.0, DURATION_HEIGHT) * scale;
        let width = bg_size.x * percent;
        let scissor = [pos_offset.x, pos_offset.y, width, bg_size.y];

        if let Some(mut color) = self.healthbar_color.clone() {

            // add bg
            if let Some(mut bg) = self.healthbar_bg_image.clone() {
                bg.pos = pos_offset;
                let tex_size = bg.tex_size();
                let ratio = tex_size.y / tex_size.x;

                bg.set_size(Vector2::new(
                    bg_size.x,
                    bg_size.x * ratio
                ));

                list.push(bg);
            }
            if let Some(mut bg_1) = self.healthbar_bg_image_1.clone() {
                bg_1.pos = pos_offset;
                let tex_size = bg_1.tex_size();
                let ratio = tex_size.y / tex_size.x;

                bg_1.set_size(Vector2::new(
                    bg_size.x,
                    bg_size.x * ratio
                ));

                list.push(bg_1);
            }


            color.pos = pos_offset;
            let tex_size = color.tex_size();
            let ratio = tex_size.y / tex_size.x;
            color.set_size(Vector2::new(
                bg_size.x,
                bg_size.x * ratio
            ));

            
            list.push(ScissoredDrawable::new(
                scissor, 
                Box::new(color.clone())
            ));
            

            // // add drained health
            // let width2 = bg_size.x * self.last_health_ratio;
            // // if width2 != width {
            //     color.color.a = 0.2;
            //     list.push(ScissoredDrawable::new(
            //         [pos_offset.x + width, pos_offset.y, width2 - width, bg_size.y], 
            //         Box::new(color)
            //     ));
            // // }

            if let Some(mut color_1) = self.healthbar_color_1.clone() {
                color_1.pos = pos_offset;
                let tex_size = color_1.tex_size();
                let ratio = tex_size.y / tex_size.x;
                color_1.set_size(Vector2::new(bg_size.x, bg_size.x * ratio));

                list.push(ScissoredDrawable::new(
                    scissor, 
                    Box::new(color_1.clone())
                ));

                // // add drained health
                // let width2 = bg_size.x * self.last_health_ratio;
                // // if width2 != width {
                //     color_fill.color.a = 0.2;
                //     list.push(ScissoredDrawable::new(
                //         [pos_offset.x + width, pos_offset.y, width2 - width, bg_size.y], 
                //         Box::new(color_fill)
                //     ));
                // // }
            }

        } else {
            let len = self.common_game_settings.healthbar_colors.len();
            let index = ((len as f32 * percent) as usize).min(len - 1);

            // bg
            list.push(
                Rectangle::new(
                    pos_offset,
                    bg_size,
                    self.common_game_settings.healthbar_bg_color,
                ).border(Border::new(
                    self.common_game_settings.healthbar_border_color, 
                    1.8
                ))
            );

            // fill
            list.push(Rectangle::new(
                pos_offset,
                Vector2::new((self.container_size.x / 2.0) * percent, DURATION_HEIGHT) * scale,
                self.common_game_settings.healthbar_colors[index],
            ));
        }


    }
}


pub const HEALTH_BAR: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "health_bar",
    default_layout: GameplayWidgetLayout::new_default(
        GameplayWidgetAnchor::Screen, 
        Alignment::TOP_LEFT,
        None,
        None,
    ),
    build: HealthBarElement::build,
};
