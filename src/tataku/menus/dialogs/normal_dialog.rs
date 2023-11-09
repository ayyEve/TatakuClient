#![allow(unused, dead_code)]

use encoding_rs::WINDOWS_1250;

use crate::prelude::*;
const Y_PADDING:f32 = 5.0;
const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 30.0);

pub type ClickFn = Box<dyn Fn(&mut GenericDialog, &mut Game) + Send + Sync>;

pub struct GenericDialog {
    bounds: Rectangle,
    layout_manager: LayoutManager,

    buttons: Vec<MenuButton>,
    actions: HashMap<String, ClickFn>,
    pub should_close: bool,
    pub close_after_click: bool,

    window_size: Arc<WindowSize>,
}
impl GenericDialog {
    pub fn new(_title: impl AsRef<str>) -> Self {
        let window_size = WindowSize::get();
        let layout_manager = LayoutManager::new();
        layout_manager.set_style(Style {
            align_items: Some(taffy::style::AlignItems::Center),
            ..Default::default()
        });

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
            layout_manager,
            buttons: Vec::new(),
            actions: HashMap::new(),
            close_after_click: false,

            should_close: false,
            window_size
        }
    }

    pub fn add_button(&mut self, text: impl ToString, on_click: ClickFn) {
        let text = text.to_string();
        // let y_pos = 100.0 + (BUTTON_SIZE.y + Y_PADDING) * self.buttons.len() as f32;

        let mut button = MenuButton::new(
            Style {
                size: Size {
                    width: Dimension::Percent(0.2),
                    height: Dimension::Auto,
                },
                ..Default::default()
            },
            &text,
            &self.layout_manager,
            Font::Main,
        ).with_tag(&text);
        self.buttons.push(button);
        self.actions.insert(text, on_click);

        self.layout_manager.apply_layout(self.bounds.size);
        let layout = self.layout_manager.clone();
        self.buttons.iter_mut().for_each(|i|i.apply_layout(&layout, Vector2::ZERO));
    }
}

#[async_trait]
impl Dialog<Game> for GenericDialog {
    fn should_close(&self) -> bool { self.should_close }
    fn get_bounds(&self) -> Bounds { *self.bounds }
    async fn force_close(&mut self) { self.should_close = true; }

    fn container_size_changed(&mut self, size: Vector2) {
        // self.window_size = window_size;
        self.layout_manager.apply_layout(size);

        for b in self.buttons.iter_mut() {
            b.apply_layout(&self.layout_manager, Vector2::ZERO);
        }
    }


    async fn on_key_press(&mut self, key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
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
                self.should_close = self.close_after_click;
                break
            }
        }
        self.buttons = buttons;
        self.actions = actions;

        true
    }

    async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
        // background and border
        let mut bounds = self.bounds;
        bounds.pos += offset;
        list.push(bounds);

        // draw buttons
        for button in self.buttons.iter_mut() {
            button.draw(offset, list);
        }
    }
}
