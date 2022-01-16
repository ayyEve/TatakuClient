#![allow(unused, dead_code)]
//TODO: name this better

use crate::prelude::*;
const Y_PADDING:f64 = 5.0;
const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 30.0);

pub type ClickFn = Box<dyn Fn(&mut NormalDialog, &mut Game)>;

pub struct NormalDialog {
    bounds: Rectangle,
    buttons: Vec<MenuButton>,
    actions: HashMap<String, ClickFn>,
    pub should_close: bool
}
impl NormalDialog {
    pub fn new(_title: impl AsRef<str>) -> Self {
        let window = Settings::window_size();

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
            buttons: Vec::new(),
            actions: HashMap::new(),

            should_close: false
        }
    }

    pub fn add_button(&mut self, text: impl AsRef<str>, on_click: ClickFn) {
        let text = text.as_ref().to_owned();
        let window = Settings::window_size();

        let y_pos = 100.0 + (BUTTON_SIZE.y + Y_PADDING) * self.buttons.len() as f64;

        let mut button = MenuButton::new(
            Vector2::new((window.x - BUTTON_SIZE.x) / 2.0, y_pos),
            BUTTON_SIZE,
            &text
        );
        button.set_tag(&text);
        self.buttons.push(button);
        self.actions.insert(text, on_click);
    }
}
impl Dialog<Game> for NormalDialog {
    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }
    fn should_close(&self) -> bool {
        self.should_close
    }

    fn on_key_press(&mut self, key:&Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        if key == &Key::Escape {
            self.should_close = true;
            return true
        }

        false
    }

    fn on_mouse_move(&mut self, pos:&Vector2, _g:&mut Game) {
        for button in self.buttons.iter_mut() {
            button.on_mouse_move(*pos)
        }
    }
    fn on_mouse_down(&mut self, pos:&Vector2, button:&MouseButton, mods:&KeyModifiers, game:&mut Game) -> bool {
        let mut buttons = std::mem::take(&mut self.buttons);
        let actions = std::mem::take(&mut self.actions);

        for m_button in buttons.iter_mut() {
            if m_button.on_click(*pos, *button, *mods) {
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

    fn draw(&mut self, args:&RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>) {
        // background and border
        let mut bg_rect = self.bounds.clone();
        bg_rect.depth = *depth;

        // draw buttons
        let depth = depth - 0.0001;
        for button in self.buttons.iter_mut() {
            list.extend(button.draw(*args, Vector2::zero(), depth));
        }

        list.push(Box::new(bg_rect));
    }
}
