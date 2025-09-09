use crate::prelude::*;

const SPECTATOR_ITEM_SIZE:Vector2 = Vector2::new(100.0, 40.0);
const PADDING:f32 = 4.0;


#[derive(Default)]
struct SpectatorsElement {
    // spectators: Vec<String>,
    layout: Option<(Arc<parley::Layout<Color>>, Vector2)>,
}
impl SpectatorsElement {
    fn build(
        _: &GamemodeInfo,
        _: &Arc<CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        Box::new(Self {
            // spectators: Vec::new(),
            layout: None,
        })
    }
}
impl GameplayWidget for SpectatorsElement {
    fn display_name(&self) -> &'static str { "Spectators" }

    fn max_size(&self) -> Vector2 {
        // TODO: setup a proper size
        Vector2::new(
            SPECTATOR_ITEM_SIZE.x,
            (SPECTATOR_ITEM_SIZE.y + PADDING) * 5.0 - PADDING
        )
    }


    fn update(&mut self, shell: &mut GameplayWidgetUpdateShell) {
        let spectators = shell.manager.spectators();
        if spectators.updated {
            spectators.updated = false;
            if spectators.list.is_empty() {
                self.layout = None;
                return;
            }

            // info!("updated spectator element list");
            // self.spectators = manager.spectators().clone();

            let style = TextStyle {
                font_size: 30.0 * shell.scale.y,
                line_height: SPECTATOR_ITEM_SIZE.y * shell.scale.y,
                color: Color::WHITE,
                ..Default::default()
            };

            let text = spectators.list
                .iter()
                .fold(
                    format!("Spectators: ({})\n", spectators.list.len()),
                    |i, v| i + &v.username + "\n"
                );
            
            let mut layout = shell.font_context.simple_text(
                text.trim(), 
                &style,
            );

            layout.break_all_lines(None);
            let size = Vector2::new(
                layout.width(),
                layout.height()
            );

            self.layout = Some((Arc::new(layout), size));
        }
    }

    fn draw(
        &mut self,
        shell: &mut GameplayWidgetDrawShell,
    ) {
        let Some((layout, layout_size)) = &self.layout 
        else { return };

        // if self.spectators.list.is_empty() { return }

        // draw spectators
        shell.list.push(Rectangle::new(
            shell.pos_offset,
            *layout_size,
            Color::WHITE.alpha(0.8),
        ));

        shell.list.push(Transformed::new(
            Transform::default()
                .translate(shell.pos_offset + PADDING),
            Box::new(Text::new(layout.clone()))
        ));

        // for (i, user) in self.spectators.list.iter().enumerate() {
        //     // draw username
        //     // list.push(Text::new(
        //     //          pos_offset + Vector2::new(
        //     //          0.0,
        //     //          (SPECTATOR_ITEM_SIZE.y + PADDING) * i as f32
        //     //     ) * scale,
        //     //     30.0 * scale.y,
        //     //     &user.username,
        //     //     Color::WHITE,
        //     //     DefaultFont::Main
        //     // ));
        // }
    }
}



pub const SPECTATORS: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "spectators",
    default_layout: GameplayWidgetLayout::new_default(
        GameplayWidgetAnchor::element(
            "health_bar",
            GameplayWidgetAlign::Below
        ),
        Alignment::TOP_LEFT,
        None,
        None,
    ),
    build: SpectatorsElement::build,
};
