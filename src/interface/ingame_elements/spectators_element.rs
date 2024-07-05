
use crate::prelude::*;

const SPECTATOR_ITEM_SIZE:Vector2 = Vector2::new(100.0, 40.0);
const PADDING:f32 = 4.0;


pub struct SpectatorsElement {
    spectators: SpectatorList
}
impl SpectatorsElement {
    pub fn new() -> Self {
        Self {
            spectators: SpectatorList::default()
        }
    }
}
#[async_trait]
impl InnerUIElement for SpectatorsElement {
    fn display_name(&self) -> &'static str { "Spectators" }

    fn get_bounds(&self) -> Bounds {
        Bounds::new(
            Vector2::ZERO,
            // TODO: setup a proper size
            Vector2::new(
                SPECTATOR_ITEM_SIZE.x,
                (SPECTATOR_ITEM_SIZE.y + PADDING) * 5.0 - PADDING
            )
        )
    }


    fn update(&mut self, manager: &mut GameplayManager) {
        if manager.spectator_info.spectators.updated {
            info!("updated spectator element list");
            self.spectators = manager.spectator_info.spectators.clone();
            manager.spectator_info.spectators.updated = false;
        }
    }

    fn draw(&mut self, pos_offset:Vector2, scale:Vector2, list: &mut RenderableCollection) {
        if self.spectators.list.is_empty() { return }

        // draw spectators
        list.push(visibility_bg(
            pos_offset,
            Vector2::new(SPECTATOR_ITEM_SIZE.x, (SPECTATOR_ITEM_SIZE.y + PADDING) * self.spectators.list.len() as f32) * scale,
        ));

        for (i, user) in self.spectators.list.iter().enumerate() {
            // draw username
            list.push(Text::new(
                pos_offset + Vector2::new(0.0, (SPECTATOR_ITEM_SIZE.y + PADDING) * i as f32) * scale,
                30.0 * scale.y,
                &user.username,
                Color::WHITE, 
                Font::Main
            ))
        }
    }

    async fn reload_skin(&mut self, _skin_manager: &mut SkinManager) {}
}
