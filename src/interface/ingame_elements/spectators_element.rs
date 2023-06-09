
use crate::prelude::*;

const SPECTATOR_ITEM_SIZE:Vector2 = Vector2::new(100.0, 40.0);
const PADDING:f32 = 4.0;


pub struct SpectatorsElement {
    spectator_cache: Vec<(u32, String)>
}
impl SpectatorsElement {
    pub fn new() -> Self {
        Self {
            spectator_cache: Vec::new()
        }
    }
}

impl InnerUIElement for SpectatorsElement {
    fn display_name(&self) -> &'static str { "Spectators" }

    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::ZERO,
            // TODO: setup a proper size
            Vector2::new(
                SPECTATOR_ITEM_SIZE.x,
                (SPECTATOR_ITEM_SIZE.y + PADDING) * 5.0 - PADDING
            )
        )
    }


    fn update(&mut self, manager: &mut IngameManager) {
        self.spectator_cache = manager.spectator_cache.clone();
    }

    fn draw(&mut self, pos_offset:Vector2, scale:Vector2, list: &mut RenderableCollection) {

        // draw spectators
        if self.spectator_cache.len() > 0 {
            const DEPTH:f32 = -1000.0;


            list.push(visibility_bg(
                pos_offset,
                Vector2::new(SPECTATOR_ITEM_SIZE.x, (SPECTATOR_ITEM_SIZE.y + PADDING) * self.spectator_cache.len() as f32) * scale,
                DEPTH
            ));
            let font = get_font();
            for (i, (_, username)) in self.spectator_cache.iter().enumerate() {
                // draw username
                list.push(Text::new(
                    Color::WHITE, 
                    DEPTH - 0.001, 
                    pos_offset + Vector2::new(0.0, (SPECTATOR_ITEM_SIZE.y + PADDING) * i as f32) * scale,
                    30.0 * scale.y,
                    username.clone(),
                    font.clone()
                ))
            }
        }

    }
}
