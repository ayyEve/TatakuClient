use crate::prelude::*;

use tokio::sync::mpsc::{Receiver, channel};

/// this is a dialog i'll use to test stuff with
pub struct TestDialog {
    list: ScrollableArea,
    add_button: MenuButton,
    search_text: TextInput,

    receiver: Receiver<()>,

    last_text: String,

    should_close: bool
}
impl TestDialog {
    pub fn new() -> Self {
        let mut add_button = MenuButton::new(Vector2::ZERO, Vector2::new(100.0, 50.0), "Add", get_font());
        let search_text = TextInput::new(Vector2::with_x(110.0), Vector2::new(200.0, 50.0), "Search", "", get_font());

        let (sender, receiver) = channel(10);
        add_button.on_click = Arc::new(move |_|sender.try_send(()).unwrap());

        let list = ScrollableArea::new(Vector2::with_y(60.0), Vector2::new(1000.0, 500.0), ListMode::Grid(GridSettings {
            item_margin: Vector2::new(5.0, 20.0),
            row_alignment: HorizontalAlign::Center,
            grid: Vec::new(),
        }));

        Self {
            list,
            search_text,
            add_button,
            receiver,
            last_text: String::new(),

            should_close: false,
        }
    }
}

#[async_trait]
impl Dialog<Game> for TestDialog {
    fn name(&self) -> &'static str { "test weee" }
    fn title(&self) -> &'static str { "Test Dialog" }
    fn should_close(&self) -> bool { self.should_close }
    fn get_bounds(&self) -> Bounds { Bounds::new(Vector2::ZERO, self.list.size() + Vector2::new(0.0, 50.0 + 60.0)) }
    async fn window_size_changed(&mut self, _window_size: Arc<WindowSize>) {}

    
    async fn force_close(&mut self) { self.should_close = true; }

    async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
        self.list.on_mouse_move(pos);
        self.add_button.on_mouse_move(pos);
        self.search_text.on_mouse_move(pos);
    }
    
    async fn on_mouse_scroll(&mut self, delta:f32, _g:&mut Game) -> bool {
        self.list.on_scroll(delta);
        true
    }
    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.list.on_click(pos, button, *mods);
        self.add_button.on_click(pos, button, *mods);
        self.search_text.on_click(pos, button, *mods);
        true
    }
    async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool { 
        self.list.on_click_release(pos, button);
        self.add_button.on_click_release(pos, button);
        self.search_text.on_click_release(pos, button);
        true
    }

    
    async fn on_text(&mut self, text:&String) -> bool {
        self.search_text.on_text(text.clone());
        true
    }
    async fn on_key_press(&mut self, key:Key, mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.search_text.on_key_press(key, *mods);
        true
    }
    async fn on_key_release(&mut self, key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.search_text.on_key_release(key);
        true
    }


    async fn update(&mut self, _g:&mut Game) {
        if let Ok(()) = self.receiver.try_recv() {
            let c = self.list.items.len() as u32;
            let item = PanelUser::new(OnlineUser::new(c, format!("User {c}")));
            self.list.add_item(Box::new(item));
        }

        let text = self.search_text.get_text();
        if text != self.last_text {
            if text.is_empty() {
                self.list.apply_filter(&Vec::new(), true);
            } else {
                let query = text.split(" ").map(|a|a.to_owned()).collect::<Vec<String>>();
                self.list.apply_filter(&query, true);
            }
            self.list.refresh_layout();

            self.last_text = text;
        }
    }

    async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
        list.push(visibility_bg(offset, self.get_bounds().size));

        self.list.draw(offset, list);
        self.add_button.draw(offset, list);
        self.search_text.draw(offset, list);
    }

}

