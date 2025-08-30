use crate::prelude::*;

const SPECTATOR_ITEM_SIZE:Vector2 = Vector2::new(100.0, 40.0);
const PADDING:f32 = 4.0;


#[derive(Default)]
struct SpectatorsElement {
    spectators: SpectatorList
}
impl SpectatorsElement {
    fn build(
        _: &GamemodeInfo,
        _: &Arc<CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        Box::new(Self {
            spectators: SpectatorList::default()
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


    fn update(&mut self, manager: &mut dyn GameplayManagerTrait) {
        if manager.spectators().updated {
            info!("updated spectator element list");
            self.spectators = manager.spectators().clone();
            manager.spectators().updated = false;
        }
    }

    fn draw(
        &mut self,
        pos_offset: Vector2,
        scale: Vector2,
        _align: Alignment,
        list: &mut RenderableCollection
    ) {
        if self.spectators.list.is_empty() { return }

        // draw spectators
        list.push(Rectangle::new(
            pos_offset,
            Vector2::new(
                SPECTATOR_ITEM_SIZE.x,
                (SPECTATOR_ITEM_SIZE.y + PADDING)
                * self.spectators.list.len() as f32
            ) * scale,
            Color::WHITE.alpha(0.8),
        ));

        for (i, user) in self.spectators.list.iter().enumerate() {
            // draw username
            // list.push(Text::new(
            //     pos_offset
            //         + Vector2::new(
            //             0.0,
            //             (SPECTATOR_ITEM_SIZE.y + PADDING) * i as f32
            //         )
            //         * scale,
            //     30.0 * scale.y,
            //     &user.username,
            //     Color::WHITE,
            //     DefaultFont::Main
            // ));
        }
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
