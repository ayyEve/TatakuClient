use crate::prelude::*;


pub struct SkinSelect {
    should_close: bool,
    dropdown: Dropdown<SkinDropdownable>,
    current_skin: String
}
impl SkinSelect {
    pub async fn new() -> Self {
        let current_skin = get_settings!().current_skin.clone();
        Self {
            dropdown: Dropdown::new(
                Vector2::new(300.0, 200.0),
                500.0,
                20.0,
                "Skin",
                Some(SkinDropdownable::Skin(current_skin.clone())),
                get_font()
            ),
            current_skin,
            should_close: false,
        }
    }

    async fn check_skin_change(&mut self) {
        let selected = self.dropdown.get_value().downcast::<Option<SkinDropdownable>>();
        if let Ok(s) = selected {
            if let Some(SkinDropdownable::Skin(s)) = *s {
                if s == self.current_skin { return }

                trace!("skin changing to {}", s);
                self.current_skin = s.clone();
                get_settings_mut!().current_skin = s;
            }
        }
    }
}
#[async_trait]
impl Dialog<Game> for SkinSelect {
    fn name(&self) -> &'static str {"skin_select"}
    fn should_close(&self) -> bool {self.should_close}
    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(Vector2::ZERO, WindowSize::get().0)
    }
    
    async fn draw(&mut self, depth: f32, list: &mut RenderableCollection) {
        self.draw_background(depth, Color::WHITE, list);
        self.dropdown.draw(Vector2::ZERO, depth, list)
    }

    async fn update(&mut self, _g:&mut Game) {
        self.dropdown.update()
    }

    async fn on_mouse_move(&mut self, p:Vector2, _g:&mut Game) {
        self.dropdown.on_mouse_move(p)
    }

    async fn on_mouse_scroll(&mut self, delta:f32, _g:&mut Game) -> bool {
        self.dropdown.on_scroll(delta);
        true
    }

    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.dropdown.on_click(pos, button, *mods);
        self.check_skin_change().await;
        true
    }
    async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.dropdown.on_click_release(pos, button);
        true
    }

    async fn on_text(&mut self, _text:&String) -> bool {
        true
    }

    async fn on_key_press(&mut self, key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        if key == Key::Escape {self.should_close = true}
        
        true
    }
    async fn on_key_release(&mut self, _key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        true
    }

    
    async fn window_size_changed(&mut self, _window_size: Arc<WindowSize>) {
        
    }
}
