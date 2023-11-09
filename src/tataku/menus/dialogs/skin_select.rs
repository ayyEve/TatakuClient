use crate::prelude::*;


pub struct SkinSelect {
    should_close: bool,
    layout_manager: LayoutManager,
    dropdown: Dropdown<SkinDropdownable>,
    current_skin: String
}
impl SkinSelect {
    pub async fn new() -> Self {
        let current_skin = Settings::get().current_skin.clone();
        let layout_manager = LayoutManager::new();
        Self {
            dropdown: Dropdown::new(
                Style::default(),
                20.0,
                "Skin",
                Some(SkinDropdownable::Skin(current_skin.clone())),
                &layout_manager,
                Font::Main
            ),
            layout_manager,
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
                Settings::get_mut().current_skin = s;
            }
        }
    }
}
#[async_trait]
impl Dialog<Game> for SkinSelect {
    fn name(&self) -> &'static str { "skin_select" }
    fn should_close(&self) -> bool { self.should_close }
    fn get_bounds(&self) -> Bounds { Bounds::new(Vector2::ZERO, WindowSize::get().0) }
    async fn force_close(&mut self) { self.should_close = true; }
    
    async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
        self.draw_background(Color::WHITE, offset, list);
        self.dropdown.draw(offset, list)
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

    async fn on_key_press(&mut self, _key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        true
    }
    async fn on_key_release(&mut self, _key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        true
    }

    
    fn container_size_changed(&mut self, size: Vector2) {
        self.layout_manager.apply_layout(size);
        
    }
}
