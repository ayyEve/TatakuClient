use crate::prelude::*;


pub struct SkinSelect {
    num: usize,
    should_close: bool,
    // dropdown: Dropdown<SkinDropdownable>,
    // current_skin: String
    // current_skin: CurrentSkinHelper,
}
impl SkinSelect {
    pub async fn new() -> Self {
        // let current_skin = Settings::get().current_skin.clone();
        Self {
            num: 0,

            // dropdown: Dropdown::new(
            //     Vector2::new(300.0, 200.0),
            //     500.0,
            //     20.0,
            //     "Skin",
            //     Some(SkinDropdownable::Skin(current_skin.clone())),
            //     Font::Main
            // ),
            // current_skin: CurrentSkinHelper::new(),  
            should_close: false,
        }
    }

    // async fn check_skin_change(&mut self) {
    //     let selected = self.dropdown.get_value().downcast::<Option<SkinDropdownable>>();
    //     if let Ok(s) = selected {
    //         if let Some(SkinDropdownable::Skin(s)) = *s {
    //             if s == self.current_skin { return }

    //             trace!("skin changing to {}", s);
    //             self.current_skin = s.clone();
    //             Settings::get_mut().current_skin = s;
    //         }
    //     }
    // }
}
#[async_trait]
impl Dialog for SkinSelect {
    fn name(&self) -> &'static str { "skin_select" }
    
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }

    fn should_close(&self) -> bool { self.should_close }
    // // fn get_bounds(&self) -> Bounds { Bounds::new(Vector2::ZERO, WindowSize::get().0) }
    async fn force_close(&mut self) { self.should_close = true; }
    

    async fn handle_message(&mut self, message: Message, _values: &mut ValueCollection) {
        let Some(tag) = message.tag.as_string() else { return }; 

        Settings::get_mut().current_skin = tag;
    }


    
    async fn update(&mut self, _values: &mut ValueCollection) -> Vec<TatakuAction> { 
        // self.current_skin.update();

        Vec::new()
    }


    fn view(&self) -> IcedElement {
        use iced_elements::*;
        let current_skin = Settings::get().current_skin.clone();
        
        let owner = MessageOwner::new_dialog(self);
        Dropdown::new(AVAILABLE_SKINS.read().clone(), Some(current_skin), move|s|Message::new(owner, "selected_skin", MessageType::Dropdown(s)))
        .into_element()
    }




    // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
    //     self.draw_background(Color::WHITE, offset, list);
    //     self.dropdown.draw(offset, list)
    // }

    // async fn update(&mut self, _g:&mut Game) {
    //     self.dropdown.update()
    // }

    // async fn on_mouse_move(&mut self, p:Vector2, _g:&mut Game) {
    //     self.dropdown.on_mouse_move(p)
    // }

    // async fn on_mouse_scroll(&mut self, delta:f32, _g:&mut Game) -> bool {
    //     self.dropdown.on_scroll(delta);
    //     true
    // }

    // async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
    //     self.dropdown.on_click(pos, button, *mods);
    //     self.check_skin_change().await;
    //     true
    // }
    // async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
    //     self.dropdown.on_click_release(pos, button);
    //     true
    // }

    // async fn on_text(&mut self, _text:&String) -> bool {
    //     true
    // }

    // async fn on_key_press(&mut self, _key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
    //     true
    // }
    // async fn on_key_release(&mut self, _key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
    //     true
    // }

    
    // async fn window_size_changed(&mut self, _window_size: Arc<WindowSize>) {
        
    // }
}
