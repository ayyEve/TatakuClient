use crate::prelude::*;

const BASE_SIZE:Vector2 = Vector2::new(400.0, 700.0);
const INPUT_SIZE:Vector2 = Vector2::new(400.0, 30.0);
const PADDING:Vector2 = Vector2::new(0.0, 5.0);


pub struct GameImportDialog {
    pos: Vector2,
    should_close: bool,

    input_scrollable: ScrollableArea,

    add_button: MenuButton<Font2, Text>,
    confirm_button: MenuButton<Font2, Text>,
}
impl GameImportDialog {
    pub async fn new() -> Self {
        let font = get_font();
        let item_size = Vector2::new(20.0, 50.0);
        let button_height = 50.0; // AKA bottom margin

        let mut scrollable = ScrollableArea::new(
            Vector2::new(10.0, 30.0),
            BASE_SIZE - (item_size + Vector2::y_only(button_height)),
            true
        );

        Settings::get()
            .await
            .external_games_folders
            .iter()
            .for_each(|f| {
                scrollable.add_item(Box::new(TextInput::<Font2, Text>::new(
                    INPUT_SIZE.y() + PADDING,
                    INPUT_SIZE,
                    "Game Path",
                    f,
                    font.clone()
                )))
            });

        
        let add_button = MenuButton::<Font2, Text>::new(
            Vector2::new(0.0, BASE_SIZE.y - button_height),
            Vector2::new(100.0, button_height),
            "Add",
            font.clone()
        );

        let confirm_button = MenuButton::<Font2, Text>::new(
            Vector2::new(120.0, BASE_SIZE.y - button_height),
            Vector2::new(100.0, button_height),
            "Done",
            font.clone()
        );

        Self {
            pos: Vector2::one() * 200.0,

            should_close: false,
            input_scrollable: scrollable,

            add_button,
            confirm_button
        }
    }
}

#[async_trait]
impl Dialog<Game> for GameImportDialog {
    fn name(&self) -> &'static str {"game_import"}
    fn should_close(&self) -> bool {self.should_close}
    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(self.pos, BASE_SIZE)
    }
    
    async fn draw(&mut self, args:&RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>) {
        let pos = self.pos;

        self.draw_background(*depth + 10.0, Color::WHITE, list);

        self.input_scrollable.draw(*args, pos, *depth, list);
        self.add_button.draw(*args, pos, *depth, list);
        self.confirm_button.draw(*args, pos, *depth, list);
    }

    async fn update(&mut self, _g:&mut Game) {
        self.input_scrollable.update();
        self.add_button.update();
        self.confirm_button.update();
    }

    async fn on_mouse_move(&mut self, p:&Vector2, _g:&mut Game) {
        let p = *p - self.pos;
        self.input_scrollable.on_mouse_move(p);
        self.add_button.on_mouse_move(p);
        self.confirm_button.on_mouse_move(p);
    }

    async fn on_mouse_scroll(&mut self, delta:&f64, _g:&mut Game) -> bool {
        self.input_scrollable.on_scroll(*delta);
        self.add_button.on_scroll(*delta);
        self.confirm_button.on_scroll(*delta);
        true
    }

    async fn on_mouse_down(&mut self, pos:&Vector2, button:&MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
        let pos = *pos - self.pos;

        self.input_scrollable.on_click(pos, *button, *mods);

        if self.add_button.on_click(pos, *button, *mods) {
            self.input_scrollable.add_item(Box::new(TextInput::<Font2, Text>::new(
                INPUT_SIZE.y() + PADDING,
                INPUT_SIZE,
                "Game Path",
                "",
                get_font()
            )))
        }

        if self.confirm_button.on_click(pos, *button, *mods) {
            let mut settings = get_settings_mut!();
            settings.external_games_folders.clear();

            for i in self.input_scrollable.items.iter() {
                if let Some(path) = i.get_value().downcast_ref::<String>() {
                    if !path.is_empty() {
                        settings.external_games_folders.push(path.clone());
                    } else {
                        warn!("empty path")
                    }
                } else {
                    warn!("bad cast")
                }
            }
            self.should_close = true;
        }

        true
    }

    async fn on_mouse_up(&mut self, pos:&Vector2, button:&MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        let pos = *pos - self.pos;
        self.input_scrollable.on_click_release(pos, *button);
        self.add_button.on_click_release(pos, *button);
        self.confirm_button.on_click_release(pos, *button);
        true
    }

    async fn on_text(&mut self, text:&String) -> bool {
        self.input_scrollable.on_text(text.clone());
        true
    }

    async fn on_key_press(&mut self, key:&Key, mods:&KeyModifiers, _g:&mut Game) -> bool {
        if key == &Key::Escape { self.should_close = true }
        
        self.input_scrollable.on_key_press(*key, *mods);
        self.add_button.on_key_press(*key, *mods);
        self.confirm_button.on_key_press(*key, *mods);
        
        true
    }
    async fn on_key_release(&mut self, key:&Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.input_scrollable.on_key_release(*key);
        self.add_button.on_key_release(*key);
        self.confirm_button.on_key_release(*key);
        true
    }

    
    async fn window_size_changed(&mut self, _window_size: Arc<WindowSize>) {
        
    }
}