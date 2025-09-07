use crate::prelude::*;

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

    fn max_size(&self) -> Vector2 {
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
        shell.list.push(Rectangle::new(
            shell.pos_offset, // - Vector2::with_y(DURATION_HEIGHT + 3.0),
            Vector2::new(
                self.container_size.x * self.duration_ratio, 
                DURATION_HEIGHT
            ) * shell.scale,
            self.common_game_settings.duration_color_full,
        ));

        // border
        shell.list.push(
            Rectangle::new(
                shell.pos_offset, // + Vector2::with_y(-(DURATION_HEIGHT + 3.0)),
                Vector2::new(self.container_size.x, DURATION_HEIGHT) * shell.scale,
                self.common_game_settings.duration_color,
            )
            .border(Border::new(
                self.common_game_settings.duration_border_color, 
                1.8 * shell.scale.x
            )
        ));
    }
}


pub const DURATION_BAR: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "duration_bar",
    default_layout: GameplayWidgetLayout::new_default(
        GameplayWidgetAnchor::Screen, 
        Alignment::BOTTOM_LEFT,
        None,
        None,
    ),
    build: DurationBarElement::build,
};