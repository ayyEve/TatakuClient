use crate::prelude::*;
use tataku::{
    Border,
    Vector2,
};
use engine::{
    settings::common_gameplay::CommonGameplaySettings,
    gameplay::{
        widgets::*,
        GamemodeInfo,
    },
};

/// how tall is the duration bar
pub(crate) const DURATION_HEIGHT:f32 = 35.0;

struct DurationBarElement {
    common_game_settings: Arc<CommonGameplaySettings>,
    duration_ratio: f32,
    container_size: Vector2
}
impl DurationBarElement {
    fn build(
        _: &GamemodeInfo,
        settings: &Arc<CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        Box::new(Self {
            common_game_settings: settings.clone(),
            duration_ratio: 0.0,
            container_size: Vector2::ONE,
        })
    }
}
impl GameplayWidget for DurationBarElement {
    fn display_name(&self) -> &'static str { "Duration Bar" }

    fn preferred_size(&self) -> Vector2 {
        Vector2::new(self.container_size.x, DURATION_HEIGHT)
    }

    fn update(&mut self, shell: &mut GameplayWidgetUpdateShell) {
        self.container_size = shell.manager.bounds().size;
        self.duration_ratio = shell.manager.time() / shell.manager.end_time();
    }

    fn draw(
        &mut self,
        shell: &mut GameplayWidgetDrawShell
    ) {
        // fill
        shell.list.push(graphics::Transformed {
            transform: shell.transform,
            drawable: Box::new(graphics::Rectangle::new(
                Vector2::ZERO,
                Vector2::new(
                    self.container_size.x * self.duration_ratio,
                    DURATION_HEIGHT
                ) ,
                self.common_game_settings.duration_color_full,
            )),
        });

        // border
        shell.list.push(graphics::Transformed {
            transform: shell.transform,
            drawable: Box::new(graphics::Rectangle::new(
                Vector2::ZERO,
                Vector2::new(self.container_size.x, DURATION_HEIGHT),
                self.common_game_settings.duration_color,
            )
            .border(Border::new(
                self.common_game_settings.duration_border_color,
                1.8
            ))),
        });
    }
}

pub const DURATION_BAR: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "duration_bar",
    default_layout: GameplayWidgetLayout {
        anchor: GameplayWidgetAnchor::Screen,
        align: tataku::Alignment::BOTTOM_LEFT,
        transform: graphics::Transform::identity(),
    },
    build: DurationBarElement::build,
};
