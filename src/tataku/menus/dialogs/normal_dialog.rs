#![allow(unused, dead_code)]

use crate::prelude::*;
const Y_PADDING:f32 = 5.0;
const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 30.0);

pub type ClickFn = Box<dyn Fn(&mut GenericDialog, &mut Game) + Send + Sync>;

pub struct GenericDialog {
    bounds: Rectangle,
    buttons: Vec<MenuButton>,
    actions: HashMap<String, ClickFn>,
    pub should_close: bool,
    window_size: Arc<WindowSize>
}
impl GenericDialog {
    pub fn new(_title: impl AsRef<str>) -> Self {
        let window_size = WindowSize::get();

        let bounds = Rectangle::new(
            Vector2::ZERO,
            window_size.0,
            Color::BLACK.alpha(0.7),
            Some(Border::new(
                Color::BLACK, 
                1.5
            ))
        );
        
        Self {
            bounds,
            buttons: Vec::new(),
            actions: HashMap::new(),

            should_close: false,
            window_size
        }
    }

    pub fn add_button(&mut self, text: impl AsRef<str>, on_click: ClickFn) {
        let text = text.as_ref().to_owned();

        let y_pos = 100.0 + (BUTTON_SIZE.y + Y_PADDING) * self.buttons.len() as f32;

        let mut button = MenuButton::new(
            Vector2::new((self.window_size.x - BUTTON_SIZE.x) / 2.0, y_pos),
            BUTTON_SIZE,
            &text,
            get_font(),
        );
        button.set_tag(&text);
        self.buttons.push(button);
        self.actions.insert(text, on_click);
    }
}

#[async_trait]
impl Dialog<Game> for GenericDialog {
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size;
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }
    fn should_close(&self) -> bool {
        self.should_close
    }

    async fn on_key_press(&mut self, key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        if key == Key::Escape {
            self.should_close = true;
            return true
        }

        false
    }

    async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
        for button in self.buttons.iter_mut() {
            button.on_mouse_move(pos)
        }
    }
    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, game:&mut Game) -> bool {
        let mut buttons = std::mem::take(&mut self.buttons);
        let actions = std::mem::take(&mut self.actions);

        for m_button in buttons.iter_mut() {
            if m_button.on_click(pos, button, *mods) {
                let tag = m_button.get_tag();
                let action = actions.get(&tag).unwrap();
                action(self, game);
                // self.should_close = true;
                break
            }
        }
        self.buttons = buttons;
        self.actions = actions;

        true
    }

    async fn draw(&mut self, list: &mut RenderableCollection) {
        // background and border
        list.push(self.bounds.clone());

        // draw buttons
        for button in self.buttons.iter_mut() {
            button.draw(Vector2::ZERO, list);
        }
    }
}