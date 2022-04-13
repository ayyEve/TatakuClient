use crate::prelude::*;

pub struct BeatmapDialog {
    bounds: Rectangle,
    target_map: String,
    delete_map: MenuButton<Font2, Text>,
    should_close: bool
}
impl BeatmapDialog {
    pub fn new(map_hash: String) -> Self {
        let window = Settings::window_size();

        const Y_PADDING:f64 = 5.0;
        const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 30.0);

        let offset = 100.0;
        let count = 0;

        let delete_map = MenuButton::new(
            Vector2::new((window.x - BUTTON_SIZE.x) / 2.0, offset + (count as f64 * (BUTTON_SIZE.y + Y_PADDING))),
            BUTTON_SIZE,
            "Delete Map",
            get_font(),
        );


        let bounds = Rectangle::new(
            Color::BLACK.alpha(0.7),
            0.0,
            Vector2::zero(),
            window,
            Some(Border::new(
                Color::BLACK, 
                1.5
            ))
        );
        
        Self {
            bounds,
            delete_map,
            target_map: map_hash,

            should_close: false
        }
    }
}

#[async_trait]
impl Dialog<Game> for BeatmapDialog {
    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }
    fn should_close(&self) -> bool {
        self.should_close
    }

    async fn on_key_press(&mut self, key:&Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        if key == &Key::Escape {
            self.should_close = true;
            return true
        }

        false
    }

    async fn on_mouse_move(&mut self, pos:&Vector2, _g:&mut Game) {
        self.delete_map.on_mouse_move(*pos)
    }
    async fn on_mouse_down(&mut self, pos:&Vector2, button:&MouseButton, mods:&KeyModifiers, game:&mut Game) -> bool {
        if self.delete_map.on_click(*pos, *button, *mods) {
            trace!("delete map {}", self.target_map);

            BEATMAP_MANAGER.write().await.delete_beatmap(self.target_map.clone(), game).await;
            self.should_close = true;
        }
        true
    }

    async fn draw(&mut self, args:&RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>) {
        // background and border
        let mut bg_rect = self.bounds.clone();
        bg_rect.depth = *depth;


        // draw buttons
        let depth = depth - 0.0001;
        self.delete_map.draw(*args, Vector2::zero(), depth, list);

        list.push(Box::new(bg_rect));
    }

}
