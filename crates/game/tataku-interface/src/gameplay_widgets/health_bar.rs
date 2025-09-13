use crate::prelude::*;
use tataku::{
    Alignment,
    Border,
    Vector2,
    Color,
};
use engine::{
    settings::common_gameplay::CommonGameplaySettings,
    gameplay::{
        widgets::*,
        GamemodeInfo,
    },
};
use graphics::{
    Image,
    SkinUsage,
};

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
        Vector2::new(self.container_size.x / 2.0, super::DURATION_HEIGHT)
    }

    fn reload_skin(
        &mut self,
        shell: &mut GameplayWidgetReloadSkinShell,
    ) {
        // peppy calls the healthbar texture "scorebar"
        self.healthbar_color = shell.skin_manager.get_texture(
            "scorebar-colour",
            shell.source,
            SkinUsage::Gamemode,
            false
        );
        self.healthbar_color_1 = shell.skin_manager.get_texture(
            "scorebar-colour1",
            shell.source,
            SkinUsage::Gamemode,
            false
        );
        self.healthbar_bg_image = shell.skin_manager.get_texture(
            "scorebar-bg",
            shell.source,
            SkinUsage::Gamemode,
            false
        );
        self.healthbar_bg_image_1 = shell.skin_manager.get_texture(
            "scorebar-bg1",
            shell.source,
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


    fn update(&mut self, shell: &mut GameplayWidgetUpdateShell) {
        self.container_size = shell.manager.bounds().size;

        self.health_ratio = shell.manager.health().get_ratio();
        if self.last_health_ratio == -1000.0 {
            self.last_health_ratio = self.health_ratio;
        }

        let time = shell.manager.time();
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
        shell: &mut GameplayWidgetDrawShell
    ) {
        let percent = self.health_ratio;

        if let Some(mut color) = self.healthbar_color.clone() {
            let tex_size = color.tex_size();
            let width = tex_size.x * percent;

            let scissor = [
                shell.pos_offset.x, 
                shell.pos_offset.y, 
                width * shell.scale.x, 
                tex_size.y * shell.scale.y
            ];

            // add bg
            if let Some(mut bg) = self.healthbar_bg_image.clone() {
                bg.pos = shell.pos_offset;
                bg.scale *= shell.scale;

                shell.list.push(bg);
            }
            if let Some(mut bg_1) = self.healthbar_bg_image_1.clone() {
                bg_1.pos = shell.pos_offset;
                bg_1.scale *= shell.scale;

                shell.list.push(bg_1);
            }


            color.pos = shell.pos_offset;
            color.scale *= shell.scale;

            shell.list.push(graphics::Scissored::new(
                scissor,
                Box::new(color)
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
                color_1.pos = shell.pos_offset;
                color_1.scale *= shell.scale;

                shell.list.push(graphics::Scissored::new(
                    scissor,
                    Box::new(color_1)
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
            let bg_size = Vector2::new(
                self.container_size.x / 2.0, 
                super::DURATION_HEIGHT
            ) * shell.scale;

            let len = self.common_game_settings.healthbar_colors.len();
            let index = ((len as f32 * percent) as usize).min(len - 1);

            // bg
            shell.list.push(graphics::Rectangle::new(
                shell.pos_offset,
                bg_size,
                self.common_game_settings.healthbar_bg_color,
            ).border(Border::new(
                self.common_game_settings.healthbar_border_color,
                1.8
            )));

            // fill
            shell.list.push(graphics::Rectangle::new(
                shell.pos_offset,
                Vector2::new(
                    (self.container_size.x / 2.0) * percent, 
                    super::DURATION_HEIGHT
                ) * shell.scale,
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
